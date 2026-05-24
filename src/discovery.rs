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
use crate::manifest::{LensManifest, LensSurface};
use crate::output_spec::{LensOutputSpec, OUTPUT_SPEC_FILENAME};

/// Manifest filename
pub const MANIFEST_FILENAME: &str = "lens.toml";
pub const LEGACY_MANIFEST_FILENAME: &str = "plugin.toml";
pub const LENS_URI_PREFIX: &str = "lens:";

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

    /// Get the surface type from manifest (T-LENS-SURFACE-001)
    pub fn surface(&self) -> LensSurface {
        self.manifest.lens.surface.clone()
    }

    /// Check if this lens supports a given surface type (T-LENS-SURFACE-001 multi-surface)
    pub fn supports_surface(&self, surface: &LensSurface) -> bool {
        self.manifest.lens.supports_surface(surface)
    }

    /// Get all supported surfaces (T-LENS-SURFACE-001)
    pub fn all_surfaces(&self) -> Vec<LensSurface> {
        self.manifest
            .lens
            .all_surfaces()
            .into_iter()
            .cloned()
            .collect()
    }

    /// Stable launch URI for this discovered lens.
    pub fn launch_uri(&self) -> String {
        format!("{}{}", LENS_URI_PREFIX, self.id())
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

        let mut lens_dirs = Vec::new();

        for entry in entries {
            let entry = entry.map_err(|e| {
                LensError::Initialization(format!("Failed to read directory entry: {}", e))
            })?;

            let path = entry.path();
            if !path.is_dir() {
                continue;
            }

            lens_dirs.push(path);
        }

        lens_dirs.sort();

        for path in lens_dirs {
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

        discovered.sort_by(|a, b| {
            a.id()
                .cmp(b.id())
                .then_with(|| a.manifest_path.cmp(&b.manifest_path))
        });

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
            LensError::InvalidInput(format!(
                "Failed to read manifest {:?}: {}",
                manifest_path, e
            ))
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

    /// Resolve a `lens:<id>` launch URI to a discovered manifest.
    ///
    /// This is a source-side preflight for host runtimes. It proves the launch
    /// URI maps to exactly one installed lens manifest before a desktop host
    /// creates any window or Agent Client Protocol session.
    pub fn resolve_lens_uri(&self, launch_uri: &str) -> Result<DiscoveredLens> {
        let lens_id = parse_lens_uri(launch_uri)?;
        let matches: Vec<DiscoveredLens> = self
            .scan()?
            .into_iter()
            .filter(|lens| lens.id() == lens_id)
            .collect();

        match matches.len() {
            0 => Err(LensError::LensNotFound(format!(
                "{}{}",
                LENS_URI_PREFIX, lens_id
            ))),
            1 => Ok(matches.into_iter().next().expect("one match")),
            _ => Err(LensError::InvalidInput(format!(
                "Launch URI '{}{}' matched {} installed manifests; lens IDs must be unique",
                LENS_URI_PREFIX,
                lens_id,
                matches.len()
            ))),
        }
    }

    /// Resolve a `lens:<id>` launch URI and verify the manifest supports a surface.
    pub fn resolve_lens_uri_for_surface(
        &self,
        launch_uri: &str,
        surface: &LensSurface,
    ) -> Result<DiscoveredLens> {
        let lens = self.resolve_lens_uri(launch_uri)?;
        if lens.supports_surface(surface) {
            return Ok(lens);
        }

        Err(LensError::InvalidInput(format!(
            "Launch URI '{}' resolved to lens '{}' but manifest surfaces {:?} do not support {:?}",
            launch_uri,
            lens.id(),
            lens.all_surfaces(),
            surface
        )))
    }
}

/// Parse a runtime launch URI in the form `lens:<id>`.
pub fn parse_lens_uri(launch_uri: &str) -> Result<&str> {
    let lens_id = launch_uri.strip_prefix(LENS_URI_PREFIX).ok_or_else(|| {
        LensError::InvalidInput(format!("Expected lens:<id>, got '{}'", launch_uri))
    })?;

    if lens_id.is_empty() {
        return Err(LensError::InvalidInput(
            "Expected lens:<id> with a non-empty lens id".to_string(),
        ));
    }

    if lens_id.trim() != lens_id {
        return Err(LensError::InvalidInput(format!(
            "Lens launch id '{}' must not contain surrounding whitespace",
            lens_id
        )));
    }

    if lens_id.contains('/') || lens_id.contains('\\') {
        return Err(LensError::InvalidInput(format!(
            "Lens launch id '{}' must not contain path separators",
            lens_id
        )));
    }

    Ok(lens_id)
}

/// Load a manifest from a directory path (looks for lens.toml in the dir)
pub fn load_manifest<P: AsRef<Path>>(path: P) -> Result<LensManifest> {
    let path = path.as_ref();

    // If path points to a directory, look for the manifest inside it
    let manifest_path = if path.is_dir() {
        let primary = path.join(MANIFEST_FILENAME);
        if primary.exists() {
            primary
        } else {
            path.join(LEGACY_MANIFEST_FILENAME)
        }
    } else {
        path.to_path_buf()
    };

    let content = std::fs::read_to_string(&manifest_path).map_err(|e| {
        LensError::InvalidInput(format!(
            "Failed to read manifest {:?}: {}",
            manifest_path, e
        ))
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

    fn create_test_lens_with_manifest(dir: &Path, dir_name: &str, manifest: &str) {
        let lens_dir = dir.join(dir_name);
        fs::create_dir_all(&lens_dir).unwrap();
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
    fn test_scan_returns_lenses_in_deterministic_id_order() {
        let temp_dir = tempdir().unwrap();
        create_test_lens(temp_dir.path(), "zeta", "Zeta Lens");
        create_test_lens(temp_dir.path(), "alpha", "Alpha Lens");
        create_test_lens(temp_dir.path(), "middle", "Middle Lens");

        let discovery = LensDiscovery::new(temp_dir.path());
        let lenses = discovery.scan().unwrap();
        let ids: Vec<&str> = lenses.iter().map(|lens| lens.id()).collect();

        assert_eq!(ids, vec!["alpha", "middle", "zeta"]);
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
    fn test_resolve_lens_uri_returns_matching_manifest() {
        let temp_dir = tempdir().unwrap();
        create_test_lens(temp_dir.path(), "quick", "Quick");

        let discovery = LensDiscovery::new(temp_dir.path());
        let lens = discovery.resolve_lens_uri("lens:quick").unwrap();

        assert_eq!(lens.id(), "quick");
        assert_eq!(lens.name(), "Quick");
        assert_eq!(lens.launch_uri(), "lens:quick");
        assert_eq!(
            lens.manifest_path,
            temp_dir.path().join("quick").join(MANIFEST_FILENAME)
        );
    }

    #[test]
    fn test_resolve_lens_uri_rejects_duplicate_manifest_ids() {
        let temp_dir = tempdir().unwrap();
        let manifest = r#"
[lens]
id = "quick"
name = "Quick"
version = "1.0.0"
"#;
        create_test_lens_with_manifest(temp_dir.path(), "quick-a", manifest);
        create_test_lens_with_manifest(temp_dir.path(), "quick-b", manifest);

        let discovery = LensDiscovery::new(temp_dir.path());
        let error = discovery.resolve_lens_uri("lens:quick").unwrap_err();

        assert!(matches!(error, LensError::InvalidInput(_)));
        assert!(error.to_string().contains("matched 2 installed manifests"));
    }

    #[test]
    fn test_resolve_lens_uri_for_surface_validates_manifest_surface() {
        let temp_dir = tempdir().unwrap();
        let manifest = r#"
[lens]
id = "quick"
name = "Quick"
version = "1.0.0"
surface = "tray"
surfaces = ["tray"]
"#;
        create_test_lens_with_manifest(temp_dir.path(), "quick", manifest);

        let discovery = LensDiscovery::new(temp_dir.path());
        let lens = discovery
            .resolve_lens_uri_for_surface("lens:quick", &LensSurface::Tray)
            .unwrap();

        assert_eq!(lens.id(), "quick");

        let error = discovery
            .resolve_lens_uri_for_surface("lens:quick", &LensSurface::DesktopApp)
            .unwrap_err();
        assert!(matches!(error, LensError::InvalidInput(_)));
        assert!(error.to_string().contains("do not support DesktopApp"));
    }

    #[test]
    fn test_parse_lens_uri_rejects_invalid_launch_ids() {
        let invalid_prefix = parse_lens_uri("http:quick").unwrap_err();
        assert!(invalid_prefix.to_string().contains("Expected lens:<id>"));

        let empty_id = parse_lens_uri("lens:").unwrap_err();
        assert!(empty_id.to_string().contains("non-empty lens id"));

        let padded_id = parse_lens_uri("lens: quick").unwrap_err();
        assert!(padded_id.to_string().contains("surrounding whitespace"));

        let path_id = parse_lens_uri("lens:../quick").unwrap_err();
        assert!(path_id.to_string().contains("path separators"));
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
