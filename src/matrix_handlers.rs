use crate::config::Config;
use crate::metrics::record_request_metric;
use crate::openrouter::call_openrouter;
use crate::templates::TemplateRenderer;
use matrix_sdk::{
    room::Room,
    ruma::{
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
        OwnedUserId,
    },
    Client, RoomState,
};
use tracing::{error, info};

pub async fn on_room_message(
    event: OriginalSyncRoomMessageEvent,
    room: Room,
    bot_user_id: OwnedUserId,
    http_client: reqwest::Client,
    config: Config,
    template_renderer: TemplateRenderer,
) {
    // Ignore messages from the bot itself
    if event.sender == bot_user_id {
        return;
    }

    if room.state() != RoomState::Joined {
        return;
    }

    let MessageType::Text(text_content) = &event.content.msgtype else {
        return;
    };

    let bot_mention = format!("@{}", bot_user_id.localpart());
    let message_body = &text_content.body;

    // Check if the message mentions the bot (case insensitive)
    let has_mention = message_body
        .get(..bot_mention.len())
        .is_some_and(|prefix| prefix.eq_ignore_ascii_case(&bot_mention));

    if !has_mention {
        return;
    }

    // Extract the question after the bot mention (safe because we verified the prefix exists)
    let question = message_body.get(bot_mention.len()..).unwrap_or("").trim();

    if question.is_empty() {
        return;
    }

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

    // Render the question template
    let rendered_question =
        match template_renderer.render_question(&config.question_template, question) {
            Ok(q) => q,
            Err(e) => {
                error!("Failed to render question template: {}", e);
                record_request_metric(&user_id, &room_id, "failure");
                return;
            }
        };

    // Call OpenRouter to get a response
    match call_openrouter(
        &http_client,
        &config.openrouter_api_key,
        &config.model,
        &rendered_question,
    )
    .await
    {
        Ok(answer) => {
            // Render the reply template
            let rendered_reply =
                match template_renderer.render_reply(&config.reply_template, &answer) {
                    Ok(r) => r,
                    Err(e) => {
                        error!("Failed to render reply template: {}", e);
                        record_request_metric(&user_id, &room_id, "failure");
                        return;
                    }
                };

            // Send the response as a reply
            let mut content = RoomMessageEventContent::text_plain(rendered_reply);
            content.relates_to = Some(Relation::Reply {
                in_reply_to: InReplyTo::new(event.event_id.clone()),
            });

            if let Err(e) = room.send(content).await {
                error!("Failed to send message: {}", e);
                record_request_metric(&user_id, &room_id, "failure");
            } else {
                record_request_metric(&user_id, &room_id, "success");
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

            record_request_metric(&user_id, &room_id, "failure");
        }
    }
}

pub async fn on_stripped_state_member(
    room_member: StrippedRoomMemberEvent,
    client: Client,
    room: Room,
) {
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
