use crate::error::{HelpdeskError, Result};
use std::env;

const DEFAULT_OPENROUTER_MODEL: &str = "openai/gpt-3.5-turbo";
const DEFAULT_METRICS_PORT: u16 = 9090;
const DEFAULT_QUESTION_TEMPLATE: &str = "{{ query }}";
const DEFAULT_REPLY_TEMPLATE: &str = "{{ response }}";

#[derive(Clone)]
pub struct Config {
    pub matrix_homeserver_url: String,
    pub matrix_username: String,
    pub matrix_password: String,
    pub openrouter_api_key: String,
    pub openrouter_model: String,
    pub metrics_port: u16,
    pub question_template: String,
    pub reply_template: String,
}

impl Config {
    pub fn from_env() -> Result<Self> {
        let matrix_homeserver_url = env::var("MATRIX_HOMESERVER_URL")
            .map_err(|_| HelpdeskError::Config("MATRIX_HOMESERVER_URL not set".to_string()))?;
        
        let matrix_username = env::var("MATRIX_USERNAME")
            .map_err(|_| HelpdeskError::Config("MATRIX_USERNAME not set".to_string()))?;
        
        let matrix_password = env::var("MATRIX_PASSWORD")
            .map_err(|_| HelpdeskError::Config("MATRIX_PASSWORD not set".to_string()))?;
        
        let openrouter_api_key = env::var("OPENROUTER_API_KEY")
            .map_err(|_| HelpdeskError::Config("OPENROUTER_API_KEY not set".to_string()))?;
        
        let openrouter_model = env::var("OPENROUTER_MODEL")
            .unwrap_or_else(|_| DEFAULT_OPENROUTER_MODEL.to_string());
        
        let metrics_port = env::var("METRICS_PORT")
            .unwrap_or_else(|_| DEFAULT_METRICS_PORT.to_string())
            .parse::<u16>()
            .map_err(|_| HelpdeskError::Config("METRICS_PORT must be a valid port number".to_string()))?;
        
        let question_template = env::var("QUESTION_TEMPLATE")
            .unwrap_or_else(|_| DEFAULT_QUESTION_TEMPLATE.to_string());
        
        let reply_template = env::var("REPLY_TEMPLATE")
            .unwrap_or_else(|_| DEFAULT_REPLY_TEMPLATE.to_string());
        
        Ok(Config {
            matrix_homeserver_url,
            matrix_username,
            matrix_password,
            openrouter_api_key,
            openrouter_model,
            metrics_port,
            question_template,
            reply_template,
        })
    }
}
