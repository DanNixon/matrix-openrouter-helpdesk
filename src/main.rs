use anyhow::Result;
use matrix_sdk::{
    config::SyncSettings,
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
use serde::{Deserialize, Serialize};
use std::env;
use tracing::{error, info};

#[derive(Debug, Serialize)]
struct OpenRouterRequest {
    model: String,
    messages: Vec<Message>,
}

#[derive(Debug, Serialize, Deserialize)]
struct Message {
    role: String,
    content: String,
}

#[derive(Debug, Deserialize)]
struct OpenRouterResponse {
    choices: Vec<Choice>,
}

#[derive(Debug, Deserialize)]
struct Choice {
    message: Message,
}

async fn call_openrouter(api_key: &str, model: &str, prompt: &str) -> Result<String> {
    let client = reqwest::Client::new();
    let request = OpenRouterRequest {
        model: model.to_string(),
        messages: vec![Message {
            role: "user".to_string(),
            content: prompt.to_string(),
        }],
    };

    let response = client
        .post("https://openrouter.ai/api/v1/chat/completions")
        .header("Authorization", format!("Bearer {}", api_key))
        .header("Content-Type", "application/json")
        .json(&request)
        .send()
        .await?;

    let response_data: OpenRouterResponse = response.json().await?;
    
    if let Some(choice) = response_data.choices.first() {
        Ok(choice.message.content.clone())
    } else {
        Err(anyhow::anyhow!("No response from OpenRouter"))
    }
}

async fn on_room_message(
    event: OriginalSyncRoomMessageEvent,
    room: Room,
    bot_user_id: OwnedUserId,
    openrouter_api_key: String,
    openrouter_model: String,
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

    // Check if the message mentions the bot
    if !message_body.starts_with(&bot_mention) {
        return;
    }

    // Extract the question after the bot mention
    let question = message_body
        .strip_prefix(&bot_mention)
        .unwrap_or("")
        .trim();

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

    // Call OpenRouter to get a response
    match call_openrouter(&openrouter_api_key, &openrouter_model, question).await {
        Ok(answer) => {
            // Send the response as a reply
            let mut content = RoomMessageEventContent::text_plain(answer);
            content.relates_to = Some(Relation::Reply {
                in_reply_to: InReplyTo::new(event.event_id.clone()),
            });

            if let Err(e) = room.send(content).await {
                error!("Failed to send message: {}", e);
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
        }
    }
}

async fn on_stripped_state_member(
    room_member: StrippedRoomMemberEvent,
    client: Client,
    room: Room,
) {
    if room_member.state_key != client.user_id().unwrap() {
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

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    // Read configuration from environment variables
    let homeserver_url = env::var("MATRIX_HOMESERVER_URL")
        .expect("MATRIX_HOMESERVER_URL environment variable not set");
    let username =
        env::var("MATRIX_USERNAME").expect("MATRIX_USERNAME environment variable not set");
    let password =
        env::var("MATRIX_PASSWORD").expect("MATRIX_PASSWORD environment variable not set");
    let openrouter_api_key = env::var("OPENROUTER_API_KEY")
        .expect("OPENROUTER_API_KEY environment variable not set");
    let openrouter_model = env::var("OPENROUTER_MODEL")
        .unwrap_or_else(|_| "openai/gpt-3.5-turbo".to_string());

    info!("Logging in to {}", homeserver_url);

    let client = Client::builder()
        .homeserver_url(&homeserver_url)
        .build()
        .await?;

    client
        .matrix_auth()
        .login_username(&username, &password)
        .initial_device_display_name("OpenRouter Helpdesk Bot")
        .await?;

    let bot_user_id = client.user_id().unwrap().to_owned();
    info!("Logged in as {}", bot_user_id);

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
            let api_key = openrouter_api_key.clone();
            let model = openrouter_model.clone();
            async move {
                on_room_message(event, room, bot_user_id, api_key, model).await;
            }
        },
    );

    info!("Starting sync...");
    client.sync(SyncSettings::default()).await?;

    Ok(())
}
