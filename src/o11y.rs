use metrics::{counter, describe_counter};
use metrics_exporter_prometheus::PrometheusBuilder;
use miette::{Context, IntoDiagnostic};
use std::net::SocketAddr;
use tracing::info;

const HELP_DESK_REQUESTS_TOTAL: &str = "helpdesk_requests_total";

pub(super) fn init(address: SocketAddr) -> miette::Result<()> {
    PrometheusBuilder::new()
        .with_http_listener(address)
        .install()
        .into_diagnostic()
        .wrap_err("Failed to start prometheus metrics exporter")?;
    info!("Metrics server listening on {}", address);

    describe_counter!(
        HELP_DESK_REQUESTS_TOTAL,
        "Total number of helpdesk requests, labeled by matrix_user, matrix_room, and result."
    );

    Ok(())
}

pub(crate) fn record_request(user_id: &str, room_id: &str, result: &str) {
    counter!(
        HELP_DESK_REQUESTS_TOTAL,
        "matrix_user" => user_id.to_string(),
        "matrix_room" => room_id.to_string(),
        "result" => result.to_string()
    )
    .increment(1);
}
