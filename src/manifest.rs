//! # Lens Manifest Schema
//!
//! Defines the lens.toml manifest format for Graphyn lenses.
//! Lenses declare their metadata and custom message types in this file.
//!
//! # Example lens.toml
//!
//! ```toml
//! [lens]
//! id = "graphyn-base"
//! name = "Knowledge Base"
//! version = "0.1.0"
//! description = "Local knowledge base with semantic search"
//!
//! [[message_types]]
//! key = "search_results"
//! component = "components/SearchResults.tsx"
//! description = "Displays semantic search results with relevance scores"
//!
//! [[message_types]]
//! key = "document_preview"
//! component = "components/DocumentPreview.tsx"
//! description = "Renders a document with syntax highlighting"
//! ```

use serde::{Deserialize, Serialize};

/// Lens manifest parsed from lens.toml
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LensManifest {
    /// Core lens metadata
    #[serde(alias = "plugin")]
    pub lens: LensMetadata,

    /// Security configuration for installation validation
    #[serde(default)]
    pub security: Option<SecurityConfig>,

    /// Custom message types this lens emits
    #[serde(default)]
    pub message_types: Vec<MessageType>,

    /// Lens capabilities (optional)
    #[serde(default)]
    pub capabilities: Vec<String>,

    /// Dependencies on other lenses (v1 format)
    #[serde(default)]
    pub dependencies: Vec<LensDependency>,

    /// Optional keyboard shortcuts declared by this lens
    #[serde(default)]
    pub shortcuts: Vec<LensShortcut>,

    // === v2 fields ===
    /// Structured authors (v2)
    #[serde(default)]
    pub authors: Vec<Author>,

    /// License information (v2)
    #[serde(default)]
    pub license_info: Option<License>,

    /// Registry metadata (v2)
    #[serde(default)]
    pub registry: Option<RegistryMetadata>,

    /// MCP tool declarations (v2)
    #[serde(default)]
    pub mcp_tools: Vec<McpTool>,

    /// Entry points by mode (v2)
    #[serde(default)]
    pub entry_points: Vec<EntryPoint>,

    /// Lifecycle hooks (v2)
    #[serde(default)]
    pub hooks: Option<LifecycleHooks>,

    /// Enhanced dependencies with optional flag (v2)
    #[serde(default)]
    pub dependencies_v2: Vec<LensDependencyV2>,
}

/// Security configuration for lens installation
///
/// Example in lens.toml:
/// ```toml
/// [security]
/// library_hash = "sha256:a1b2c3d4e5f6..."
/// permissions = ["fs:read:~/Documents", "network:api.example.com"]
/// sandbox = "restricted"
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityConfig {
    /// SHA256 hash of the lens library for verification
    /// Format: "sha256:<hex_hash>"
    #[serde(default)]
    pub library_hash: Option<String>,

    /// Permissions requested by this lens
    /// Format: "<type>:<scope>" e.g., "fs:read:~/path", "network:api.com"
    #[serde(default)]
    pub permissions: Vec<String>,

    /// Sandbox level for lens execution
    /// - "restricted": No fs/network access
    /// - "network": Network only, no fs
    /// - "full": Full access (requires explicit approval)
    #[serde(default)]
    pub sandbox: SandboxLevel,
}

/// Sandbox levels for lens execution
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum SandboxLevel {
    /// No filesystem or network access
    #[default]
    Restricted,
    /// Network access only, no filesystem
    Network,
    /// Full access (requires explicit user approval)
    Full,
}

/// Lens surface type — determines where the lens renders (T-LENS-SURFACE-001)
///
/// Each lens declares its surface in lens.toml:
/// ```toml
/// [lens]
/// id = "my-lens"
/// surface = "pane"  # or "pack" or "tray" or "desktop_app"
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum LensSurface {
    /// Renders as an inline pane within the main layout (default)
    #[default]
    Pane,
    /// Renders as a pack/sidebar panel
    Pack,
    /// Renders as an ephemeral tray window
    Tray,
    /// Runs as a standalone desktop application window
    DesktopApp,
}

/// Parsed permission with type and scope
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Permission {
    /// Permission type: "fs", "network", "secrets", etc.
    pub permission_type: String,
    /// Permission action: "read", "write", "execute"
    pub action: Option<String>,
    /// Permission scope: path, domain, or resource identifier
    pub scope: String,
    /// Human-readable description
    pub description: String,
}

