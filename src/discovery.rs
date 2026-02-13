//! # Lens Discovery
//!
//! Discover installed Lenses from the filesystem.
//!
//! Requires the `runtime` feature.
//!
//! # Directory Structure
//!
//! ```text
//! ~/.graphyn/lenses/
//! ├── figma/
//! │   ├── lens.toml           # Manifest
//! │   ├── lens.output.yaml    # Declarative output contract
//! │   └── libfigma.dylib      # Compiled lens (macOS)
//! └── vibe/
//!     ├── lens.toml
//!     └── libvibe.dylib
//! ```

use std::path::{Path, PathBuf};

use crate::error::{LensError, Result};
use crate::manifest::LensManifest;
use crate::output_spec::{LensOutputSpec, OUTPUT_SPEC_FILENAME};

/// Manifest filename
pub const MANIFEST_FILENAME: &str = "lens.toml";

/// Default lenses directory name
pub const LENS_DIR: &str = "lenses";

/// A discovered Lens with its manifest and location
#[derive(Debug, Clone)]
pub struct DiscoveredLens {
    /// Parsed lens manifest
    pub manifest: LensManifest,

    /// Path to the lens directory
    pub path: PathBuf,

    /// Path to the manifest file
    pub manifest_path: PathBuf,

    /// Path to lens.output.yaml (if exists)
    pub output_spec_path: Option<PathBuf>,

    /// Parsed lens.output.yaml (if exists)
    pub output_spec: Option<LensOutputSpec>,

    /// Path to the compiled lens library (if exists)
    pub library_path: Option<PathBuf>,
}

impl DiscoveredLens {
    /// Get the lens ID from manifest
    pub fn id(&self) -> &str {
        &self.manifest.lens.id
    }

    /// Get the lens name from manifest
    pub fn name(&self) -> &str {
        &self.manifest.lens.name
    }

    /// Get the lens version from manifest
    pub fn version(&self) -> &str {
        &self.manifest.lens.version
    }

    /// Get path to a component file
    pub fn component_path(&self, component: &str) -> PathBuf {
        self.path.join(component)
    }
}

/// Lens discovery service
#[derive(Debug, Clone)]
pub struct LensDiscovery {
    /// Base directory to scan for lenses
    lenses_dir: PathBuf,
}

impl LensDiscovery {
    /// Create a new discovery instance for the given directory
    pub fn new<P: AsRef<Path>>(lenses_dir: P) -> Self {
        Self {
            lenses_dir: lenses_dir.as_ref().to_path_buf(),
        }
    }

    /// Create discovery for the default ~/.graphyn/lenses directory
    pub fn default_directory() -> Result<Self> {
        let home = dirs::home_dir().ok_or_else(|| {
            LensError::Initialization("Could not determine home directory".to_string())
        })?;

        let lenses_dir = home.join(".graphyn").join(LENS_DIR);
        Ok(Self::new(lenses_dir))
    }

    /// Create discovery for a custom graphyn directory
    pub fn for_graphyn_dir<P: AsRef<Path>>(graphyn_dir: P) -> Self {
        Self::new(graphyn_dir.as_ref().join(LENS_DIR))
    }

    /// Get the lenses directory path
    pub fn plugins_dir(&self) -> &Path {
        &self.lenses_dir
    }

    /// Check if the lenses directory exists
    pub fn exists(&self) -> bool {
        self.lenses_dir.exists() && self.lenses_dir.is_dir()
    }

    /// Ensure the lenses directory exists, creating it if needed
    pub fn ensure_exists(&self) -> Result<()> {
        if !self.lenses_dir.exists() {
            std::fs::create_dir_all(&self.lenses_dir).map_err(|e| {
                LensError::Initialization(format!(
                    "Failed to create lenses directory {:?}: {}",
                    self.lenses_dir, e
                ))
            })?;
        }
        Ok(())
    }

    /// Scan the lenses directory and discover all lenses
    pub fn scan(&self) -> Result<Vec<DiscoveredLens>> {
        self.scan_with_options(false)
    }

