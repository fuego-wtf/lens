//! # Lens Output Spec
//!
//! Declarative output contracts loaded from `lens.output.yaml`.
//! This file defines the payload shape and framework-owned render hints for each
//! lens output key.

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use std::collections::HashSet;
use std::path::Path;

use crate::error::{LensError, Result};

/// Standard output spec file name expected in each lens directory.
pub const OUTPUT_SPEC_FILENAME: &str = "lens.output.yaml";

/// Lens output specification parsed from `lens.output.yaml`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LensOutputSpec {
    /// Lens identifier. Must match lens.toml `[lens].id`.
    pub lens_id: String,

    /// Declarative output definitions for Data/Checkpoint rendering.
    #[serde(default)]
    pub outputs: Vec<OutputDefinition>,
}

/// One output definition for a specific output key.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutputDefinition {
    /// Stable output key (maps to LensEvent::Data key or Checkpoint phase).
    pub key: String,

    /// Human-readable title.
    pub title: String,

    /// Human-readable description.
    #[serde(default)]
    pub description: String,

    /// JSON-schema-like payload descriptor.
    #[serde(default = "default_payload_schema")]
    pub payload_schema: Value,

    /// Framework-owned render blocks.
    #[serde(default)]
    pub render_blocks: Vec<RenderBlock>,

    /// Whether this output needs user interaction.
    #[serde(default)]
    pub interactivity: InteractivityMode,

    /// Fields that must exist in the runtime payload object.
    #[serde(default)]
    pub required_fields: Vec<String>,

    /// Known error modes for this output.
    #[serde(default)]
    pub error_modes: Vec<OutputErrorMode>,

    /// Example payloads for testing and docs.
    #[serde(default)]
    pub examples: Vec<Value>,
}

/// A framework renderer block.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RenderBlock {
    /// Framework block type.
    #[serde(rename = "type")]
    pub block_type: RenderBlockType,

    /// Optional block title.
    #[serde(default)]
    pub title: Option<String>,

    /// Optional payload field path this block binds to.
    #[serde(default)]
    pub source: Option<String>,

    /// Optional renderer-specific options.
    #[serde(default)]
    pub options: Value,
}

/// Framework-owned block catalog.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum RenderBlockType {
    Header,
    KpiRow,
    Table,
    CardList,
    Timeline,
    Diff,
    JsonView,
    CheckpointGate,
    Notice,
    Actions,
}

/// Interaction mode for an output.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, Default, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum InteractivityMode {
    #[default]
    None,
    Confirm,
    Select,
    Form,
}

/// Structured error mode for a specific output.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutputErrorMode {
    pub code: String,
    pub message: String,
    #[serde(default)]
    pub recoverable: bool,
}

fn default_payload_schema() -> Value {
    Value::Object(Map::new())
}

impl LensOutputSpec {
    /// Parse output spec from YAML string.
    pub fn from_yaml(yaml: &str) -> Result<Self> {
        let spec: Self = serde_yaml::from_str(yaml).map_err(|e| {
            LensError::InvalidInput(format!("Failed to parse lens output spec YAML: {}", e))
        })?;
        spec.validate()?;
        Ok(spec)
    }

    /// Parse output spec from file path.
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let path = path.as_ref();
        let content = std::fs::read_to_string(path).map_err(|e| {
            LensError::InvalidInput(format!(
                "Failed to read output spec {:?}: {}",
                path, e
            ))
        })?;
        Self::from_yaml(&content)
    }

    /// Serialize output spec to YAML.
    pub fn to_yaml(&self) -> Result<String> {
        serde_yaml::to_string(self).map_err(|e| {
            LensError::InvalidInput(format!("Failed to serialize output spec to YAML: {}", e))
        })
    }

    /// Return an output definition by key.
    pub fn get_output(&self, key: &str) -> Option<&OutputDefinition> {
        self.outputs.iter().find(|o| o.key == key)
    }

    /// Validate high-level structural constraints.
    pub fn validate(&self) -> Result<()> {
        if self.lens_id.trim().is_empty() {
            return Err(LensError::InvalidInput(
                "lens.output.yaml: lens_id is required".to_string(),
            ));
        }

        if self.outputs.is_empty() {
            return Err(LensError::InvalidInput(format!(
                "lens.output.yaml for '{}' must declare at least one output",
                self.lens_id
            )));
        }

        let mut keys = HashSet::new();
        for output in &self.outputs {
            if output.key.trim().is_empty() {
                return Err(LensError::InvalidInput(format!(
                    "lens.output.yaml for '{}': output key cannot be empty",
                    self.lens_id
                )));
            }
            if !keys.insert(output.key.clone()) {
                return Err(LensError::InvalidInput(format!(
                    "lens.output.yaml for '{}': duplicate output key '{}'",
                    self.lens_id, output.key
                )));
            }
            output.validate(&self.lens_id)?;
        }

        Ok(())
    }

    /// Validate a runtime payload for a specific output key.
    pub fn validate_payload(&self, key: &str, payload: &Value) -> Result<()> {
        let output = self.get_output(key).ok_or_else(|| {
            LensError::InvalidInput(format!(
                "Output key '{}' is not declared in lens.output.yaml for '{}'",
                key, self.lens_id
            ))
        })?;

        if output.required_fields.is_empty() {
            return Ok(());
        }

        let obj = payload.as_object().ok_or_else(|| {
            LensError::InvalidInput(format!(
                "Output '{}' expects an object payload to validate required_fields",
                key
            ))
        })?;

        for required in &output.required_fields {
            if !obj.contains_key(required) {
                return Err(LensError::InvalidInput(format!(
                    "Output '{}' missing required field '{}'",
                    key, required
                )));
            }
        }

        Ok(())
    }
}

