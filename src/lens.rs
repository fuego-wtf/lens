use async_trait::async_trait;

use crate::{LensContext, LensResult, Result};

/// Core lens trait that all lenses must implement.
///
/// # Example
///
/// ```rust
/// use async_trait::async_trait;
/// use lens::{Lens, LensContext, LensResult, Result};
///
/// struct HelloLens;
///
/// #[async_trait]
/// impl Lens for HelloLens {
///     fn id(&self) -> &str { "hello" }
///     fn name(&self) -> &str { "Hello Lens" }
///     fn version(&self) -> &str { "1.0.0" }
///
///     async fn execute(&self, ctx: LensContext) -> Result<LensResult> {
///         let name = ctx.input.get("name")
///             .and_then(|v| v.as_str())
///             .unwrap_or("World");
///
///         Ok(LensResult::success(serde_json::json!({
///             "greeting": format!("Hello, {}!", name)
///         })))
///     }
/// }
/// ```
#[async_trait]
pub trait Lens: Send + Sync {
    /// Unique lens identifier (e.g. "nolimit", "figma")
    fn id(&self) -> &str;

    /// Human-readable lens name
    fn name(&self) -> &str;

    /// Lens version (semver)
    fn version(&self) -> &str;

    /// Execute the lens task
    ///
    /// This is a non-streaming execution. For observable execution
    /// with progress events, implement `StreamingLens` instead.
    async fn execute(&self, ctx: LensContext) -> Result<LensResult>;

    /// Whether this lens exposes MCP tools for agent consumption
    ///
    /// Override to return `true` if the lens implements `McpServerLens`.
    /// This allows runtime detection without complex downcasting.
    ///
    /// # Default
    ///
    /// Returns `false` by default. Lenses implementing `McpServerLens`
    /// should override this to return `true`.
    fn supports_mcp(&self) -> bool {
        false
    }

    /// Lens description for discovery and documentation
    fn description(&self) -> &str {
        ""
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use std::path::PathBuf;

    struct TestLens {
        id: String,
        name: String,
        version: String,
    }

    impl TestLens {
        fn new() -> Self {
            Self {
                id: "test".to_string(),
                name: "Test Lens".to_string(),
                version: "1.0.0".to_string(),
            }
        }
    }

    #[async_trait]
    impl Lens for TestLens {
        fn id(&self) -> &str {
            &self.id
        }

        fn name(&self) -> &str {
            &self.name
        }

        fn version(&self) -> &str {
            &self.version
        }

        async fn execute(&self, ctx: LensContext) -> Result<LensResult> {
            let value = ctx.input.get("value")
                .and_then(|v| v.as_i64())
                .unwrap_or(0);

            Ok(LensResult::success(json!({
                "doubled": value * 2
            })))
        }
    }

    #[test]
    fn test_lens_metadata() {
        let lens = TestLens::new();

        assert_eq!(lens.id(), "test");
        assert_eq!(lens.name(), "Test Lens");
        assert_eq!(lens.version(), "1.0.0");
    }

    #[tokio::test]
    async fn test_lens_execute_success() {
        let lens = TestLens::new();
        let ctx = LensContext::new(
            PathBuf::from("/tmp"),
            json!({"value": 21}),
        );

        let result = lens.execute(ctx).await.unwrap();

        assert!(result.success);
        assert_eq!(result.output["doubled"], 42);
    }

    #[tokio::test]
    async fn test_lens_execute_with_default() {
        let lens = TestLens::new();
        let ctx = LensContext::new(
            PathBuf::from("/tmp"),
            json!({}),
        );

        let result = lens.execute(ctx).await.unwrap();

        assert!(result.success);
        assert_eq!(result.output["doubled"], 0);
    }

    #[test]
    fn test_lens_is_send_sync() {
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<TestLens>();
    }
}
