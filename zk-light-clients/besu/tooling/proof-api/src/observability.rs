use std::{
    collections::BTreeMap,
    sync::{
        atomic::{AtomicU64, Ordering},
        Arc, Mutex,
    },
    time::Instant,
};

#[derive(Default)]
pub struct Metrics {
    inflight: AtomicU64,
    values: Mutex<BTreeMap<(&'static str, String), Value>>,
}
#[derive(Default)]
struct Value {
    count: u64,
    duration: f64,
    bytes: u64,
}
pub struct Measurement {
    metrics: Arc<Metrics>,
    method: &'static str,
    started: Instant,
    finished: bool,
    fields: serde_json::Value,
}
impl Metrics {
    pub fn start(self: &Arc<Self>, method: &'static str) -> Measurement {
        self.inflight.fetch_add(1, Ordering::Relaxed);
        Measurement {
            metrics: self.clone(),
            method,
            started: Instant::now(),
            finished: false,
            fields: serde_json::json!({}),
        }
    }
    pub fn render(&self) -> String {
        use std::fmt::Write;
        let mut output = format!("# TYPE besu_prover_up gauge\nbesu_prover_up 1\n# TYPE besu_prover_inflight_requests gauge\nbesu_prover_inflight_requests {}\n", self.inflight.load(Ordering::Relaxed));
        if let Ok(values) = self.values.lock() {
            for ((method, result), value) in values.iter() {
                let labels = format!("method=\"{method}\",result=\"{result}\"");
                let _ = writeln!(
                    output,
                    "besu_prover_requests_total{{{labels}}} {}",
                    value.count
                );
                let _ = writeln!(
                    output,
                    "besu_prover_request_duration_seconds_sum{{{labels}}} {}",
                    value.duration
                );
                let _ = writeln!(
                    output,
                    "besu_prover_request_duration_seconds_count{{{labels}}} {}",
                    value.count
                );
                let _ = writeln!(
                    output,
                    "besu_prover_response_bytes_total{{{labels}}} {}",
                    value.bytes
                );
            }
        }
        output
    }
}
impl Measurement {
    /// Capture identifiers before validation or moving the request into the engine.
    pub fn with_request(
        mut self,
        client: Option<&crate::proto::Client>,
        height: Option<u64>,
        kind: Option<i32>,
        packets: &[crate::proto::Packet],
    ) -> Self {
        static NEXT_REQUEST: AtomicU64 = AtomicU64::new(1);
        self.fields = serde_json::json!({
            "request_id": format!("{}-{}", prover_script::logging::timestamp_ms(), NEXT_REQUEST.fetch_add(1, Ordering::Relaxed)),
            "chain_id": client.map(|c| &c.chain_id),
            "client_id": client.map(|c| &c.client_id),
            "height": height,
            "proof_kind": kind,
            "packets": packets.iter().map(|p| serde_json::json!({
                "sequence": p.sequence, "source_client": p.source_client,
                "destination_client": p.destination_client,
            })).collect::<Vec<_>>(),
        });
        self
    }

    fn record(
        &self,
        result: &str,
        error: Option<&tonic::Status>,
        bytes: usize,
    ) -> serde_json::Value {
        let mut fields = self.fields.clone();
        fields["component"] = serde_json::json!("prover-service");
        fields["method"] = serde_json::json!(self.method);
        fields["result"] = serde_json::json!(result);
        fields["duration_ms"] = serde_json::json!(self.started.elapsed().as_millis());
        fields["response_bytes"] = serde_json::json!(bytes);
        fields["error"] = serde_json::json!(error.map(tonic::Status::message));
        fields
    }

    pub fn finish(mut self, error: Option<&tonic::Status>, bytes: usize) {
        let code = error.map_or(tonic::Code::Ok, tonic::Status::code);
        let result = match code {
            tonic::Code::Ok => "ok",
            tonic::Code::Cancelled => "canceled",
            tonic::Code::DeadlineExceeded => "deadline_exceeded",
            tonic::Code::InvalidArgument => "invalid_argument",
            tonic::Code::NotFound => "not_found",
            tonic::Code::ResourceExhausted => "resource_exhausted",
            tonic::Code::Unavailable => "unavailable",
            tonic::Code::FailedPrecondition => "failed_precondition",
            tonic::Code::Unimplemented => "unimplemented",
            _ => "internal",
        };
        let duration = self.started.elapsed();
        if let Ok(mut values) = self.metrics.values.lock() {
            let value = values.entry((self.method, result.to_owned())).or_default();
            value.count += 1;
            value.duration += duration.as_secs_f64();
            value.bytes += bytes as u64;
        }
        prover_script::logging::log(self.record(result, error, bytes));
        self.finished = true;
    }
}
impl Drop for Measurement {
    fn drop(&mut self) {
        self.metrics.inflight.fetch_sub(1, Ordering::Relaxed);
        if !self.finished {
            prover_script::logging::log(self.record("canceled", None, 0));
            if let Ok(mut values) = self.metrics.values.lock() {
                let value = values
                    .entry((self.method, "canceled".to_owned()))
                    .or_default();
                value.count += 1;
                value.duration += self.started.elapsed().as_secs_f64();
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::proto::{Client, Packet};

    #[test]
    fn failure_record_retains_all_packet_identifiers_without_payloads() {
        let metrics = Arc::new(Metrics::default());
        let measurement = metrics.start("packet_proofs").with_request(
            Some(&Client {
                chain_id: "chain-a".into(),
                client_id: "client-a".into(),
            }),
            Some(42),
            Some(1),
            &[
                Packet {
                    sequence: 7,
                    source_client: "source".into(),
                    destination_client: "destination".into(),
                    ..Default::default()
                },
                Packet {
                    sequence: 8,
                    source_client: "source".into(),
                    destination_client: "destination".into(),
                    ..Default::default()
                },
            ],
        );
        let error = tonic::Status::unavailable("PROVER_FAILED: test failure");
        let record = measurement.record("unavailable", Some(&error), 0);
        assert_eq!(record["chain_id"], "chain-a");
        assert_eq!(record["client_id"], "client-a");
        assert_eq!(record["height"], 42);
        assert_eq!(record["packets"][0]["sequence"], 7);
        assert_eq!(record["packets"][1]["sequence"], 8);
        assert_eq!(record["packets"][0]["source_client"], "source");
        assert_eq!(record["packets"][0]["destination_client"], "destination");
        assert_eq!(record["error"], error.message());
        assert!(record["request_id"].as_str().is_some_and(|s| !s.is_empty()));
        assert!(record["packets"][0].get("payloads").is_none());
        measurement.finish(Some(&error), 0);
        assert!(metrics.render().contains("result=\"unavailable\""));
        assert!(metrics.render().contains("besu_prover_inflight_requests 0"));
    }

    #[test]
    fn missing_client_and_cancellation_can_be_logged() {
        let measurement = Arc::new(Metrics::default())
            .start("state_proof")
            .with_request(None, Some(0), None, &[]);
        let record = measurement.record("canceled", None, 0);
        assert!(record["client_id"].is_null());
        assert_eq!(record["result"], "canceled");
        assert!(record["request_id"].is_string());
    }
}
