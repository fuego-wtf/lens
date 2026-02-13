use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::Arc;
use crate::oauth::OAuthBroker;

/// Trait for invoking external MCP tools from within a lens.
///
/// Injected by the host (Desktop) into `LensContext` when MCP servers are available.
/// Lenses should treat this as optional — always provide a local fallback when
/// `tool_caller` is `None` or when a call fails.
///
/// # Example
/// ```ignore
/// if let Some(caller) = &ctx.tool_caller {
///     match caller.call_tool("mcp__graphyn-base__search", json!({"query": "Button"})).await {
///         Ok(result) => { /* use real KB results */ }
///         Err(_) => { /* fall back to local patterns */ }
///     }
/// }
/// ```
#[async_trait]
pub trait ToolCaller: Send + Sync {
    /// Call an MCP tool by name with JSON parameters. Returns JSON result.
    async fn call_tool(&self, name: &str, params: serde_json::Value) -> crate::Result<serde_json::Value>;
}

/// Context passed to lens execution
#[derive(Clone, Serialize, Deserialize)]
pub struct LensContext {
    /// Current working directory
    pub cwd: PathBuf,

    /// Lens-specific input data
    pub input: serde_json::Value,

    /// Optional configuration
    #[serde(skip_serializing_if = "Option::is_none")]
    pub config: Option<serde_json::Value>,

    /// Optional MCP tool caller — injected by host (Desktop) when available.
    /// Lenses should always fall back gracefully when this is `None`.
    #[serde(skip)]
    pub tool_caller: Option<Arc<dyn ToolCaller>>,

    /// Optional OAuth broker — injected by host (Desktop) for third-party API access.
    /// Lenses declare OAuth requirements in lens.toml → Desktop injects broker.
    #[serde(skip)]
    pub oauth_broker: Option<Arc<dyn OAuthBroker>>,
}

impl std::fmt::Debug for LensContext {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("LensContext")
            .field("cwd", &self.cwd)
            .field("input", &self.input)
            .field("config", &self.config)
            .field("tool_caller", &self.tool_caller.as_ref().map(|_| "<ToolCaller>"))
            .field("oauth_broker", &self.oauth_broker.as_ref().map(|_| "<OAuthBroker>"))
            .finish()
    }
}

impl LensContext {
    /// Create a new lens context
    pub fn new(cwd: PathBuf, input: serde_json::Value) -> Self {
        Self {
            cwd,
            input,
            config: None,
            tool_caller: None,
            oauth_broker: None,
        }
    }

    /// Create context with configuration
    pub fn with_config(cwd: PathBuf, input: serde_json::Value, config: serde_json::Value) -> Self {
        Self {
            cwd,
            input,
            config: Some(config),
            tool_caller: None,
            oauth_broker: None,
        }
    }

    /// Attach a tool caller to this context (builder pattern)
    pub fn with_tool_caller(mut self, caller: Arc<dyn ToolCaller>) -> Self {
        self.tool_caller = Some(caller);
        self
    }

    /// Attach an OAuth broker to this context (builder pattern)
    pub fn with_oauth_broker(mut self, broker: Arc<dyn OAuthBroker>) -> Self {
        self.oauth_broker = Some(broker);
        self
    }
}

/// Result returned from lens execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LensResult {
    /// Success or failure
    pub success: bool,

    /// Lens-specific output data
    pub output: serde_json::Value,

    /// Optional message
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
}

impl LensResult {
    /// Create a success result
    pub fn success(output: serde_json::Value) -> Self {
        Self {
            success: true,
            output,
            message: None,
        }
    }

    /// Create a success result with message
    pub fn success_with_message(output: serde_json::Value, message: String) -> Self {
        Self {
            success: true,
            output,
            message: Some(message),
        }
    }

    /// Create a failure result
    pub fn failure(message: String) -> Self {
        Self {
            success: false,
            output: serde_json::Value::Null,
            message: Some(message),
        }
    }

    /// Create a failure result with output
    pub fn failure_with_output(output: serde_json::Value, message: String) -> Self {
        Self {
            success: false,
            output,
            message: Some(message),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_lens_context_new() {
        let cwd = PathBuf::from("/tmp/test");
        let input = json!({"key": "value"});

        let ctx = LensContext::new(cwd.clone(), input.clone());

        assert_eq!(ctx.cwd, cwd);
        assert_eq!(ctx.input, input);
        assert!(ctx.config.is_none());
    }

    #[test]
    fn test_lens_context_with_config() {
        let cwd = PathBuf::from("/tmp/test");
        let input = json!({"file_key": "abc123"});
        let config = json!({"timeout": 30});

        let ctx = LensContext::with_config(cwd.clone(), input.clone(), config.clone());

        assert_eq!(ctx.cwd, cwd);
        assert_eq!(ctx.input, input);
        assert_eq!(ctx.config, Some(config));
    }

    #[test]
    fn test_lens_context_serialization() {
        let ctx = LensContext::new(
            PathBuf::from("/tmp"),
            json!({"test": true}),
        );

        let serialized = serde_json::to_string(&ctx).unwrap();
        let deserialized: LensContext = serde_json::from_str(&serialized).unwrap();

        assert_eq!(deserialized.cwd, ctx.cwd);
        assert_eq!(deserialized.input, ctx.input);
        assert_eq!(deserialized.config, ctx.config);
    }

    #[test]
    fn test_lens_context_config_skipped_when_none() {
        let ctx = LensContext::new(PathBuf::from("/tmp"), json!({}));
        let serialized = serde_json::to_string(&ctx).unwrap();

        assert!(!serialized.contains("config"));
    }

    #[test]
    fn test_lens_result_success() {
        let output = json!({"result": "ok"});
        let result = LensResult::success(output.clone());

        assert!(result.success);
        assert_eq!(result.output, output);
        assert!(result.message.is_none());
    }

    #[test]
    fn test_lens_result_success_with_message() {
        let output = json!({"count": 42});
        let message = "Processed 42 items".to_string();

        let result = LensResult::success_with_message(output.clone(), message.clone());

        assert!(result.success);
        assert_eq!(result.output, output);
        assert_eq!(result.message, Some(message));
    }

    #[test]
    fn test_lens_result_failure() {
        let message = "Something went wrong".to_string();
        let result = LensResult::failure(message.clone());

        assert!(!result.success);
        assert_eq!(result.output, serde_json::Value::Null);
        assert_eq!(result.message, Some(message));
    }

    #[test]
    fn test_lens_result_failure_with_output() {
        let output = json!({"partial": "data"});
        let message = "Partial failure".to_string();

        let result = LensResult::failure_with_output(output.clone(), message.clone());

        assert!(!result.success);
        assert_eq!(result.output, output);
        assert_eq!(result.message, Some(message));
    }

    #[test]
    fn test_lens_result_serialization() {
        let result = LensResult::success_with_message(
            json!({"data": [1, 2, 3]}),
            "Complete".to_string(),
        );

        let serialized = serde_json::to_string(&result).unwrap();
        let deserialized: LensResult = serde_json::from_str(&serialized).unwrap();

        assert_eq!(deserialized.success, result.success);
        assert_eq!(deserialized.output, result.output);
        assert_eq!(deserialized.message, result.message);
    }

    #[test]
    fn test_lens_result_message_skipped_when_none() {
        let result = LensResult::success(json!({}));
        let serialized = serde_json::to_string(&result).unwrap();

        assert!(!serialized.contains("message"));
    }
}
