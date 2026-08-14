// SPDX-License-Identifier: Apache-2.0

//! Metrics module for the proof API.

#![allow(missing_docs)]

use prometheus::{
    register_counter, register_histogram_vec, register_int_counter_vec, register_int_gauge,
    Counter, HistogramVec, IntCounterVec, IntGauge,
};
use std::{sync::LazyLock, time::Instant};

/// Prometheus metric for total number of requests
pub static REQUEST_COUNTER: LazyLock<Counter> = LazyLock::new(|| {
    register_counter!("eureka_proof_api_request_total", "Total number of requests").unwrap()
});

/// Prometheus metric for response time in seconds, distinguished by method, `src_chain`, and `dst_chain`
pub static RESPONSE_TIME: LazyLock<HistogramVec> = LazyLock::new(|| {
    register_histogram_vec!(
        "eureka_proof_api_response_time_seconds",
        "Response time in seconds",
        &["method", "src_chain", "dst_chain"]
    )
    .unwrap()
});

/// Prometheus metric for response codes, distinguished by method, `src_chain`, `dst_chain`, and `status_code`
pub static RESPONSE_CODE: LazyLock<IntCounterVec> = LazyLock::new(|| {
    register_int_counter_vec!(
        "eureka_proof_api_response_codes",
        "Response Codes",
        &["method", "src_chain", "dst_chain", "status_code"]
    )
    .unwrap()
});

/// Prometheus metric for number of connected clients, or concurrent requests
pub static CONNECTED_CLIENTS: LazyLock<IntGauge> = LazyLock::new(|| {
    register_int_gauge!("eureka_proof_api_connected_clients", "Connected clients").unwrap()
});

/// Generic metrics tracking middleware for service calls
/// # Errors
/// Returns an error if the function itself returns an error.
pub async fn track_metrics<F, Fut, R>(
    method: &str,
    src_chain: &str,
    dst_chain: &str,
    f: F,
) -> Result<tonic::Response<R>, tonic::Status>
where
    F: FnOnce() -> Fut,
    Fut: std::future::Future<Output = Result<tonic::Response<R>, tonic::Status>>,
{
    let timer = Instant::now();
    CONNECTED_CLIENTS.inc();
    REQUEST_COUNTER.inc();

    let result = f().await;
    let status_code: isize = match &result {
        Ok(_) => 0,
        Err(status) => status.code() as isize,
    };

    RESPONSE_TIME
        .with_label_values(&[method, src_chain, dst_chain])
        .observe(timer.elapsed().as_secs_f64());

    RESPONSE_CODE
        .with_label_values(&[method, src_chain, dst_chain, &status_code.to_string()])
        .inc();

    CONNECTED_CLIENTS.dec();
    result
}
