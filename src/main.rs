mod context;
mod o11y;
mod openrouter;

use crate::context::Context;
use matrix_sdk::{
    authentication::matrix::MatrixSession,
    config::SyncSettings,
    ruma::{
        api::client::{filter::FilterDefinition, profile::DisplayName},
        events::{
            reaction::ReactionEventContent,
            relation::{Annotation, InReplyTo},
            room::{
                member::StrippedRoomMemberEvent,
                message::{
                    MessageType, OriginalSyncRoomMessageEvent, Relation, RoomMessageEventContent,
                },
            },
        },
        UserId,
    },
    Client, Error, LoopCtrl, Room, RoomState,
};
use miette::IntoDiagnostic;
use rand::{distr::Alphanumeric, Rng};
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use tokio::fs;
use tracing::{debug, error, info, warn};

#[tokio::main]
async fn main() -> miette::Result<()> {
    miette::set_panic_hook();
    tracing_subscriber::fmt::init();

    // Load configuration
    let ctx = Context::from_cli()?;

    // Set up Prometheus metrics exporter
    o11y::init(ctx.args.metrics_endpoint)?;

    let session_file = ctx.matrix_session_file();

    let (client, sync_token) = if session_file.exists() {
        restore_session(&session_file).await?
    } else {
        (login(&ctx).await?, None)
    };

    sync(client, ctx, sync_token).await
}

/// The data needed to re-build a client.
#[derive(Debug, Serialize, Deserialize)]
struct ClientSession {
    /// The URL of the homeserver of the user.
    homeserver: String,

    /// The path of the database.
    db_path: PathBuf,

    /// The passphrase of the database.
    passphrase: String,
}

/// The full session to persist.
#[derive(Debug, Serialize, Deserialize)]
struct FullSession {
    /// The data to re-build the client.
    client_session: ClientSession,

    /// The Matrix user session.
    user_session: MatrixSession,

    /// The latest sync token.
    ///
    /// It is only needed to persist it when using `Client::sync_once()` and we
    /// want to make our syncs faster by not receiving all the initial sync
    /// again.
    #[serde(skip_serializing_if = "Option::is_none")]
    sync_token: Option<String>,
}

/// Restore a previous session.
async fn restore_session(session_file: &Path) -> miette::Result<(Client, Option<String>)> {
    info!(
        "Previous session found in '{}'",
        session_file.to_string_lossy()
    );

    // The session was serialized as JSON in a file.
    let serialized_session = fs::read_to_string(session_file).await.into_diagnostic()?;
    let FullSession {
        client_session,
        user_session,
        sync_token,
    } = serde_json::from_str(&serialized_session).into_diagnostic()?;

    // Build the client with the previous settings from the session.
    let client = Client::builder()
        .homeserver_url(client_session.homeserver)
        .sqlite_store(client_session.db_path, Some(&client_session.passphrase))
        .build()
        .await
        .into_diagnostic()?;

    info!("Restoring session for {}…", user_session.meta.user_id);

    // Restore the Matrix user session.
    client
        .restore_session(user_session)
        .await
        .into_diagnostic()?;

    Ok((client, sync_token))
}

/// Login with a new device.
async fn login(ctx: &Context) -> miette::Result<Client> {
    info!("No previous session found, logging in…");

    let (client, client_session) = build_client(ctx).await?;
    let matrix_auth = client.matrix_auth();

    matrix_auth
        .login_username(&ctx.args.matrix_username, &ctx.args.matrix_password)
        .initial_device_display_name("matrix-openrouter-helpdesk")
        .await
        .into_diagnostic()?;
    info!("Logged in as {}", ctx.args.matrix_username);

    let session_file = ctx.matrix_session_file();

    // Persist the session to reuse it later.
    // This is not very secure, for simplicity. If the system provides a way of
    // storing secrets securely, it should be used instead.
    // Note that we could also build the user session from the login response.
    let user_session = matrix_auth
        .session()
        .expect("A logged-in client should have a session");
    let serialized_session = serde_json::to_string(&FullSession {
        client_session,
        user_session,
        sync_token: None,
    })
    .into_diagnostic()?;
    fs::write(&session_file, serialized_session)
        .await
        .into_diagnostic()?;

    info!("Session persisted in {}", session_file.to_string_lossy());

    // After logging in, you might want to verify this session with another one (see
    // the `emoji_verification` example), or bootstrap cross-signing if this is your
    // first session with encryption, or if you need to reset cross-signing because
    // you don't have access to your old sessions (see the
    // `cross_signing_bootstrap` example).

    Ok(client)
}