impl Permission {
    /// Parse a permission string like "fs:read:~/Documents"
    pub fn parse(permission_str: &str) -> Option<Self> {
        let parts: Vec<&str> = permission_str.splitn(3, ':').collect();
        match parts.as_slice() {
            [ptype, scope] => Some(Permission {
                permission_type: ptype.to_string(),
                action: None,
                scope: scope.to_string(),
                description: format!("{} access to {}", ptype, scope),
            }),
            [ptype, action, scope] => Some(Permission {
                permission_type: ptype.to_string(),
                action: Some(action.to_string()),
                scope: scope.to_string(),
                description: format!("{} {} to {}", action, ptype, scope),
            }),
            _ => None,
        }
    }
}

impl SecurityConfig {
    /// Parse all permission strings into Permission structs
    pub fn parsed_permissions(&self) -> Vec<Permission> {
        self.permissions
            .iter()
            .filter_map(|s| Permission::parse(s))
            .collect()
    }

    /// Check if this lens requires full access
    pub fn requires_full_access(&self) -> bool {
        self.sandbox == SandboxLevel::Full
    }

    /// Verify the library hash matches expected
    pub fn verify_hash(&self, actual_hash: &str) -> bool {
        self.library_hash
            .as_ref()
            .map(|expected| expected == actual_hash)
            .unwrap_or(true) // No hash = no verification required
    }
}

/// Core lens metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LensMetadata {
    /// Unique lens identifier (e.g., "graphyn-base", "figma")
    pub id: String,

    /// Human-readable lens name
    pub name: String,

    /// Lens version (semver)
    pub version: String,

    /// Lens description
    #[serde(default)]
    pub description: String,

    /// Lens author(s) - v1 format (string array, deprecated)
    #[serde(default)]
    pub authors: Vec<String>,

    /// Lens license - v1 format (deprecated, use license table)
    #[serde(default)]
    pub license: Option<String>,

    /// Lens homepage/repository URL
    #[serde(default)]
    pub repository: Option<String>,

    /// Minimum framework version required
    #[serde(default)]
    pub min_framework_version: Option<String>,

    /// Maximum framework version supported
    #[serde(default)]
    pub max_framework_version: Option<String>,

    /// Manifest version (1 or 2, defaults to 1)
    #[serde(default = "default_manifest_version")]
    pub manifest_version: u32,

    /// Surface type — determines where the lens renders (T-LENS-SURFACE-001)
    /// Defaults to Pane for backward compatibility
    #[serde(default)]
    pub surface: LensSurface,

    /// All supported surface types (T-LENS-SURFACE-001 multi-surface)
    /// A lens can render as both pane and pack, for example.
    /// If empty, the lens supports only its primary `surface` type.
    #[serde(default)]
    pub surfaces: Vec<LensSurface>,
}

fn default_manifest_version() -> u32 {
    1
}

/// Structured author information (v2)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Author {
    /// Author name
    pub name: String,
    /// Author email
    #[serde(default)]
    pub email: Option<String>,
    /// Author role: "maintainer", "contributor", "sponsor"
    #[serde(default)]
    pub role: Option<String>,
}

/// License information (v2)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct License {
    /// SPDX license identifier
    pub spdx: String,
    /// Path to license file
    #[serde(default)]
    pub file: Option<String>,
}

/// Registry metadata (v2)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegistryMetadata {
    /// Category: "productivity", "development", "integration", "experimental"
    #[serde(default)]
    pub category: Option<String>,
    /// Tags for search/filtering
    #[serde(default)]
    pub tags: Vec<String>,
    /// Lens homepage URL
    #[serde(default)]
    pub homepage: Option<String>,
    /// Source repository URL
    #[serde(default)]
    pub repository: Option<String>,
    /// Issue tracker URL
    #[serde(default)]
    pub issues: Option<String>,
    /// Path to icon file
    #[serde(default)]
    pub icon: Option<String>,
    /// Paths to screenshot files
    #[serde(default)]
    pub screenshots: Vec<String>,
}

/// MCP tool declaration (v2)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpTool {
    /// Tool name
    pub name: String,
    /// Tool description
    pub description: String,
    /// Path to JSON schema for input
    #[serde(default)]
    pub input_schema: Option<String>,
}

/// Entry point by mode (v2)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntryPoint {
    /// Mode: "ask", "plan-first", "code", "designer"
    pub mode: String,
    /// Entry point file path
    pub file: String,
    /// Path to system prompt template
    #[serde(default)]
    pub system_prompt: Option<String>,
}

