use clap::Parser;
use miette::IntoDiagnostic;
use std::{fs, net::SocketAddr, path::PathBuf};

#[derive(Parser, Clone)]
#[command(author, version, about, long_about = None)]
pub struct Config {
    /// Matrix homeserver URL
    #[arg(env = "MATRIX_HOMESERVER_URL", long)]
    pub matrix_homeserver_url: String,

    /// Path to store Matrix session data
    #[arg(env = "MATRIX_SESSION_PATH", long, default_value = "./matrix_session")]
    pub matrix_session_path: PathBuf,

    /// Matrix username
    #[arg(env = "MATRIX_USERNAME", long)]
    pub matrix_username: String,

    /// Matrix password
    #[arg(env = "MATRIX_PASSWORD", long)]
    pub matrix_password: String,

    /// OpenRouter API key
    #[arg(env = "OPENROUTER_API_KEY", long)]
    pub openrouter_api_key: String,

    /// OpenRouter model to use
    #[arg(env = "OPENROUTER_MODEL", long, default_value = "openai/gpt-5-mini")]
    pub model: String,

    /// System prompt for configured model
    #[arg(
        env = "SYSTEM_PROMPT",
        long,
        default_value = "You are answering single questions from a user. Don't offer any follow up options."
    )]
    pub system_prompt: String,

    /// Endpoint for Prometheus metrics
    #[arg(env = "METRICS_ENDPOINT", long, default_value = "127.0.0.1:9090")]
    pub metrics_endpoint: SocketAddr,

    /// Path to reply template file
    #[arg(env = "REPLY_TEMPLATE_FILE", long)]
    reply_template_file: Option<String>,

    #[clap(skip)]
    pub reply_template: String,
}

const DEFAULT_REPLY_TEMPLATE: &str = "{{ response }}";

impl Config {
    pub fn from_env() -> miette::Result<Self> {
        let mut config = Self::parse();

        config.reply_template = match &config.reply_template_file {
            Some(path) => fs::read_to_string(path).into_diagnostic()?,
            None => DEFAULT_REPLY_TEMPLATE.to_string(),
        };

        Ok(config)
    }
}
