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

    /// Dependencies on other lenses (optional)
    #[serde(default)]
    pub dependencies: Vec<LensDependency>,
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

    /// Lens author(s)
    #[serde(default)]
    pub authors: Vec<String>,

    /// Lens license
    #[serde(default)]
    pub license: Option<String>,

    /// Lens homepage/repository URL
    #[serde(default)]
    pub repository: Option<String>,

    /// Minimum framework version required
    #[serde(default)]
    pub min_framework_version: Option<String>,
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
        }
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
        assert_eq!(security.library_hash, Some("sha256:abc123def456".to_string()));
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
}
