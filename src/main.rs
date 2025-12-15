mod config;
mod error;
mod matrix_handlers;
mod metrics;
mod openrouter;
mod session;
mod templates;

use crate::config::Config;
use crate::error::{HelpdeskError, Result};
use crate::matrix_handlers::{on_room_message, on_stripped_state_member};
use crate::session::{get_sync_settings, restore_or_create_session};
use crate::templates::TemplateRenderer;
use matrix_sdk::{
    room::Room,
    ruma::events::room::{member::StrippedRoomMemberEvent, message::OriginalSyncRoomMessageEvent},
    Client,
};
use metrics_exporter_prometheus::PrometheusBuilder;
use miette::IntoDiagnostic;
use tracing::info;

#[tokio::main]
async fn main() -> Result<()> {
    miette::set_panic_hook();
    tracing_subscriber::fmt::init();

    // Load configuration
    let config = Config::from_env()?;

    // Set up Prometheus metrics exporter
    let _prometheus_handle = PrometheusBuilder::new()
        .with_http_listener(([0, 0, 0, 0], config.metrics_port))
        .install_recorder()
        .into_diagnostic()?;

    info!("Metrics server listening on port {}", config.metrics_port);

    // Restore or create Matrix session
    let client = restore_or_create_session(&config).await?;

    let bot_user_id = client
        .user_id()
        .ok_or(HelpdeskError::NoUserId)?
        .to_owned();
    info!("Logged in as {}", bot_user_id);

    let http_client = reqwest::Client::new();

    // Set up auto-join for room invites
    client.add_event_handler(
        move |room_member: StrippedRoomMemberEvent, client: Client, room: Room| async move {
            on_stripped_state_member(room_member, client, room).await;
        },
    );

    // Set up message handler
    client.add_event_handler(
        move |event: OriginalSyncRoomMessageEvent, room: Room| {
            let bot_user_id = bot_user_id.clone();
            let http_client = http_client.clone();
            let config = config.clone();
            let template_renderer = TemplateRenderer::new();
            async move {
                on_room_message(event, room, bot_user_id, http_client, config, template_renderer)
                    .await;
            }
        },
    );

    info!("Starting sync...");
    client.sync(get_sync_settings()).await.into_diagnostic()?;

    Ok(())
}
