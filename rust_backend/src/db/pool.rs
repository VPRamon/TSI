//! Database connection pool management.

use bb8::Pool;
use bb8_tiberius::ConnectionManager;
use once_cell::sync::OnceCell;
use serde::Deserialize;
use std::time::Duration;
use tiberius::Config;

use super::config::{DbAuthMethod, DbConfig};

/// Type alias for the database connection pool.
pub type DbPool = Pool<ConnectionManager>;

static DB_POOL: OnceCell<DbPool> = OnceCell::new();

#[derive(Debug, Deserialize)]
struct AadTokenResponse {
    access_token: String,
    #[allow(dead_code)]
    expires_in: Option<String>,
}

async fn fetch_aad_token(config: &DbConfig) -> Result<String, String> {
    let token_url = format!(
        "https://login.microsoftonline.com/{}/oauth2/token",
        config.tenant_id
    );

    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(20))
        .build()
        .map_err(|e| format!("Failed to build HTTP client: {}", e))?;

    let params = [
        ("grant_type", "password"),
        ("client_id", config.client_id.as_str()),
        ("username", config.username.as_str()),
        ("password", config.password.as_str()),
        ("resource", config.resource.as_str()),
    ];

    let response = client
        .post(&token_url)
        .form(&params)
        .send()
        .await
        .map_err(|e| format!("Failed to request AAD token: {}", e))?;

    let status = response.status();
    let body = response
        .text()
        .await
        .unwrap_or_else(|_| "<empty response>".to_string());

    if !status.is_success() {
        return Err(format!(
            "AAD token request failed ({}): {}",
            status,
            body.trim()
        ));
    }

    let token_response: AadTokenResponse = serde_json::from_str(&body)
        .map_err(|e| format!("Failed to parse AAD token response: {} ({})", e, body))?;

    Ok(token_response.access_token)
}

/// Build a Tiberius config, including Azure AD token retrieval when requested.
pub async fn build_tiberius_config(config: &DbConfig) -> Result<Config, String> {
    let mut sql_config = Config::new();
    sql_config.host(&config.server);
    sql_config.port(config.port);
    sql_config.database(&config.database);

    match &config.auth_method {
        DbAuthMethod::SqlPassword => {
            sql_config.authentication(tiberius::AuthMethod::sql_server(
                &config.username,
                &config.password,
            ));
        }
        DbAuthMethod::AadToken(token) => {
            sql_config.authentication(tiberius::AuthMethod::aad_token(token));
        }
        DbAuthMethod::AadPassword => {
            let token = fetch_aad_token(config).await?;
            sql_config.authentication(tiberius::AuthMethod::aad_token(token));
        }
    }

    sql_config.encryption(tiberius::EncryptionLevel::Required);

    if config.trust_cert {
        sql_config.trust_cert();
    }

    Ok(sql_config)
}

/// Initialize the global database connection pool.
///
/// # Arguments
/// * `config` - Database configuration
///
/// # Returns
/// Ok(()) if pool initialized successfully, Err otherwise.
///
/// # Errors
/// Returns an error if the pool cannot be created or is already initialized.
/// Common errors:
/// - "Timed out in bb8": Firewall blocking connection or server unreachable
/// - "Login failed": Invalid credentials
/// - "Cannot open server": Server name incorrect or firewall blocking
pub async fn init_pool(config: &DbConfig) -> Result<(), String> {
    if DB_POOL.get().is_some() {
        return Ok(());
    }

    let sql_config = build_tiberius_config(config).await?;

    let manager = ConnectionManager::new(sql_config);

    // Build pool with extended timeout for initial connection
    let pool = Pool::builder()
        .max_size(5)
        .connection_timeout(Duration::from_secs(30))
        .build(manager)
        .await
        .map_err(|e| {
            let err_msg = format!("Failed to create connection pool: {}", e);
            if err_msg.to_lowercase().contains("timeout") {
                format!(
                    "{}\n\nPossible causes: firewall blocking 1433, wrong hostname, or invalid Azure AD token/credentials.",
                    err_msg
                )
            } else {
                err_msg
            }
        })?;

    DB_POOL
        .set(pool)
        .map_err(|_| "Pool already initialized".to_string())?;

    Ok(())
}

/// Get a reference to the global database pool.
///
/// # Returns
/// Reference to the pool if initialized, Err otherwise.
pub fn get_pool() -> Result<&'static DbPool, String> {
    DB_POOL
        .get()
        .ok_or_else(|| "Database pool not initialized. Call init_pool first.".to_string())
}
