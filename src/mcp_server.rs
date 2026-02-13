//! # MCP Server Interface for Lenses
//!
//! Enables lenses to expose MCP (Model Context Protocol) tools for agent consumption.
//!
//! This creates a **dual-interface** pattern where the same lens can be:
//! - Used by users via @mention → `Lens::execute()`
//! - Used by agents via MCP → `McpServerLens::call_tool()`
//!
//! # Architecture
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────────────────────┐
//! │                          DUAL-INTERFACE LENS                              │
//! │                                                                             │
//! │   USER INTERFACE              │   AGENT INTERFACE                           │
//! │   (LensGateway)             │   (MCP Server)                              │
//! │                               │                                             │
//! │   @base search                │   mcp.graphyn-base.search()                 │
//! │        │                      │          │                                  │
//! │        ▼                      │          ▼                                  │
//! │   Lens::execute(ctx)        │   McpServerLens::call_tool("search", {})  │
//! │        │                      │          │                                  │
//! │        └──────────────────────┴──────────┘                                  │
//! │                               │                                             │
//! │                               ▼                                             │
//! │                      Arc<SharedLogic>::search()                             │
//! │                                                                             │
//! └─────────────────────────────────────────────────────────────────────────────┘
//! ```
//!
//! # Example Implementation
//!
//! ```rust,ignore
//! use lens::{Lens, McpServerLens, McpTool, McpToolInput};
//! use std::sync::Arc;
//!
//! struct KnowledgeBase { /* shared logic */ }
//!
//! impl KnowledgeBase {
//!     async fn search(&self, query: &str) -> Vec<Document> { /* ... */ }
//! }
//!
//! struct BaseLens {
//!     kb: Arc<KnowledgeBase>,
//! }
//!
//! #[async_trait]
//! impl Lens for BaseLens {
//!     fn id(&self) -> &str { "base" }
//!     fn name(&self) -> &str { "Knowledge Base" }
//!     fn version(&self) -> &str { "1.0.0" }
//!
//!     async fn execute(&self, ctx: LensContext) -> Result<LensResult> {
//!         let query = ctx.input.get("query").unwrap().as_str().unwrap();
//!         let results = self.kb.search(query).await;
//!         Ok(LensResult::success(serde_json::to_value(results)?))
//!     }
//! }
//!
//! #[async_trait]
//! impl McpServerLens for BaseLens {
//!     fn mcp_tools(&self) -> Vec<McpTool> {
//!         vec![
//!             McpTool::new("search")
//!                 .description("Search the knowledge base")
//!                 .input(McpToolInput::string("query", "Search query").required())
//!                 .build(),
//!         ]
//!     }
//!
//!     async fn call_tool(&self, name: &str, params: Value) -> Result<Value> {
//!         match name {
//!             "search" => {
//!                 let query = params["query"].as_str().unwrap();
//!                 let results = self.kb.search(query).await;
//!                 Ok(serde_json::to_value(results)?)
//!             }
//!             _ => Err(LensError::InvalidInput(format!("Unknown tool: {}", name))),
//!         }
//!     }
//! }
//! ```

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::error::Result;
use crate::lens::Lens;

/// MCP tool definition that agents can call
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpTool {
    /// Tool name (e.g., "search", "decompose", "validate")
    pub name: String,

    /// Human-readable description of what the tool does
    pub description: String,

    /// JSON Schema for tool input parameters
    #[serde(rename = "inputSchema")]
    pub input_schema: McpToolSchema,
}

/// JSON Schema for MCP tool inputs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpToolSchema {
    /// Schema type (always "object" for MCP tools)
    #[serde(rename = "type")]
    pub schema_type: String,

    /// Property definitions
    #[serde(default)]
    pub properties: std::collections::HashMap<String, McpPropertySchema>,

    /// Required property names
    #[serde(default)]
    pub required: Vec<String>,
}

/// Schema for a single property
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpPropertySchema {
    /// Property type ("string", "number", "boolean", "object", "array")
    #[serde(rename = "type")]
    pub prop_type: String,

    /// Property description
    #[serde(default)]
    pub description: String,

    /// Default value (if any)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub default: Option<Value>,

    /// Enum values (for string enums)
    #[serde(default, skip_serializing_if = "Option::is_none", rename = "enum")]
    pub enum_values: Option<Vec<String>>,

    /// Items schema (for arrays)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub items: Option<Box<McpPropertySchema>>,
}