    /// Scan with configurable validation policies
    pub fn scan_with_options(&self, require_output_spec: bool) -> Result<Vec<DiscoveredLens>> {
        if !self.exists() {
            return Ok(Vec::new());
        }

        let mut discovered = Vec::new();

        let entries = std::fs::read_dir(&self.lenses_dir).map_err(|e| {
            LensError::Initialization(format!(
                "Failed to read lenses directory {:?}: {}",
                self.lenses_dir, e
            ))
        })?;

        for entry in entries {
            let entry = entry.map_err(|e| {
                LensError::Initialization(format!("Failed to read directory entry: {}", e))
            })?;

            let path = entry.path();
            if !path.is_dir() {
                continue;
            }

            match self.load_lens(&path) {
                Ok(lens) => {
                    if require_output_spec && lens.output_spec.is_none() {
                        eprintln!(
                            "Warning: Lens at {:?} missing {}",
                            path, OUTPUT_SPEC_FILENAME
                        );
                        continue;
                    }
                    discovered.push(lens);
                }
                Err(e) => {
                    eprintln!("Warning: Failed to load lens from {:?}: {}", path, e);
                }
            }
        }

        Ok(discovered)
    }

    /// Load a single lens from a directory
    pub fn load_lens<P: AsRef<Path>>(&self, lens_dir: P) -> Result<DiscoveredLens> {
        let lens_dir = lens_dir.as_ref();
        let manifest_path = lens_dir.join(MANIFEST_FILENAME);

        if !manifest_path.exists() {
            return Err(LensError::InvalidInput(format!(
                "No {} found in {:?}",
                MANIFEST_FILENAME, lens_dir
            )));
        }

        let manifest_content = std::fs::read_to_string(&manifest_path).map_err(|e| {
            LensError::InvalidInput(format!("Failed to read manifest {:?}: {}", manifest_path, e))
        })?;

        let manifest = LensManifest::from_toml(&manifest_content).map_err(|e| {
            LensError::InvalidInput(format!(
                "Failed to parse manifest {:?}: {}",
                manifest_path, e
            ))
        })?;

        // Load output spec if present
        let output_spec_path_candidate = lens_dir.join(OUTPUT_SPEC_FILENAME);
        let (output_spec_path, output_spec) = if output_spec_path_candidate.exists() {
            let spec = load_output_spec(&output_spec_path_candidate).map_err(|e| {
                LensError::InvalidInput(format!(
                    "Failed to load output spec {:?}: {}",
                    output_spec_path_candidate, e
                ))
            })?;
            (Some(output_spec_path_candidate), Some(spec))
        } else {
            (None, None)
        };

        // Look for compiled library
        let library_path = self.find_library(lens_dir, &manifest.lens.id);

        Ok(DiscoveredLens {
            manifest,
            path: lens_dir.to_path_buf(),
            manifest_path,
            output_spec_path,
            output_spec,
            library_path,
        })
    }

    /// Find the compiled library for a lens
    fn find_library(&self, lens_dir: &Path, lens_id: &str) -> Option<PathBuf> {
        let lib_name = lens_id.replace('-', "_");

        let extensions = if cfg!(target_os = "macos") {
            &["dylib", "so"][..]
        } else if cfg!(target_os = "windows") {
            &["dll"][..]
        } else {
            &["so"][..]
        };

        for ext in extensions {
            let lib_path = lens_dir.join(format!("lib{}.{}", lib_name, ext));
            if lib_path.exists() {
                return Some(lib_path);
            }
            let lib_path = lens_dir.join(format!("{}.{}", lib_name, ext));
            if lib_path.exists() {
                return Some(lib_path);
            }
        }

        // Check target/release (development mode)
        let target_release = lens_dir.join("target").join("release");
        if target_release.exists() {
            for ext in extensions {
                let lib_path = target_release.join(format!("lib{}.{}", lib_name, ext));
                if lib_path.exists() {
                    return Some(lib_path);
                }
            }
        }

        None
    }

    /// Get a specific lens by ID
    pub fn get_lens(&self, lens_id: &str) -> Result<Option<DiscoveredLens>> {
        let lenses = self.scan()?;
        Ok(lenses.into_iter().find(|l| l.id() == lens_id))
    }

    /// Check if a lens is installed
    pub fn is_installed(&self, lens_id: &str) -> Result<bool> {
        Ok(self.get_lens(lens_id)?.is_some())
    }
}

