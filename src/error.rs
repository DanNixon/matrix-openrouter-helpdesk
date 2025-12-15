use miette::Diagnostic;
use thiserror::Error;

#[derive(Error, Debug, Diagnostic)]
pub enum HelpdeskError {
    #[error("No response from OpenRouter")]
    #[diagnostic(
        code(helpdesk::no_response),
        help("Check if the OpenRouter API is working correctly")
    )]
    NoResponse,

    #[error("Configuration error: {0}")]
    #[diagnostic(code(helpdesk::config))]
    Config(String),

    #[error("Template rendering error: {0}")]
    #[diagnostic(code(helpdesk::template))]
    Template(String),

    #[error("Failed to get user_id after login")]
    #[diagnostic(
        code(helpdesk::no_user_id),
        help("Ensure the Matrix credentials are correct")
    )]
    NoUserId,
}

pub type Result<T> = miette::Result<T>;
