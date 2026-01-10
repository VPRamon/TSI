//! Database configuration and environment variable handling.

use std::env;

/// Authentication method to use when connecting to SQL Server.
#[derive(Debug, Clone)]
pub enum DbAuthMethod {
    /// Traditional SQL Server username/password authentication.
    SqlPassword,
    /// Azure AD password flow (ROPC) to obtain an access token.
    AadPassword,
    /// Direct Azure AD access token provided via env var.
    AadToken(String),
}

/// Database configuration loaded from environment variables.
#[derive(Debug, Clone)]
pub struct DbConfig {
    /// SQL Server hostname
    pub server: String,
    /// Database name
    pub database: String,
    /// Username for authentication (SQL login or AAD UPN)
    pub username: String,
    /// Password for authentication (SQL password or AAD password)
    pub password: String,
    /// SQL Server port (default: 1433)
    pub port: u16,
    /// Whether to trust the server certificate
    pub trust_cert: bool,
    /// Authentication strategy resolved from env vars
    pub auth_method: DbAuthMethod,
    /// Azure AD tenant id (or `common`/`organizations`)
    pub tenant_id: String,
    /// Azure AD client id used for the password flow
    pub client_id: String,
    /// Resource to request in the AAD token (default: Azure SQL)
    pub resource: String,
}

impl DbConfig {
    /// Create a new database configuration from environment variables.
    ///
    /// # Environment Variables
    /// - `DB_SERVER` (required): SQL Server hostname
    /// - `DB_DATABASE` (required): Database name
    /// - `DB_USERNAME` (required): Username (SQL login or AAD UPN)
    /// - `DB_PASSWORD` (required): Password (SQL password or AAD password)
    /// - `DB_PORT` (optional, default: 1433): SQL Server port
    /// - `DB_TRUST_CERT` (optional, default: true): Trust server certificate
    /// - `DB_AUTH_METHOD` (optional): `sql_password` | `aad_password` | `aad_token`
    ///   - defaults to `aad_password` when username looks like an email (UPN), otherwise SQL password
    /// - `AZURE_TENANT_ID` (optional): Tenant id for AAD (`common` by default)
    /// - `AZURE_CLIENT_ID` (optional): Client id for ROPC (`1950a258-227b-4e31-a9cf-717495945fc2` by default)
    /// - `AZURE_SQL_RESOURCE` (optional): Resource identifier (`https://database.windows.net/` by default)
    /// - `AZURE_ACCESS_TOKEN` (required if `DB_AUTH_METHOD=aad_token`)
    ///
    /// # Errors
    /// Returns an error if required variables are not set.
    pub fn from_env() -> Result<Self, String> {
        let server = env::var("DB_SERVER")
            .map_err(|_| "DB_SERVER environment variable not set".to_string())?;
        let database = env::var("DB_DATABASE")
            .map_err(|_| "DB_DATABASE environment variable not set".to_string())?;
        let username = env::var("DB_USERNAME")
            .map_err(|_| "DB_USERNAME environment variable not set".to_string())?;
        let password = env::var("DB_PASSWORD")
            .map_err(|_| "DB_PASSWORD environment variable not set".to_string())?;
        let port = env::var("DB_PORT")
            .unwrap_or_else(|_| "1433".to_string())
            .parse()
            .map_err(|_| "DB_PORT must be a valid port number".to_string())?;
        let trust_cert = env::var("DB_TRUST_CERT")
            .unwrap_or_else(|_| "true".to_string())
            .parse()
            .unwrap_or(true);

        let auth_method_env = env::var("DB_AUTH_METHOD").unwrap_or_else(|_| "".to_string());
        let tenant_id = env::var("AZURE_TENANT_ID").unwrap_or_else(|_| "common".to_string());
        let client_id = env::var("AZURE_CLIENT_ID")
            // Public client id used by Microsoft tools (works with ROPC for Azure SQL)
            .unwrap_or_else(|_| "1950a258-227b-4e31-a9cf-717495945fc2".to_string());
        let resource = env::var("AZURE_SQL_RESOURCE")
            .unwrap_or_else(|_| "https://database.windows.net/".to_string());

        let auth_method = match auth_method_env.to_lowercase().as_str() {
            "aad_password" | "aad" | "active_directory_password" => DbAuthMethod::AadPassword,
            "aad_token" | "access_token" => {
                let token = env::var("AZURE_ACCESS_TOKEN").map_err(|_| {
                    "AZURE_ACCESS_TOKEN must be set when DB_AUTH_METHOD=aad_token".to_string()
                })?;
                DbAuthMethod::AadToken(token)
            }
            "sql" | "sql_password" | "" => {
                // If the username looks like an AAD UPN, default to AAD password auth
                if username.contains('@') {
                    DbAuthMethod::AadPassword
                } else {
                    DbAuthMethod::SqlPassword
                }
            }
            other => {
                return Err(format!(
                "Unsupported DB_AUTH_METHOD '{}'. Use sql_password, aad_password, or aad_token.",
                other
            ))
            }
        };

        Ok(Self {
            server,
            database,
            username,
            password,
            port,
            trust_cert,
            auth_method,
            tenant_id,
            client_id,
            resource,
        })
    }
}