/// Build a new client.
async fn build_client(ctx: &Context) -> miette::Result<(Client, ClientSession)> {
    let mut rng = rand::rng();

    // Generating a subfolder for the database is not mandatory, but it is useful if
    // you allow several clients to run at the same time. Each one must have a
    // separate database, which is a different folder with the SQLite store.
    let db_subfolder: String = (&mut rng)
        .sample_iter(Alphanumeric)
        .take(7)
        .map(char::from)
        .collect();
    let db_path = ctx.args.matrix_session_path.join(db_subfolder);

    // Generate a random passphrase.
    let passphrase: String = (&mut rng)
        .sample_iter(Alphanumeric)
        .take(32)
        .map(char::from)
        .collect();

    let homeserver = ctx.args.matrix_homeserver_url.clone();

    let client = Client::builder()
        .homeserver_url(&homeserver)
        .sqlite_store(&db_path, Some(&passphrase))
        .build()
        .await
        .into_diagnostic()?;

    Ok((
        client,
        ClientSession {
            homeserver,
            db_path,
            passphrase,
        },
    ))
}

/// Setup the client to listen to new messages.
async fn sync(
    client: Client,
    ctx: Context,
    initial_sync_token: Option<String>,
) -> miette::Result<()> {
    info!("Launching a first sync to ignore past messages…");

    let session_file = &ctx.matrix_session_file();

    // Enable room members lazy-loading, it will speed up the initial sync a lot
    // with accounts in lots of rooms.
    // See <https://spec.matrix.org/v1.6/client-server-api/#lazy-loading-room-members>.
    let filter = FilterDefinition::with_lazy_loading();

    let mut sync_settings = SyncSettings::default().filter(filter.into());

    // We restore the sync where we left.
    // This is not necessary when not using `sync_once`. The other sync methods get
    // the sync token from the store.
    if let Some(sync_token) = initial_sync_token {
        sync_settings = sync_settings.token(sync_token);
    }

    // Let's ignore messages before the program was launched.
    // This is a loop in case the initial sync is longer than our timeout. The
    // server should cache the response and it will ultimately take less time to
    // receive.
    loop {
        match client.sync_once(sync_settings.clone()).await {
            Ok(response) => {
                // This is the last time we need to provide this token, the sync method after
                // will handle it on its own.
                sync_settings = sync_settings.token(response.next_batch.clone());
                persist_sync_token(session_file, response.next_batch).await?;
                break;
            }
            Err(error) => {
                warn!("An error occurred during initial sync: {error}");
                warn!("Trying again…");
            }
        }
    }

    info!("The client is ready! Listening to new messages…");

    let bot_id = client
        .user_id()
        .ok_or_else(|| miette::miette!("No bot ID for client"))?
        .to_owned();
    info!("My user ID is {bot_id:?}");

    // Listen for query messages
    client.add_event_handler(
        move |event: OriginalSyncRoomMessageEvent, client: Client, room: Room| {
            let ctx = ctx.clone();
            let bot_id = bot_id.clone();
            async move {
                on_room_message(event, room, client, ctx, &bot_id).await;
            }
        },
    );

    // Set up auto-join for room invites
    client.add_event_handler(
        move |room_member: StrippedRoomMemberEvent, client: Client, room: Room| async move {
            auto_join_room(room_member, client, room).await;
        },
    );

    // This loops until we kill the program or an error happens.
    client
        .sync_with_result_callback(sync_settings, |sync_result| async move {
            let response = sync_result?;

            // We persist the token each time to be able to restore our session
            persist_sync_token(session_file, response.next_batch)
                .await
                .map_err(|err| Error::UnknownError(err.into()))?;

            Ok(LoopCtrl::Continue)
        })
        .await
        .into_diagnostic()?;

    Ok(())
}

/// Persist the sync token for a future session.
/// Note that this is needed only when using `sync_once`. Other sync methods get
/// the sync token from the store.
async fn persist_sync_token(session_file: &Path, sync_token: String) -> miette::Result<()> {
    let serialized_session = fs::read_to_string(session_file).await.into_diagnostic()?;
    let mut full_session: FullSession =
        serde_json::from_str(&serialized_session).into_diagnostic()?;

    full_session.sync_token = Some(sync_token);
    let serialized_session = serde_json::to_string(&full_session).into_diagnostic()?;
    fs::write(session_file, serialized_session)
        .await
        .into_diagnostic()?;

    Ok(())
}