/// Builder for creating MCP tools with fluent API
pub struct McpToolBuilder {
    name: String,
    description: String,
    properties: std::collections::HashMap<String, McpPropertySchema>,
    required: Vec<String>,
}

impl McpToolBuilder {
    /// Create a new tool builder
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            description: String::new(),
            properties: std::collections::HashMap::new(),
            required: Vec::new(),
        }
    }

    /// Set the tool description
    pub fn description(mut self, desc: impl Into<String>) -> Self {
        self.description = desc.into();
        self
    }

    /// Add a string parameter
    pub fn string_param(mut self, name: impl Into<String>, desc: impl Into<String>) -> Self {
        let name = name.into();
        self.properties.insert(
            name.clone(),
            McpPropertySchema {
                prop_type: "string".to_string(),
                description: desc.into(),
                default: None,
                enum_values: None,
                items: None,
            },
        );
        self
    }

    /// Add a required string parameter
    pub fn string_param_required(
        mut self,
        name: impl Into<String>,
        desc: impl Into<String>,
    ) -> Self {
        let name = name.into();
        self.properties.insert(
            name.clone(),
            McpPropertySchema {
                prop_type: "string".to_string(),
                description: desc.into(),
                default: None,
                enum_values: None,
                items: None,
            },
        );
        self.required.push(name);
        self
    }

    /// Add a number parameter
    pub fn number_param(mut self, name: impl Into<String>, desc: impl Into<String>) -> Self {
        let name = name.into();
        self.properties.insert(
            name.clone(),
            McpPropertySchema {
                prop_type: "number".to_string(),
                description: desc.into(),
                default: None,
                enum_values: None,
                items: None,
            },
        );
        self
    }

    /// Add a boolean parameter
    pub fn bool_param(mut self, name: impl Into<String>, desc: impl Into<String>) -> Self {
        let name = name.into();
        self.properties.insert(
            name.clone(),
            McpPropertySchema {
                prop_type: "boolean".to_string(),
                description: desc.into(),
                default: None,
                enum_values: None,
                items: None,
            },
        );
        self
    }

    /// Add an object parameter
    pub fn object_param(mut self, name: impl Into<String>, desc: impl Into<String>) -> Self {
        let name = name.into();
        self.properties.insert(
            name.clone(),
            McpPropertySchema {
                prop_type: "object".to_string(),
                description: desc.into(),
                default: None,
                enum_values: None,
                items: None,
            },
        );
        self
    }

    /// Add a string enum parameter
    pub fn enum_param(
        mut self,
        name: impl Into<String>,
        desc: impl Into<String>,
        values: Vec<String>,
    ) -> Self {
        let name = name.into();
        self.properties.insert(
            name.clone(),
            McpPropertySchema {
                prop_type: "string".to_string(),
                description: desc.into(),
                default: None,
                enum_values: Some(values),
                items: None,
            },
        );
        self
    }

    /// Mark a parameter as required
    pub fn required(mut self, name: impl Into<String>) -> Self {
        self.required.push(name.into());
        self
    }

    /// Build the MCP tool
    pub fn build(self) -> McpTool {
        McpTool {
            name: self.name,
            description: self.description,
            input_schema: McpToolSchema {
                schema_type: "object".to_string(),
                properties: self.properties,
                required: self.required,
            },
        }
    }
}

impl McpTool {
    /// Create a new tool builder
    pub fn builder(name: impl Into<String>) -> McpToolBuilder {
        McpToolBuilder::new(name)
    }
}

/// MCP tool call response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpToolResponse {
    /// Response content
    pub content: Vec<McpContent>,

    /// Whether this is an error response
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub is_error: bool,
}

/// MCP content block
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum McpContent {
    /// Text content
    #[serde(rename = "text")]
    Text { text: String },

    /// Image content (base64)
    #[serde(rename = "image")]
    Image { data: String, mime_type: String },

    /// Resource reference
    #[serde(rename = "resource")]
    Resource { uri: String, text: Option<String> },
}

impl McpToolResponse {
    /// Create a text response
    pub fn text(content: impl Into<String>) -> Self {
        Self {
            content: vec![McpContent::Text {
                text: content.into(),
            }],
            is_error: false,
        }
    }

    /// Create a JSON response (serialized to text)
    pub fn json<T: Serialize>(value: &T) -> Result<Self> {
        let text = serde_json::to_string_pretty(value)
            .map_err(|e| crate::error::LensError::Other(e.to_string()))?;
        Ok(Self::text(text))
    }

