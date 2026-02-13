//! # Lens
//!
//! A trait standard for crafting specialized agent perspectives.
//!
//! This crate defines the **Lens** interface — what a Lens is, how it
//! communicates, and how it streams events. It contains no runtime,
//! no loader, no host logic. Pure contract.
//!
//! Implementations live in separate crates (e.g., `graphyn-lens` ecosystem).
//!
//! # Quick Start
//!
//! ```rust
//! use async_trait::async_trait;
//! use lens::{Lens, LensContext, LensResult, Result};
//!
//! struct MyLens;
//!
//! #[async_trait]
//! impl Lens for MyLens {
//!     fn id(&self) -> &str { "my-lens" }
//!     fn name(&self) -> &str { "My Lens" }
//!     fn version(&self) -> &str { "0.1.0" }
//!
//!     async fn execute(&self, ctx: LensContext) -> Result<LensResult> {
//!         Ok(LensResult::success(serde_json::json!({ "ok": true })))
//!     }
//! }
//! ```

pub mod context;
pub mod error;
pub mod events;
pub mod lens;
pub mod manifest;
pub mod mcp_server;
pub mod oauth;
pub mod output_spec;
pub mod streaming;

pub use context::{LensContext, LensResult, ToolCaller};
pub use error::{LensError, Result};
pub use oauth::{OAuthBroker, OAuthError, OAuthToken};
pub use events::LensEvent;
pub use lens::Lens;
pub use manifest::{
    LensDependency, LensManifest, LensMetadata, MessageType, Permission, SandboxLevel,
    SecurityConfig,
};
pub use mcp_server::{
    McpContent, McpPropertySchema, McpServerLens, McpTool, McpToolBuilder, McpToolResponse,
    McpToolSchema,
};
pub use output_spec::{
    InteractivityMode, LensOutputSpec, OutputDefinition, OutputErrorMode, RenderBlock,
    RenderBlockType, OUTPUT_SPEC_FILENAME,
};
pub use streaming::{LensEventStream, StreamingLens};
