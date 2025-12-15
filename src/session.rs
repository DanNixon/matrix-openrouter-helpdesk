use crate::config::Config;
use matrix_sdk::{config::SyncSettings, Client};
use miette::IntoDiagnostic;
use rand::{distr::Alphanumeric, Rng};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use tokio::fs;
use tracing::info;

/// The full session to persist.
#[derive(Debug, Serialize, Deserialize)]
struct FullSession {
    /// The URL of the homeserver of the user.
    homeserver: String,

    /// The path of the database.
    db_path: PathBuf,

    /// The passphrase of the database.
    passphrase: String,
}

/// Restore a previous session or create a new one.
pub async fn restore_or_create_session(config: &Config) -> miette::Result<Client> {
    let session_file = config.session_path.join("session.json");
    let db_path = config.session_path.join("db");

    // Create session directory if it doesn't exist
    fs::create_dir_all(&config.session_path)
        .await
        .into_diagnostic()?;

    if session_file.exists() {
        info!("Previous session found, restoring...");
        restore_session(&session_file).await
    } else {
        info!("No previous session found, creating new session...");
        login(config, &db_path, &session_file).await
    }
}

/// Restore a previous session.
async fn restore_session(session_file: &Path) -> miette::Result<Client> {
    let serialized_session = fs::read_to_string(session_file).await.into_diagnostic()?;
    let session_data: FullSession = serde_json::from_str(&serialized_session).into_diagnostic()?;

    // Build the client with the previous settings from the session.
    // The SQLite store will automatically restore the session data
    let client = Client::builder()
        .homeserver_url(&session_data.homeserver)
        .sqlite_store(&session_data.db_path, Some(&session_data.passphrase))
        .build()
        .await
        .into_diagnostic()?;

    info!("Session restored from database");

    Ok(client)
}

/// Login with a new device and persist the session.
async fn login(config: &Config, db_path: &Path, session_file: &Path) -> miette::Result<Client> {
    let mut rng = rand::rng();

    // Generate a random passphrase for the database.
    let passphrase: String = (&mut rng)
        .sample_iter(Alphanumeric)
        .take(32)
        .map(char::from)
        .collect();

    // Build the client with SQLite store for encryption support
    let client = Client::builder()
        .homeserver_url(&config.matrix_homeserver_url)
        .sqlite_store(db_path, Some(&passphrase))
        .build()
        .await
        .into_diagnostic()?;

    // Login
    client
        .matrix_auth()
        .login_username(&config.matrix_username, &config.matrix_password)
        .initial_device_display_name("OpenRouter Helpdesk Bot")
        .await
        .into_diagnostic()?;

    // Persist the session metadata (the actual session data is in the SQLite store)
    let session_data = FullSession {
        homeserver: config.matrix_homeserver_url.clone(),
        db_path: db_path.to_path_buf(),
        passphrase,
    };

    let serialized_session = serde_json::to_string(&session_data).into_diagnostic()?;
    fs::write(session_file, serialized_session)
        .await
        .into_diagnostic()?;

    info!("Session persisted to {}", session_file.display());

    Ok(client)
}