    /// Create an error response
    pub fn error(message: impl Into<String>) -> Self {
        Self {
            content: vec![McpContent::Text {
                text: message.into(),
            }],
            is_error: true,
        }
    }
}

/// Trait for lenses that expose MCP tools for agent consumption
///
/// This enables the dual-interface pattern where the same lens serves
/// both users (via @mention) and agents (via MCP protocol).
///
/// # Optionality
///
/// This trait is **optional**. Lenses that only need user-facing functionality
/// can implement just the `Lens` trait. The framework detects MCP support
/// at runtime using the `supports_mcp` helper.
///
/// # Example (Lens without MCP)
///
/// ```rust,ignore
/// // Simple lens - only implements Lens trait
/// struct SimpleLens;
///
/// impl Lens for SimpleLens {
///     fn id(&self) -> &str { "simple" }
///     fn name(&self) -> &str { "Simple Lens" }
///     fn version(&self) -> &str { "1.0.0" }
///     // ... execute implementation
/// }
/// // No McpServerLens impl needed!
/// ```
#[async_trait]
pub trait McpServerLens: Lens {
    /// List MCP tools this lens provides
    ///
    /// Called by MCP clients to discover available tools.
    fn mcp_tools(&self) -> Vec<McpTool>;

    /// Handle an MCP tool call
    ///
    /// # Arguments
    ///
    /// * `name` - Tool name (from `mcp_tools()`)
    /// * `params` - JSON parameters matching the tool's input schema
    ///
    /// # Returns
    ///
    /// MCP-formatted response with content blocks
    async fn call_tool(&self, name: &str, params: Value) -> Result<McpToolResponse>;

    /// Get the MCP server name (defaults to lens ID)
    fn mcp_server_name(&self) -> String {
        format!("graphyn-{}", self.id())
    }

    /// Get the MCP server version (defaults to lens version)
    fn mcp_server_version(&self) -> String {
        self.version().to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tool_builder_simple() {
        let tool = McpTool::builder("search")
            .description("Search for documents")
            .string_param_required("query", "The search query")
            .number_param("limit", "Maximum results to return")
            .build();

        assert_eq!(tool.name, "search");
        assert_eq!(tool.description, "Search for documents");
        assert_eq!(tool.input_schema.schema_type, "object");
        assert_eq!(tool.input_schema.properties.len(), 2);
        assert!(tool.input_schema.required.contains(&"query".to_string()));
    }

    #[test]
    fn test_tool_builder_with_enum() {
        let tool = McpTool::builder("format")
            .description("Format output")
            .enum_param(
                "format",
                "Output format",
                vec!["json".to_string(), "yaml".to_string(), "toml".to_string()],
            )
            .required("format")
            .build();

        let format_prop = tool.input_schema.properties.get("format").unwrap();
        assert!(format_prop.enum_values.is_some());
        assert_eq!(format_prop.enum_values.as_ref().unwrap().len(), 3);
    }

    #[test]
    fn test_mcp_response_text() {
        let response = McpToolResponse::text("Hello, world!");
        assert!(!response.is_error);
        assert_eq!(response.content.len(), 1);

        match &response.content[0] {
            McpContent::Text { text } => assert_eq!(text, "Hello, world!"),
            _ => panic!("Expected text content"),
        }
    }

    #[test]
    fn test_mcp_response_json() {
        let data = serde_json::json!({"results": [1, 2, 3]});
        let response = McpToolResponse::json(&data).unwrap();

        match &response.content[0] {
            McpContent::Text { text } => {
                assert!(text.contains("results"));
            }
            _ => panic!("Expected text content"),
        }
    }

    #[test]
    fn test_mcp_response_error() {
        let response = McpToolResponse::error("Something went wrong");
        assert!(response.is_error);

        match &response.content[0] {
            McpContent::Text { text } => assert_eq!(text, "Something went wrong"),
            _ => panic!("Expected text content"),
        }
    }

    #[test]
    fn test_tool_serialization() {
        let tool = McpTool::builder("test")
            .description("Test tool")
            .string_param_required("input", "Test input")
            .build();

        let json = serde_json::to_string(&tool).unwrap();
        assert!(json.contains("inputSchema"));
        assert!(json.contains("\"type\":\"object\""));
    }
}
