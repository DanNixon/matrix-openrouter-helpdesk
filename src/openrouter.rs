use itertools::Itertools;
use miette::IntoDiagnostic;
use serde::Deserialize;
use tracing::debug;

#[derive(Debug, Deserialize)]
struct Response {
    output: Vec<Output>,
}

impl Response {
    fn result_text(self) -> miette::Result<String> {
        let message = self
            .output
            .iter()
            .find(|o| matches!(o, Output::Message(_)))
            .ok_or_else(|| miette::miette!("no message in result"))?;

        if let Output::Message(message) = message {
            let text = message.content.iter().map(|c| c.text.clone()).join("\n");
            Ok(text)
        } else {
            unreachable!();
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type")]
enum Output {
    Reasoning,
    Message(Message),
}

#[derive(Debug, Deserialize)]
struct Message {
    content: Vec<MessageContent>,
}

#[derive(Debug, Deserialize)]
struct MessageContent {
    text: String,
}

pub async fn request(
    http_client: &reqwest::Client,
    api_key: &str,
    model: &str,
    system_prompt: &str,
    prompt: &str,
) -> miette::Result<String> {
    let request = serde_json::json!({
        "input": [
            {
                "type": "message",
                "role": "system",
                "content": system_prompt,
            },
            {
                "type": "message",
                "role": "user",
                "content": prompt,
            },
        ],
        "model": model,
    });

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

    debug!("Response: {response:?}");

    let response: Response = response.json().await.into_diagnostic()?;

    response.result_text()
}
