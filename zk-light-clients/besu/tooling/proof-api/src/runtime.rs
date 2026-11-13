use crate::{config::Config, engine::ProductionEngine, proto, service::Service, transport};
use std::{
    future::IntoFuture,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    time::Duration,
};
use tokio::{net::TcpListener, sync::oneshot, time::timeout};

pub async fn serve(config: Config) -> Result<(), Box<dyn std::error::Error>> {
    config.validate_runtime()?;
    let server = config.server.as_ref().ok_or("server is required")?.clone();
    let client = config.client.as_ref().ok_or("client is required")?.clone();
    let startup = timeout(
        Duration::from_secs(60),
        ProductionEngine::from_config(config),
    )
    .await
    .unwrap_or_else(|_| {
        Err(tonic::Status::deadline_exceeded(
            "integration startup check timed out",
        ))
    });
    let engine = Arc::new(startup.map_err(|error| {
        prover_script::logging::log(serde_json::json!({
            "component":"proof-api", "event":"integration_startup_failed",
            "chain_id":client.chain_id, "client_id":client.client_id,
            "error":error.message(), "code":error.code().to_string(),
        }));
        error
    })?);
    let listener = TcpListener::bind(server.listen).await?;
    let ready = Arc::new(AtomicBool::new(true));
    let service = Service::new(
        engine.clone(),
        proto::Client {
            chain_id: client.chain_id.clone(),
            client_id: client.client_id.clone(),
        },
    )?;
    let monitor_engine = engine.clone();
    let monitor_ready = ready.clone();
    let monitor = tokio::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_secs(15));
        interval.tick().await;
        loop {
            interval.tick().await;
            let check = timeout(Duration::from_secs(30), monitor_engine.check_integration())
                .await
                .unwrap_or_else(|_| {
                    Err(tonic::Status::deadline_exceeded(
                        "readiness integration check timed out",
                    ))
                });
            let healthy = check.is_ok();
            let before = monitor_ready.swap(healthy, Ordering::AcqRel);
            if before != healthy || !healthy {
                prover_script::logging::log(serde_json::json!({
                    "component":"readiness", "ready":healthy,
                    "chain_id":client.chain_id, "client_id":client.client_id,
                    "error":check.as_ref().err().map(tonic::Status::message),
                }));
            }
        }
    });
    let (stop, stopping) = oneshot::channel();
    let serving = axum::serve(listener, transport::router(service, ready.clone()))
        .with_graceful_shutdown(async {
            let _ = stopping.await;
        })
        .into_future();
    tokio::pin!(serving);
    prover_script::logging::log(
        serde_json::json!({"component":"proof-api", "listen":server.listen.to_string(), "result":"ready"}),
    );
    let result = tokio::select! {
        result = &mut serving => result.map_err(Into::into),
        () = shutdown_signal() => {
            ready.store(false, Ordering::Release);
            monitor.abort();
            let _ = stop.send(());
            match timeout(Duration::from_secs(server.shutdown_timeout_seconds), &mut serving).await {
                Ok(result) => result.map_err(Into::into),
                Err(_) => Ok(()),
            }
        }
    };
    ready.store(false, Ordering::Release);
    monitor.abort();
    engine.stop();
    engine.stopped().await;
    result
}
async fn shutdown_signal() {
    #[cfg(unix)]
    {
        use tokio::signal::unix::{signal, SignalKind};
        if let Ok(mut terminate) = signal(SignalKind::terminate()) {
            tokio::select! { _ = tokio::signal::ctrl_c() => {}, _ = terminate.recv() => {} }
            return;
        }
    }
    let _ = tokio::signal::ctrl_c().await;
}
