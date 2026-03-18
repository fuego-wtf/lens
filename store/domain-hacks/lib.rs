//! # Graphyn Figma Plugin
//!
//! Design decomposition plugin that transforms Figma designs into:
//! - Structured component inventory
//! - Design tokens (colors, typography, spacing)
//! - User flow sequences
//! - Linear task specifications
//!
//! ## Plugin as Conversational Agent
//!
//! This plugin operates as a specialized agent thread where:
//! - `PluginEvent::Progress` = Agent thinking/working messages
//! - `PluginEvent::Data` = Rich output (components, tokens, flows)
//! - `PluginEvent::Started/Completed` = Thread lifecycle
//!
//! ## 8-Phase Pipeline
//!
//! 1. **Token Extraction** - Colors, typography, spacing from Figma variables
//! 2. **Frame Discovery** - Identify all frames and their hierarchy
//! 3. **Flow Analysis** - Detect user journeys through prototype links
//! 4. **Component Mapping** - Map Figma components to code patterns
//! 5. **Deduplication** - Remove duplicates with user checkpoints
//! 6. **State Analysis** - Identify component variants/states
//! 7. **Knowledge Query** - Match to @base/ patterns
//! 8. **Task Generation** - Create Linear-ready task specs

pub mod error;
pub mod export;
pub mod figma_api;
pub mod phases;
pub mod plugin;
pub mod ralph;
pub mod types;

pub use error::{FigmaError, Result};
pub use plugin::FigmaPlugin;
pub use types::*;
