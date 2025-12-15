use crate::error::{HelpdeskError, Result};
use clap::Parser;
use miette::IntoDiagnostic;
use std::fs;
use std::path::PathBuf;

const DEFAULT_QUESTION_TEMPLATE: &str = "{{ query }}";
const DEFAULT_REPLY_TEMPLATE: &str = "{{ response }}";

#[derive(Parser, Clone)]
#[command(author, version, about, long_about = None)]
pub struct Config {
    /// Matrix homeserver URL
    #[arg(env = "MATRIX_HOMESERVER_URL", long)]
    pub matrix_homeserver_url: String,

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
    #[arg(env = "OPENROUTER_MODEL", long, default_value = "openai/gpt-3.5-turbo")]
    pub openrouter_model: String,

    /// Port for Prometheus metrics endpoint
    #[arg(env = "METRICS_PORT", long, default_value = "9090")]
    pub metrics_port: u16,

    /// Path to store session data
    #[arg(env = "SESSION_PATH", long, default_value = "./session")]
    pub session_path: PathBuf,

    /// Path to question template file
    #[arg(env = "QUESTION_TEMPLATE_FILE", long)]
    question_template_file: Option<String>,

    /// Path to reply template file
    #[arg(env = "REPLY_TEMPLATE_FILE", long)]
    reply_template_file: Option<String>,

    #[clap(skip)]
    pub question_template: String,

    #[clap(skip)]
    pub reply_template: String,
}

impl Config {
    pub fn from_env() -> Result<Self> {
        let mut config = Self::parse();

        // Load templates from files if specified, otherwise use default templates
        config.question_template = match &config.question_template_file {
            Some(path) => fs::read_to_string(path)
                .into_diagnostic()
                .map_err(|_| {
                    HelpdeskError::Config(format!("Failed to read question template file: {}", path))
                })?,
            None => DEFAULT_QUESTION_TEMPLATE.to_string(),
        };

        config.reply_template = match &config.reply_template_file {
            Some(path) => fs::read_to_string(path)
                .into_diagnostic()
                .map_err(|_| {
                    HelpdeskError::Config(format!("Failed to read reply template file: {}", path))
                })?,
            None => DEFAULT_REPLY_TEMPLATE.to_string(),
        };

        Ok(config)
    }
}
