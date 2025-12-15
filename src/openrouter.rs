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
) -> miette::Result<String> {
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
        Err(miette::miette!("No response"))
    }
}

// curl -X POST https://openrouter.ai/api/v1/responses \
//      -H "Authorization: Bearer sk-or-v1-xxx" \
//      -H "Content-Type: application/json" \
//      -d '{
//   "input": [
//     {
//       "role": "system",
//       "content": "You are answering single questions about a hackspace named Maker Space. You must only use information from www.makerspace.org.uk and wiki.makerspace.org.uk. Do not give the user any follow up options, just present what you find and cease communication. If you cannot find any relevant information, suggest the user asks other members and updates the wiki when they find their answer."
//     },
//     {
//       "role": "user",
//       "content": "when is the space opne?"
//     }
//   ],
//   "model": "openai/gpt-5-mini:online"
// }'
