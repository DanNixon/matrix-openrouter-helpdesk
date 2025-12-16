mod context;
mod matrix_handlers;
mod o11y;
mod openrouter;
mod session;

use crate::context::Context;
use matrix_sdk::config::SyncSettings;
use matrix_sdk::ruma::api::client::filter::FilterDefinition;
use matrix_sdk::{
    room::Room,
    ruma::events::room::{member::StrippedRoomMemberEvent, message::OriginalSyncRoomMessageEvent},
    Client,
};
use miette::IntoDiagnostic;
use tracing::info;

#[tokio::main]
async fn main() -> miette::Result<()> {
    miette::set_panic_hook();
    tracing_subscriber::fmt::init();

    // Load configuration
    let ctx = Context::from_cli()?;

    // Set up Prometheus metrics exporter
    o11y::init(ctx.args.metrics_endpoint)?;

    // Restore or create Matrix session
    let client = session::restore_or_create_session(&ctx.args).await?;

    let bot_user_id = client
        .user_id()
        .ok_or(miette::miette!("No user ID"))?
        .to_owned();
    info!("Logged in as {}", bot_user_id);

    // Set up auto-join for room invites
    client.add_event_handler(
        move |room_member: StrippedRoomMemberEvent, client: Client, room: Room| async move {
            matrix_handlers::auto_join_room(room_member, client, room).await;
        },
    );

    // Set up message handler
    client.add_event_handler(move |event: OriginalSyncRoomMessageEvent, room: Room| {
        let ctx = ctx.clone();
        let bot_user_id = bot_user_id.clone();
        async move {
            matrix_handlers::on_room_message(event, room, ctx, bot_user_id).await;
        }
    });

    info!("Starting sync...");
    let filter = FilterDefinition::with_lazy_loading();
    let sync_settings = SyncSettings::default().filter(filter.into());
    client.sync(sync_settings).await.into_diagnostic()?;

    Ok(())
}