/// Lifecycle hooks (v2)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LifecycleHooks {
    #[serde(default)]
    pub pre_install: Option<String>,
    #[serde(default)]
    pub post_install: Option<String>,
    #[serde(default)]
    pub pre_enable: Option<String>,
    #[serde(default)]
    pub post_enable: Option<String>,
    #[serde(default)]
    pub pre_disable: Option<String>,
    #[serde(default)]
    pub post_disable: Option<String>,
    #[serde(default)]
    pub pre_uninstall: Option<String>,
    #[serde(default)]
    pub post_uninstall: Option<String>,
}

/// Dependency with version constraint (v2 enhanced)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LensDependencyV2 {
    /// Lens ID
    pub id: String,
    /// Version requirement (semver range)
    #[serde(default)]
    pub version: Option<String>,
    /// Whether dependency is optional
    #[serde(default)]
    pub optional: bool,
}

/// Custom message type definition
///
/// Lenses emit custom messages via `LensEvent::Data { key, value }`.
/// The `key` field maps to a React component in the Component Registry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageType {
    /// Message type key (e.g., "search_results", "player", "component_preview")
    /// This matches the `key` field in `LensEvent::Data`
    pub key: String,

    /// Path to React component relative to lens root
    /// e.g., "components/SearchResults.tsx"
    pub component: String,

    /// Human-readable description of this message type
    #[serde(default)]
    pub description: String,

    /// Whether this message type requires user input (blocks execution)
    /// Examples: checkpoints, confirmations, questions
    #[serde(default)]
    pub interactive: bool,

    /// JSON schema for the message payload (optional, for validation)
    #[serde(default)]
    pub schema: Option<serde_json::Value>,
}

/// Lens shortcut definition.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct LensShortcut {
    /// Stable shortcut identifier within the lens (e.g., "launcher")
    pub id: String,
    /// Action key routed by the lens runtime (e.g., "launch")
    pub action: String,
    /// Accelerator combo in Tauri format (e.g., "CommandOrControl+Alt+Space")
    pub combo: String,
    /// Whether this shortcut should be global
    #[serde(default)]
    pub global: bool,
    /// Optional human description shown in UI
    #[serde(default)]
    pub description: Option<String>,
}

/// Dependency on another lens
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LensDependency {
    /// Lens ID
    pub id: String,

    /// Version requirement (semver range)
    #[serde(default)]
    pub version: Option<String>,
}

impl LensManifest {
    /// Parse manifest from TOML string
    pub fn from_toml(toml_str: &str) -> Result<Self, toml::de::Error> {
        toml::from_str(toml_str)
    }

    /// Serialize manifest to TOML string
    pub fn to_toml(&self) -> Result<String, toml::ser::Error> {
        toml::to_string_pretty(self)
    }

    /// Get message type by key
    pub fn get_message_type(&self, key: &str) -> Option<&MessageType> {
        self.message_types.iter().find(|mt| mt.key == key)
    }

    /// Check if a message type is interactive (requires user input)
    pub fn is_interactive(&self, key: &str) -> bool {
        self.get_message_type(key)
            .map(|mt| mt.interactive)
            .unwrap_or(false)
    }
}

impl Default for LensMetadata {
    fn default() -> Self {
        Self {
            id: String::new(),
            name: String::new(),
            version: "0.1.0".to_string(),
            description: String::new(),
            authors: Vec::new(),
            license: None,
            repository: None,
            min_framework_version: None,
            max_framework_version: None,
            manifest_version: 1,
            surface: LensSurface::Pane,
            surfaces: Vec::new(),
        }
    }
}

impl LensMetadata {
    /// Check if this lens supports a given surface type (T-LENS-SURFACE-001)
    pub fn supports_surface(&self, surface: &LensSurface) -> bool {
        if self.surfaces.is_empty() {
            // No explicit surfaces list — primary surface only
            &self.surface == surface
        } else {
            self.surfaces.iter().any(|s| s == surface)
        }
    }

    /// Get all supported surfaces. Falls back to primary surface if none declared.
    pub fn all_surfaces(&self) -> Vec<&LensSurface> {
        if self.surfaces.is_empty() {
            vec![&self.surface]
        } else {
            self.surfaces.iter().collect()
        }
    }
}

