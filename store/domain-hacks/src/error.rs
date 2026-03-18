use thiserror::Error;

#[derive(Debug, Error)]
pub enum DomainError {
    #[error("Invalid domain format: {0}")]
    InvalidDomain(String),
    #[error("Domain availability check failed: {0}")]
    AvailabilityCheckFailed(String),
    #[error("DNS resolution failed: {0}")]
    DnsResolutionFailed(String),
    #[error("HTTP verification failed: {0}")]
    HttpVerificationFailed(String),
    #[error("Strategy generation failed: {0}")]
    StrategyGenerationFailed(String),
    #[error("Landing page generation failed: {0}")]
    LandingPageGenerationFailed(String),
    #[error("Parse error: {0}")]
    Parse(String),
    #[error("Network error: {0}")]
    Network(String),
    #[error("Configuration error: {0}")]
    Config(String),
}

pub type Result<T> = std::result::Result<T, DomainError>;