impl OutputDefinition {
    fn validate(&self, lens_id: &str) -> Result<()> {
        if self.title.trim().is_empty() {
            return Err(LensError::InvalidInput(format!(
                "lens.output.yaml for '{}': output '{}' must have a non-empty title",
                lens_id, self.key
            )));
        }

        if self.render_blocks.is_empty() {
            return Err(LensError::InvalidInput(format!(
                "lens.output.yaml for '{}': output '{}' must include at least one render block",
                lens_id, self.key
            )));
        }

        if self.examples.is_empty() {
            return Err(LensError::InvalidInput(format!(
                "lens.output.yaml for '{}': output '{}' must include at least one example payload",
                lens_id, self.key
            )));
        }

        for field in &self.required_fields {
            if field.trim().is_empty() {
                return Err(LensError::InvalidInput(format!(
                    "lens.output.yaml for '{}': output '{}' has an empty required_fields entry",
                    lens_id, self.key
                )));
            }
        }

        for error_mode in &self.error_modes {
            if error_mode.code.trim().is_empty() || error_mode.message.trim().is_empty() {
                return Err(LensError::InvalidInput(format!(
                    "lens.output.yaml for '{}': output '{}' has invalid error_modes entry",
                    lens_id, self.key
                )));
            }
        }

        // If payload schema declares object properties, ensure required fields are represented.
        if let Some(properties) = self
            .payload_schema
            .get("properties")
            .and_then(|p| p.as_object())
        {
            for field in &self.required_fields {
                if !properties.contains_key(field) {
                    return Err(LensError::InvalidInput(format!(
                        "lens.output.yaml for '{}': output '{}' required field '{}' is not declared in payload_schema.properties",
                        lens_id, self.key, field
                    )));
                }
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_parse_and_validate_output_spec() {
        let yaml = r#"
lens_id: figma
outputs:
  - key: phase_0_tokens
    title: Design Tokens
    description: Extracted tokens from design
    payload_schema:
      type: object
      properties:
        colors:
          type: array
    render_blocks:
      - type: header
        title: Tokens
      - type: table
        source: colors
    interactivity: none
    required_fields: [colors]
    error_modes:
      - code: INVALID_TOKEN
        message: Token extraction failed
        recoverable: true
    examples:
      - colors: []
"#;

        let spec = LensOutputSpec::from_yaml(yaml).unwrap();
        assert_eq!(spec.lens_id, "figma");
        assert_eq!(spec.outputs.len(), 1);
        assert_eq!(spec.outputs[0].key, "phase_0_tokens");
        assert_eq!(spec.outputs[0].render_blocks.len(), 2);
    }

    #[test]
    fn test_reject_duplicate_keys() {
        let yaml = r#"
lens_id: test
outputs:
  - key: duplicate
    title: One
    render_blocks:
      - type: notice
    examples:
      - ok: true
  - key: duplicate
    title: Two
    render_blocks:
      - type: notice
    examples:
      - ok: true
"#;
        let err = LensOutputSpec::from_yaml(yaml).unwrap_err();
        assert!(err.to_string().contains("duplicate output key"));
    }

    #[test]
    fn test_validate_payload_required_fields() {
        let yaml = r#"
lens_id: test
outputs:
  - key: result
    title: Result
    payload_schema:
      type: object
      properties:
        status:
          type: string
        data:
          type: object
    render_blocks:
      - type: json_view
    required_fields: [status]
    examples:
      - status: ok
        data: {}
"#;

        let spec = LensOutputSpec::from_yaml(yaml).unwrap();
        spec.validate_payload("result", &json!({"status":"ok","data":{}}))
            .unwrap();

        let err = spec
            .validate_payload("result", &json!({"data":{}}))
            .unwrap_err();
        assert!(err.to_string().contains("missing required field"));
    }

    #[test]
    fn test_reject_missing_examples() {
        let yaml = r#"
lens_id: test
outputs:
  - key: no_examples
    title: No Examples
    render_blocks:
      - type: notice
"#;
        let err = LensOutputSpec::from_yaml(yaml).unwrap_err();
        assert!(err.to_string().contains("must include at least one example payload"));
    }
}
