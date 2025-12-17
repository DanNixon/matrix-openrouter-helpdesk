use itertools::Itertools;
use miette::IntoDiagnostic;
use serde::Deserialize;
use tracing::info;

use crate::context::Context;

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
#[serde(tag = "type", rename_all = "lowercase")]
enum Output {
    #[allow(dead_code)]
    Reasoning(serde_json::Value),
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

pub(crate) async fn request(ctx: &Context, question: &str) -> miette::Result<String> {
    let request: serde_json::Value =
        serde_json::from_str(&ctx.render_request(question)?).into_diagnostic()?;
    info!("Request: {request:?}");

    let response = ctx
        .http_client
        .post("https://openrouter.ai/api/v1/responses")
        .header(
            "Authorization",
            format!("Bearer {}", ctx.args.openrouter_api_key),
        )
        .header("Content-Type", "application/json")
        .json(&request)
        .send()
        .await
        .into_diagnostic()?
        .error_for_status()
        .into_diagnostic()?;
    info!("Response meta: {response:?}");

    let response: Response = response.json().await.into_diagnostic()?;
    info!("Response: {response:?}");

    response.result_text()
}