/// Load a manifest from a directory path (looks for lens.toml in the dir)
pub fn load_manifest<P: AsRef<Path>>(path: P) -> Result<LensManifest> {
    let path = path.as_ref();

    // If path points to a directory, look for the manifest inside it
    let manifest_path = if path.is_dir() {
        path.join(MANIFEST_FILENAME)
    } else {
        path.to_path_buf()
    };

    let content = std::fs::read_to_string(&manifest_path).map_err(|e| {
        LensError::InvalidInput(format!("Failed to read manifest {:?}: {}", manifest_path, e))
    })?;

    LensManifest::from_toml(&content)
        .map_err(|e| LensError::InvalidInput(format!("Failed to parse manifest: {}", e)))
}

/// Load a lens output spec from YAML file
pub fn load_output_spec<P: AsRef<Path>>(path: P) -> Result<LensOutputSpec> {
    LensOutputSpec::from_file(path)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    fn create_test_lens(dir: &Path, id: &str, name: &str) {
        let lens_dir = dir.join(id);
        fs::create_dir_all(&lens_dir).unwrap();

        let manifest = format!(
            r#"
[lens]
id = "{}"
name = "{}"
version = "1.0.0"
description = "Test lens"

[[message_types]]
key = "result"
component = "components/Result.tsx"
"#,
            id, name
        );

        fs::write(lens_dir.join(MANIFEST_FILENAME), manifest).unwrap();
    }

    #[test]
    fn test_scan_empty_directory() {
        let temp_dir = tempdir().unwrap();
        let discovery = LensDiscovery::new(temp_dir.path());
        let lenses = discovery.scan().unwrap();
        assert!(lenses.is_empty());
    }

    #[test]
    fn test_scan_nonexistent_directory() {
        let discovery = LensDiscovery::new("/nonexistent/path/to/lenses");
        let lenses = discovery.scan().unwrap();
        assert!(lenses.is_empty());
    }

    #[test]
    fn test_scan_with_lenses() {
        let temp_dir = tempdir().unwrap();
        create_test_lens(temp_dir.path(), "figma", "Figma Decomposer");
        create_test_lens(temp_dir.path(), "base", "Knowledge Base");

        let discovery = LensDiscovery::new(temp_dir.path());
        let lenses = discovery.scan().unwrap();
        assert_eq!(lenses.len(), 2);

        let ids: Vec<&str> = lenses.iter().map(|l| l.id()).collect();
        assert!(ids.contains(&"figma"));
        assert!(ids.contains(&"base"));
    }

    #[test]
    fn test_load_single_lens() {
        let temp_dir = tempdir().unwrap();
        create_test_lens(temp_dir.path(), "figma", "Figma Decomposer");

        let discovery = LensDiscovery::new(temp_dir.path());
        let lens = discovery.load_lens(temp_dir.path().join("figma")).unwrap();

        assert_eq!(lens.id(), "figma");
        assert_eq!(lens.name(), "Figma Decomposer");
        assert_eq!(lens.version(), "1.0.0");
    }

    #[test]
    fn test_get_lens_by_id() {
        let temp_dir = tempdir().unwrap();
        create_test_lens(temp_dir.path(), "figma", "Figma Decomposer");

        let discovery = LensDiscovery::new(temp_dir.path());
        let figma = discovery.get_lens("figma").unwrap();
        assert!(figma.is_some());

        let nonexistent = discovery.get_lens("nonexistent").unwrap();
        assert!(nonexistent.is_none());
    }

    #[test]
    fn test_ensure_exists() {
        let temp_dir = tempdir().unwrap();
        let lenses_dir = temp_dir.path().join("new_lenses");

        let discovery = LensDiscovery::new(&lenses_dir);
        assert!(!discovery.exists());

        discovery.ensure_exists().unwrap();
        assert!(discovery.exists());
    }

    #[test]
    fn test_load_manifest_from_dir() {
        let temp_dir = tempdir().unwrap();
        create_test_lens(temp_dir.path(), "test", "Test Lens");

        let manifest = load_manifest(temp_dir.path().join("test")).unwrap();
        assert_eq!(manifest.lens.id, "test");
    }
}
