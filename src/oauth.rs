use async_trait::async_trait;
use std::time::SystemTime;
use thiserror::Error;

/// Broker interface for fetching OAuth tokens on behalf of lenses.
#[async_trait]
pub trait OAuthBroker: Send + Sync {
    /// Fetch a token for the given provider (e.g. "figma").
    async fn get_token(&self, provider: &str) -> Result<OAuthToken, OAuthError>;

    /// Check whether the user has connected the provider.
    async fn is_connected(&self, provider: &str) -> bool;
}

/// OAuth token payload returned by the broker.
#[derive(Debug, Clone)]
pub struct OAuthToken {
    pub access_token: String,
    pub refresh_token: Option<String>,
    pub expires_at: Option<SystemTime>,
    pub scope: Option<String>,
}

/// OAuth broker errors.
#[derive(Error, Debug)]
pub enum OAuthError {
    #[error("OAuth provider not connected: {0}")]
    NotConnected(String),

    #[error("OAuth token expired")]
    Expired,

    #[error("OAuth token fetch failed: {0}")]
    NetworkError(String),
}
