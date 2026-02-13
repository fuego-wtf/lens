use thiserror::Error;

/// Lens framework error types
#[derive(Error, Debug)]
pub enum LensError {
    #[error("Lens execution failed: {0}")]
    ExecutionFailed(String),

    #[error("Invalid lens context: {0}")]
    InvalidContext(String),

    #[error("Invalid lens input: {0}")]
    InvalidInput(String),

    #[error("Lens not found: {0}")]
    LensNotFound(String),

    #[error("Lens initialization failed: {0}")]
    Initialization(String),

    #[error("Event stream error: {0}")]
    StreamError(String),

    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("{0}")]
    Other(String),
}

/// Result type for lens operations
pub type Result<T> = std::result::Result<T, LensError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_execution_failed_error() {
        let error = LensError::ExecutionFailed("task crashed".to_string());
        assert_eq!(error.to_string(), "Lens execution failed: task crashed");
    }

    #[test]
    fn test_invalid_context_error() {
        let error = LensError::InvalidContext("missing cwd".to_string());
        assert_eq!(error.to_string(), "Invalid lens context: missing cwd");
    }

    #[test]
    fn test_invalid_input_error() {
        let error = LensError::InvalidInput("expected JSON object".to_string());
        assert_eq!(error.to_string(), "Invalid lens input: expected JSON object");
    }

    #[test]
    fn test_lens_not_found_error() {
        let error = LensError::LensNotFound("figma".to_string());
        assert_eq!(error.to_string(), "Lens not found: figma");
    }

    #[test]
    fn test_stream_error() {
        let error = LensError::StreamError("channel closed".to_string());
        assert_eq!(error.to_string(), "Event stream error: channel closed");
    }

    #[test]
    fn test_serialization_error_from_serde() {
        let json_error = serde_json::from_str::<serde_json::Value>("invalid").unwrap_err();
        let error: LensError = json_error.into();
        assert!(error.to_string().starts_with("Serialization error:"));
    }

    #[test]
    fn test_io_error_from_std() {
        let io_error = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
        let error: LensError = io_error.into();
        assert!(error.to_string().starts_with("IO error:"));
    }

    #[test]
    fn test_other_error() {
        let error = LensError::Other("unknown error".to_string());
        assert_eq!(error.to_string(), "unknown error");
    }

    #[test]
    fn test_error_is_debug() {
        let error = LensError::ExecutionFailed("test".to_string());
        let debug_str = format!("{:?}", error);
        assert!(debug_str.contains("ExecutionFailed"));
    }

    #[test]
    fn test_result_type_ok() {
        let result: Result<i32> = Ok(42);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 42);
    }

    #[test]
    fn test_result_type_err() {
        let result: Result<i32> = Err(LensError::Other("fail".to_string()));
        assert!(result.is_err());
    }
}
