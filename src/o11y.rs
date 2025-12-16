use metrics::counter;
use metrics_exporter_prometheus::PrometheusBuilder;
use miette::{Context, IntoDiagnostic};
use std::net::SocketAddr;
use tracing::info;

pub(super) fn init(address: SocketAddr) -> miette::Result<()> {
    PrometheusBuilder::new()
        .with_http_listener(address)
        .install()
        .into_diagnostic()
        .wrap_err("Failed to start prometheus metrics exporter")?;

    info!("Metrics server listening on {}", address);

    Ok(())
}

pub(crate) fn record_request(user_id: &str, room_id: &str, result: &str) {
    counter!(
        "helpdesk_requests_total",
        "matrix_user" => user_id.to_string(),
        "matrix_room" => room_id.to_string(),
        "result" => result.to_string()
    )
    .increment(1);
}
