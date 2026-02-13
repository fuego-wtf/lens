//! # Dynamic Lens Loader
//!
//! Load compiled Lenses from shared libraries (.dylib, .so, .dll).
//!
//! Requires the `runtime` feature.
//!
//! Lenses expose a C ABI entry point that returns a boxed Lens trait object:
//!
//! ```rust,ignore
//! use lens::export_lens;
//!
//! struct MyLens;
//! // impl Lens for MyLens { ... }
//!
//! export_lens!(MyLens::new());
//! ```

use std::ffi::OsStr;
use std::path::Path;
use std::sync::Arc;

use libloading::{Library, Symbol};

use crate::error::{LensError, Result};
use crate::lens::Lens;

/// Function signature for lens entry point
#[allow(improper_ctypes_definitions)]
type CreateLensFn = unsafe extern "C" fn() -> *mut dyn Lens;

/// Entry point function name that lenses must export
pub const LENS_ENTRY_POINT: &[u8] = b"create_lens";

/// A loaded lens with its library handle
pub struct LoadedLens {
    /// The lens instance
    lens: Box<dyn Lens>,
    /// Library handle (must be kept alive while lens is in use)
    _library: Arc<Library>,
}

impl std::fmt::Debug for LoadedLens {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("LoadedLens")
            .field("id", &self.lens.id())
            .field("name", &self.lens.name())
            .field("version", &self.lens.version())
            .finish()
    }
}

impl LoadedLens {
    /// Get reference to the lens
    pub fn plugin(&self) -> &dyn Lens {
        self.lens.as_ref()
    }

    /// Get mutable reference to the lens
    pub fn plugin_mut(&mut self) -> &mut dyn Lens {
        self.lens.as_mut()
    }

    /// Get the lens ID
    pub fn id(&self) -> &str {
        self.lens.id()
    }

    /// Get the lens name
    pub fn name(&self) -> &str {
        self.lens.name()
    }

    /// Get the lens version
    pub fn version(&self) -> &str {
        self.lens.version()
    }
}

/// Dynamic lens loader
#[derive(Default)]
pub struct LensLoader {
    /// Loaded libraries (kept alive to prevent unloading)
    libraries: Vec<Arc<Library>>,
}

impl LensLoader {
    /// Create a new lens loader
    pub fn new() -> Self {
        Self {
            libraries: Vec::new(),
        }
    }

    /// Load a lens from a shared library path
    ///
    /// # Safety
    ///
    /// This function is unsafe because:
    /// - It loads arbitrary code from the filesystem
    /// - The lens must be compiled with a compatible Rust version
    /// - The lens must properly implement the create_lens function
    ///
    /// Only load lenses from trusted sources.
    pub unsafe fn load<P: AsRef<OsStr>>(&mut self, library_path: P) -> Result<LoadedLens> {
        let path = library_path.as_ref();
        let path_buf = Path::new(path).to_path_buf();

        if !path_buf.exists() {
            return Err(LensError::LensNotFound(format!(
                "Library not found: {:?}",
                path_buf
            )));
        }

        let library = Library::new(path).map_err(|e| {
            LensError::Initialization(format!("Failed to load library {:?}: {}", path_buf, e))
        })?;

        let library = Arc::new(library);

        let create_lens: Symbol<CreateLensFn> =
            library.get(LENS_ENTRY_POINT).map_err(|e| {
                LensError::Initialization(format!(
                    "Lens {:?} missing 'create_lens' entry point: {}",
                    path_buf, e
                ))
            })?;

        let lens_ptr = create_lens();

        if lens_ptr.is_null() {
            return Err(LensError::Initialization(format!(
                "Lens {:?} returned null from create_lens",
                path_buf
            )));
        }

        let lens = Box::from_raw(lens_ptr);
        self.libraries.push(Arc::clone(&library));

        Ok(LoadedLens {
            lens,
            _library: library,
        })
    }

    /// Load a lens and return an Arc for shared ownership
    ///
    /// # Safety
    ///
    /// Same safety requirements as `load`.
    pub unsafe fn load_shared<P: AsRef<OsStr>>(
        &mut self,
        library_path: P,
    ) -> Result<Arc<LoadedLens>> {
        Ok(Arc::new(self.load(library_path)?))
    }

    /// Get the number of loaded libraries
    pub fn loaded_count(&self) -> usize {
        self.libraries.len()
    }
}

/// Macro to generate the lens entry point
///
/// # Example
///
/// ```rust,ignore
/// use lens::export_lens;
///
/// struct MyLens { /* ... */ }
/// impl Lens for MyLens { /* ... */ }
///
/// export_lens!(MyLens::new());
/// ```
#[macro_export]
macro_rules! export_lens {
    ($constructor:expr) => {
        #[no_mangle]
        pub extern "C" fn create_lens() -> *mut dyn $crate::Lens {
            let lens: Box<dyn $crate::Lens> = Box::new($constructor);
            Box::into_raw(lens)
        }
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_loader_new() {
        let loader = LensLoader::new();
        assert_eq!(loader.loaded_count(), 0);
    }

    #[test]
    fn test_load_nonexistent_library() {
        let mut loader = LensLoader::new();
        let result = unsafe { loader.load("/nonexistent/path/liblens.dylib") };
        assert!(result.is_err());
        match result.unwrap_err() {
            LensError::LensNotFound(_) => {}
            other => panic!("Expected LensNotFound, got {:?}", other),
        }
    }
}
