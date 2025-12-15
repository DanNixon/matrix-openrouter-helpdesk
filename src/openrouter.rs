use crate::error::{HelpdeskError, Result};
use miette::IntoDiagnostic;
use serde::{Deserialize, Serialize};

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

pub async fn call_openrouter(
    http_client: &reqwest::Client,
    api_key: &str,
    model: &str,
    prompt: &str,
) -> Result<String> {
    let request = OpenRouterRequest {
        model: model.to_string(),
        messages: vec![Message {
            role: "user".to_string(),
            content: prompt.to_string(),
        }],
    };

    let response = http_client
        .post("https://openrouter.ai/api/v1/chat/completions")
        .header("Authorization", format!("Bearer {}", api_key))
        .header("Content-Type", "application/json")
        .json(&request)
        .send()
        .await
        .into_diagnostic()?
        .error_for_status()
        .into_diagnostic()?;

    let response_data: OpenRouterResponse = response.json().await.into_diagnostic()?;

    if let Some(choice) = response_data.choices.first() {
        Ok(choice.message.content.clone())
    } else {
        Err(HelpdeskError::NoResponse.into())
    }
}