async fn auto_join_room(room_member: StrippedRoomMemberEvent, client: Client, room: Room) {
    let Some(user_id) = client.user_id() else {
        error!("Client user_id is not available");
        return;
    };

    if room_member.state_key != user_id {
        return;
    }

    if room.state() != RoomState::Invited {
        return;
    }

    info!("Autojoining room {}", room.room_id());

    if let Err(e) = room.join().await {
        error!("Failed to join room: {}", e);
    } else {
        info!("Successfully joined room {}", room.room_id());
    }
}

async fn on_room_message(
    event: OriginalSyncRoomMessageEvent,
    room: Room,
    client: Client,
    ctx: Context,
    bot_id: &UserId,
) {
    if event.sender == bot_id {
        return;
    }

    if room.state() != RoomState::Joined {
        return;
    }

    let MessageType::Text(text_content) = &event.content.msgtype else {
        return;
    };

    let message_body = &text_content.body.trim();
    let mention_username = format!("@{}", bot_id.localpart());

    let question = {
        let mut question = None;

        if message_body.contains(&mention_username) {
            let re = Regex::new(&format!(
                "{mention_username}\\s*:\\s*(.+)|{mention_username}\\s+(.+)"
            ))
            .unwrap();
            question = re
                .captures(&message_body)
                .map(|c| c.get(1).or(c.get(2)).unwrap().as_str());
        }

        if let Some(mentions) = event.content.mentions {
            if mentions.user_ids.contains(bot_id) {
                let request = matrix_sdk::ruma::api::client::profile::get_profile::v3::Request::new(
                    bot_id.to_owned(),
                );

                if let Ok(resp) = client.send(request).await {
                    let bot_display_name = resp
                        .get_static::<DisplayName>()
                        .ok()
                        .flatten()
                        .unwrap_or(mention_username);

                    let re = Regex::new(&format!(
                        "{bot_display_name}\\s*:\\s*(.+)|{bot_display_name}\\s+(.+)"
                    ))
                    .unwrap();
                    question = re
                        .captures(&message_body)
                        .map(|c| c.get(1).or(c.get(2)).unwrap().as_str());
                }
            }
        }

        question
    };

    if question.is_none() {
        debug!("did not match question format: {message_body}");
        return;
    }
    let question = question.unwrap();

    info!("Processing question: {}", question);

    // Add "eyes" emoji reaction to indicate we're processing
    let reaction =
        ReactionEventContent::new(Annotation::new(event.event_id.clone(), "👀".to_string()));
    if let Err(e) = room.send(reaction).await {
        error!("Failed to add reaction: {}", e);
    }

    // Get room and user info for metrics
    let room_id = room.room_id().to_string();
    let user_id = event.sender.to_string();

    // Call OpenRouter to get a response
    match crate::openrouter::request(
        &ctx.http_client,
        &ctx.args.openrouter_api_key,
        &ctx.args.model,
        &ctx.args.system_prompt,
        question,
    )
    .await
    {
        Ok(answer) => {
            // Render the reply template
            let rendered_reply = match ctx.render_reply(&answer) {
                Ok(r) => r,
                Err(e) => {
                    error!("Failed to render reply template: {}", e);
                    crate::o11y::record_request(&user_id, &room_id, "failure");
                    return;
                }
            };

            // Send the response as a reply
            let mut content = RoomMessageEventContent::text_markdown(rendered_reply);
            content.relates_to = Some(Relation::Reply {
                in_reply_to: InReplyTo::new(event.event_id.clone()),
            });

            if let Err(e) = room.send(content).await {
                error!("Failed to send message: {}", e);
                crate::o11y::record_request(&user_id, &room_id, "failure");
            } else {
                crate::o11y::record_request(&user_id, &room_id, "success");
            }
        }
        Err(e) => {
            error!("Failed to get response from OpenRouter: {}", e);
            let mut error_content = RoomMessageEventContent::text_plain(
                "Sorry, I encountered an error while processing your question.",
            );
            error_content.relates_to = Some(Relation::Reply {
                in_reply_to: InReplyTo::new(event.event_id.clone()),
            });

            if let Err(e) = room.send(error_content).await {
                error!("Failed to send error message: {}", e);
            }

            crate::o11y::record_request(&user_id, &room_id, "failure");
        }
    }
}