impl LensManifest {
    /// Check if this is a v2 manifest
    pub fn is_v2(&self) -> bool {
        self.lens.manifest_version >= 2
    }

    /// Get all authors (combines v1 and v2 formats)
    pub fn all_authors(&self) -> Vec<String> {
        if !self.authors.is_empty() {
            // v2 format
            self.authors.iter().map(|a| a.name.clone()).collect()
        } else {
            // v1 format
            self.lens.authors.clone()
        }
    }

    /// Get license SPDX identifier
    pub fn license_spdx(&self) -> Option<&str> {
        self.license_info
            .as_ref()
            .map(|l| l.spdx.as_str())
            .or(self.lens.license.as_deref())
    }

    /// Get MCP tool by name
    pub fn get_mcp_tool(&self, name: &str) -> Option<&McpTool> {
        self.mcp_tools.iter().find(|t| t.name == name)
    }

    /// Get entry point for mode
    pub fn get_entry_point(&self, mode: &str) -> Option<&EntryPoint> {
        self.entry_points.iter().find(|e| e.mode == mode)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_manifest() {
        let toml = r#"
[lens]
id = "graphyn-base"
name = "Knowledge Base"
version = "0.1.0"
description = "Local knowledge base with semantic search"
authors = ["Fuego Labs"]
license = "MIT"

[[message_types]]
key = "search_results"
component = "components/SearchResults.tsx"
description = "Displays search results"
interactive = false

[[message_types]]
key = "confirmation"
component = "components/Confirmation.tsx"
description = "Asks for user confirmation"
interactive = true
"#;
        let manifest = LensManifest::from_toml(toml).unwrap();

        assert_eq!(manifest.lens.id, "graphyn-base");
        assert_eq!(manifest.lens.name, "Knowledge Base");
        assert_eq!(manifest.message_types.len(), 2);

        assert!(!manifest.is_interactive("search_results"));
        assert!(manifest.is_interactive("confirmation"));
    }

    #[test]
    fn test_manifest_with_capabilities() {
        // In TOML, root-level arrays need to come BEFORE any [section]
        // Or we use inline tables. Using proper ordering here.
        let toml = r#"
# Root level first
capabilities = ["read", "write"]

[lens]
id = "test-lens"
name = "Test"
version = "0.1.0"

[[dependencies]]
id = "other-lens"
version = ">=1.0.0"
"#;
        let manifest = LensManifest::from_toml(toml).unwrap();
        assert_eq!(manifest.capabilities.len(), 2);
        assert_eq!(manifest.dependencies.len(), 1);
    }

    #[test]
    fn test_minimal_manifest() {
        let toml = r#"
[lens]
id = "minimal"
name = "Minimal Lens"
version = "0.1.0"
"#;
        let manifest = LensManifest::from_toml(toml).unwrap();

        assert_eq!(manifest.lens.id, "minimal");
        assert!(manifest.message_types.is_empty());
        assert!(manifest.capabilities.is_empty());
    }

    #[test]
    fn test_manifest_with_security() {
        let toml = r#"
[lens]
id = "secure-lens"
name = "Secure Lens"
version = "1.0.0"

[security]
library_hash = "sha256:abc123def456"
permissions = ["fs:read:~/Documents", "network:api.example.com"]
sandbox = "network"
"#;
        let manifest = LensManifest::from_toml(toml).unwrap();

        assert!(manifest.security.is_some());
        let security = manifest.security.unwrap();
        assert_eq!(
            security.library_hash,
            Some("sha256:abc123def456".to_string())
        );
        assert_eq!(security.permissions.len(), 2);
        assert_eq!(security.sandbox, SandboxLevel::Network);

        let parsed = security.parsed_permissions();
        assert_eq!(parsed.len(), 2);
        assert_eq!(parsed[0].permission_type, "fs");
        assert_eq!(parsed[0].action, Some("read".to_string()));
        assert_eq!(parsed[0].scope, "~/Documents");
    }

    #[test]
    fn test_permission_parsing() {
        // Two-part permission
        let perm = Permission::parse("network:api.example.com").unwrap();
        assert_eq!(perm.permission_type, "network");
        assert!(perm.action.is_none());
        assert_eq!(perm.scope, "api.example.com");

        // Three-part permission
        let perm = Permission::parse("fs:write:~/.graphyn/data").unwrap();
        assert_eq!(perm.permission_type, "fs");
        assert_eq!(perm.action, Some("write".to_string()));
        assert_eq!(perm.scope, "~/.graphyn/data");
    }

    #[test]
    fn test_security_hash_verification() {
        let security = SecurityConfig {
            library_hash: Some("sha256:abc123".to_string()),
            permissions: vec![],
            sandbox: SandboxLevel::Restricted,
        };

        assert!(security.verify_hash("sha256:abc123"));
        assert!(!security.verify_hash("sha256:wrong"));

        // No hash = always passes
        let no_hash = SecurityConfig {
            library_hash: None,
            permissions: vec![],
            sandbox: SandboxLevel::Restricted,
        };
        assert!(no_hash.verify_hash("anything"));
    }

    // === v2 Manifest Tests ===

    #[test]
    fn test_v2_manifest_detection() {
        let toml = r#"
[lens]
id = "com.example.v2-lens"
name = "V2 Test Lens"
version = "1.0.0"
description = "A v2 format test"
manifest_version = 2
min_framework_version = "0.5.0"

[[authors]]
name = "Test Author"
email = "test@example.com"
role = "maintainer"

[license]
spdx = "MIT"

[registry]
category = "productivity"
tags = ["test", "v2"]

[[mcp_tools]]
name = "test_tool"
description = "A test MCP tool"
"#;
        let manifest = LensManifest::from_toml(toml).unwrap();

        assert_eq!(manifest.lens.manifest_version, 2);
        assert!(manifest.is_v2());
        assert!(manifest.lens.min_framework_version.is_some());
        assert_eq!(manifest.lens.min_framework_version.unwrap(), "0.5.0");
    }

    #[test]
    fn test_v2_manifest_with_entry_points() {
        let toml = r#"
[lens]
id = "com.example.multi-mode"
name = "Multi Mode Lens"
version = "1.0.0"
manifest_version = 2
min_framework_version = "0.5.0"

[[entry_points]]
mode = "ask"
file = "dist/ask.js"

[[entry_points]]
mode = "code"
file = "dist/code.js"
system_prompt = "prompts/code.md"
"#;
        let manifest = LensManifest::from_toml(toml).unwrap();

        assert_eq!(manifest.entry_points.len(), 2);
        let ask_entry = manifest.get_entry_point("ask").unwrap();
        assert_eq!(ask_entry.file, "dist/ask.js");

        let code_entry = manifest.get_entry_point("code").unwrap();
        assert_eq!(
            code_entry.system_prompt,
            Some("prompts/code.md".to_string())
        );
    }

    #[test]
    fn test_v2_manifest_with_mcp_tools() {
        let toml = r#"
[lens]
id = "com.example.mcp-lens"
name = "MCP Lens"
version = "1.0.0"
manifest_version = 2
min_framework_version = "0.5.0"

[[mcp_tools]]
name = "search"
description = "Search for documents"
input_schema = "schemas/search.json"

[[mcp_tools]]
name = "index"
description = "Index a document"
"#;
        let manifest = LensManifest::from_toml(toml).unwrap();

        assert_eq!(manifest.mcp_tools.len(), 2);
        let search_tool = manifest.get_mcp_tool("search").unwrap();
        assert_eq!(search_tool.description, "Search for documents");
        assert_eq!(
            search_tool.input_schema,
            Some("schemas/search.json".to_string())
        );
    }

    #[test]
    fn test_v2_manifest_with_lifecycle_hooks() {
        let toml = r#"
[lens]
id = "com.example.hooked-lens"
name = "Hooked Lens"
version = "1.0.0"
manifest_version = 2
min_framework_version = "0.5.0"

[hooks]
post_install = "scripts/setup.sh"
pre_uninstall = "scripts/cleanup.sh"
"#;
        let manifest = LensManifest::from_toml(toml).unwrap();

        assert!(manifest.hooks.is_some());
        let hooks = manifest.hooks.unwrap();
        assert_eq!(hooks.post_install, Some("scripts/setup.sh".to_string()));
        assert_eq!(hooks.pre_uninstall, Some("scripts/cleanup.sh".to_string()));
    }

    #[test]
    fn test_v2_manifest_with_dependencies() {
        let toml = r#"
[lens]
id = "com.example.dep-lens"
name = "Dependent Lens"
version = "1.0.0"
manifest_version = 2
min_framework_version = "0.5.0"

[[dependencies_v2]]
id = "com.graphyn.base"
version = ">=1.0.0"
optional = false

[[dependencies_v2]]
id = "com.example.optional"
version = "^1.0.0"
optional = true
"#;
        let manifest = LensManifest::from_toml(toml).unwrap();

        assert_eq!(manifest.dependencies_v2.len(), 2);
        assert_eq!(manifest.dependencies_v2[0].id, "com.graphyn.base");
        assert!(!manifest.dependencies_v2[0].optional);
        assert!(manifest.dependencies_v2[1].optional);
    }

    #[test]
    fn test_v1_v2_backwards_compatibility() {
        // v1 manifest should still parse
        let toml = r#"
[lens]
id = "legacy-lens"
name = "Legacy Lens"
version = "0.1.0"
authors = ["Legacy Author"]
license = "MIT"
"#;
        let manifest = LensManifest::from_toml(toml).unwrap();

        assert_eq!(manifest.lens.manifest_version, 1); // defaults to 1
        assert!(!manifest.is_v2());
        assert_eq!(manifest.lens.authors, vec!["Legacy Author".to_string()]);
    }

    // === Multi-Surface Tests (T-LENS-SURFACE-001) ===

    #[test]
    fn test_multi_surface_manifest() {
        let toml = r#"
[lens]
id = "terminal"
name = "Terminal"
version = "0.2.0"
description = "Terminal — run commands in your project directory"
surface = "pane"
surfaces = ["pane", "pack"]
manifest_version = 2
"#;
        let manifest = LensManifest::from_toml(toml).unwrap();

        assert_eq!(manifest.lens.id, "terminal");
        assert_eq!(manifest.lens.surface, LensSurface::Pane);
        assert_eq!(manifest.lens.surfaces.len(), 2);
        assert_eq!(manifest.lens.surfaces[0], LensSurface::Pane);
        assert_eq!(manifest.lens.surfaces[1], LensSurface::Pack);

        // supports_surface checks
        assert!(manifest.lens.supports_surface(&LensSurface::Pane));
        assert!(manifest.lens.supports_surface(&LensSurface::Pack));
        assert!(!manifest.lens.supports_surface(&LensSurface::Tray));
        assert!(!manifest.lens.supports_surface(&LensSurface::DesktopApp));

        // all_surfaces returns declared list
        let all = manifest.lens.all_surfaces();
        assert_eq!(all.len(), 2);
    }

    #[test]
    fn test_single_surface_fallback() {
        let toml = r#"
[lens]
id = "figma"
name = "Figma"
version = "0.1.0"
surface = "desktop_app"
"#;
        let manifest = LensManifest::from_toml(toml).unwrap();

        // No surfaces array — only primary surface supported
        assert!(manifest.lens.surfaces.is_empty());
        assert!(manifest.lens.supports_surface(&LensSurface::DesktopApp));
        assert!(!manifest.lens.supports_surface(&LensSurface::Pane));

        // all_surfaces falls back to primary
        let all = manifest.lens.all_surfaces();
        assert_eq!(all.len(), 1);
        assert_eq!(*all[0], LensSurface::DesktopApp);
    }

    #[test]
    fn test_tray_surface_manifest() {
        let toml = r#"
[lens]
id = "quick"
name = "Quick"
version = "0.1.0"
surface = "tray"
surfaces = ["tray"]
"#;
        let manifest = LensManifest::from_toml(toml).unwrap();

        assert_eq!(manifest.lens.surface, LensSurface::Tray);
        assert!(manifest.lens.supports_surface(&LensSurface::Tray));
        assert!(!manifest.lens.supports_surface(&LensSurface::Pane));
        assert!(!manifest.lens.supports_surface(&LensSurface::DesktopApp));
    }

    #[test]
    fn test_shortcuts_manifest() {
        let toml = r#"
[lens]
id = "quick"
name = "Quick"
version = "0.1.0"

[[shortcuts]]
id = "launcher"
action = "launch"
combo = "CommandOrControl+Alt+Space"
global = true
description = "Open quick launcher"
"#;

        let manifest = LensManifest::from_toml(toml).unwrap();
        assert_eq!(manifest.shortcuts.len(), 1);
        let shortcut = &manifest.shortcuts[0];
        assert_eq!(shortcut.id, "launcher");
        assert_eq!(shortcut.action, "launch");
        assert_eq!(shortcut.combo, "CommandOrControl+Alt+Space");
        assert!(shortcut.global);
        assert_eq!(shortcut.description.as_deref(), Some("Open quick launcher"));
    }
}
