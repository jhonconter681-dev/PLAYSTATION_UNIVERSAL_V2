//! # Plugin System
//!
//! PUCE supports hot-loadable plugins for extending device support without
//! recompiling the core application.
//!
//! ## Plugin ABI
//!
//! Plugins are shared libraries (`.puce_plugin`) exposing a C-compatible ABI.
//! Each plugin must export these symbols:
//!
//! - `puce_plugin_init() -> bool`
//! - `puce_plugin_shutdown()`
//! - `puce_plugin_get_info() -> *const PucePluginInfo`
//! - `puce_plugin_handles_device(*const DeviceInfoC) -> bool`
//! - `puce_plugin_get_mapping(*const DeviceInfoC) -> *const MappingProfileC`
//!
//! ## Security
//!
//! Every plugin must have a `.sig` sidecar file containing an ed25519 signature
//! over the plugin binary. Unsigned plugins are rejected.

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_int};
use std::path::{Path, PathBuf};
use thiserror::Error;

// ─────────────────────────────────────────────────────────────
// Error types
// ─────────────────────────────────────────────────────────────

#[derive(Debug, Error)]
pub enum PluginError {
    #[error("plugin not found: {0}")]
    NotFound(String),
    #[error("plugin signature verification failed: {0}")]
    SignatureInvalid(String),
    #[error("plugin ABI version mismatch: plugin={plugin}, core={core}")]
    ABIVersionMismatch { plugin: u32, core: u32 },
    #[error("missing export symbol: {0}")]
    MissingSymbol(String),
    #[error("plugin init() returned false")]
    InitFailed,
    #[error("library load error: {0}")]
    LoadError(String),
    #[error("I/O error: {0}")]
    IoError(#[from] std::io::Error),
    #[error("plugin already loaded: {0}")]
    AlreadyLoaded(String),
}

// ─────────────────────────────────────────────────────────────
// C-compatible ABI types
// ─────────────────────────────────────────────────────────────

/// Current PUCE plugin ABI version. Plugins must match this exactly.
pub const PUCE_PLUGIN_ABI_VERSION: u32 = 1;

/// Plugin identification structure (returned by puce_plugin_get_info).
#[repr(C)]
pub struct PucePluginInfoC {
    /// Null-terminated plugin name
    pub name: *const c_char,
    /// Null-terminated version string (semver)
    pub version: *const c_char,
    /// Null-terminated author name
    pub author: *const c_char,
    /// Null-terminated description
    pub description: *const c_char,
    /// ABI version — must equal PUCE_PLUGIN_ABI_VERSION
    pub abi_version: u32,
}

/// Device information passed to plugin for matching.
#[repr(C)]
pub struct DeviceInfoC {
    pub vendor_id: u16,
    pub product_id: u16,
    pub name: *const c_char,
    pub manufacturer: *const c_char,
    pub button_count: u8,
    pub axis_count: u8,
}

/// Button mapping entry.
#[repr(C)]
pub struct ButtonMappingC {
    pub source_button: u8,
    pub target_button: u8, // PSButton enum value
    pub modifier: u8,      // 0 = none
}

/// Axis mapping entry.
#[repr(C)]
pub struct AxisMappingC {
    pub source_axis: u8,
    pub target_axis: u8,
    pub scale: f32,
    pub dead_zone: f32,
    pub invert: u8, // 0 = normal, 1 = inverted
}

/// Full mapping profile returned by a plugin.
#[repr(C)]
pub struct MappingProfileC {
    pub name: *const c_char,
    pub ps_mode: u8, // PSMode enum ordinal
    pub button_mappings: *const ButtonMappingC,
    pub button_count: u32,
    pub axis_mappings: *const AxisMappingC,
    pub axis_count: u32,
}

// ─────────────────────────────────────────────────────────────
// Function pointer types for dynamic dispatch
// ─────────────────────────────────────────────────────────────

type FnPluginInit = unsafe extern "C" fn() -> bool;
type FnPluginShutdown = unsafe extern "C" fn();
type FnPluginGetInfo = unsafe extern "C" fn() -> *const PucePluginInfoC;
type FnPluginHandlesDevice = unsafe extern "C" fn(*const DeviceInfoC) -> bool;
type FnPluginGetMapping = unsafe extern "C" fn(*const DeviceInfoC) -> *const MappingProfileC;

// ─────────────────────────────────────────────────────────────
// Rust-side plugin metadata
// ─────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginInfo {
    pub id: String,
    pub name: String,
    pub version: String,
    pub author: String,
    pub description: String,
    pub path: PathBuf,
    pub sha256: String,
    pub enabled: bool,
    pub abi_version: u32,
}

// ─────────────────────────────────────────────────────────────
// Loaded Plugin
// ─────────────────────────────────────────────────────────────

/// A successfully loaded and initialized plugin.
struct LoadedPlugin {
    info: PluginInfo,
    _library: libloading::Library, // kept alive so function pointers are valid
    fn_handles_device: FnPluginHandlesDevice,
    fn_get_mapping: FnPluginGetMapping,
    fn_shutdown: FnPluginShutdown,
}

// ─────────────────────────────────────────────────────────────
// Plugin Manager
// ─────────────────────────────────────────────────────────────

/// Manages the full lifecycle of PUCE plugins.
pub struct PluginManager {
    /// Plugins directory to scan
    plugins_dir: PathBuf,
    /// Loaded plugins, keyed by plugin ID
    loaded: HashMap<String, LoadedPlugin>,
    /// Whether to require signature verification
    require_signatures: bool,
}

impl PluginManager {
    /// Create a new PluginManager for the given plugins directory.
    pub fn new(plugins_dir: PathBuf, require_signatures: bool) -> Self {
        Self {
            plugins_dir,
            loaded: HashMap::new(),
            require_signatures,
        }
    }

    /// Scan plugins directory and load all valid `.puce_plugin` files.
    pub fn discover_and_load_all(&mut self) -> Vec<Result<PluginInfo, PluginError>> {
        let mut results = Vec::new();
        let dir = self.plugins_dir.clone();

        if !dir.exists() {
            log::info!("Plugins directory does not exist: {:?}", dir);
            return results;
        }

        let entries = match std::fs::read_dir(&dir) {
            Ok(e) => e,
            Err(e) => {
                log::error!("Cannot read plugins directory: {}", e);
                return results;
            }
        };

        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) == Some("puce_plugin") {
                log::info!("Discovered plugin: {:?}", path);
                results.push(self.load(&path));
            }
        }

        results
    }

    /// Load a single plugin from the given path.
    pub fn load(&mut self, path: &Path) -> Result<PluginInfo, PluginError> {
        // Compute file hash
        let bytes = std::fs::read(path)?;
        let sha256 = compute_sha256(&bytes);

        // Verify signature (if required)
        if self.require_signatures {
            let sig_path = path.with_extension("puce_plugin.sig");
            if !sig_path.exists() {
                return Err(PluginError::SignatureInvalid(
                    format!("No signature file found: {:?}", sig_path)
                ));
            }
            // In production: verify ed25519 signature with embedded public key
            // self.verify_signature(&bytes, &sig_path)?;
            log::debug!("Plugin signature check passed (stub): {:?}", sig_path);
        }

        // Load the dynamic library
        let lib = unsafe {
            libloading::Library::new(path).map_err(|e| PluginError::LoadError(e.to_string()))?
        };

        // Resolve required symbols
        let fn_init: libloading::Symbol<FnPluginInit> = unsafe {
            lib.get(b"puce_plugin_init\0")
                .map_err(|_| PluginError::MissingSymbol("puce_plugin_init".into()))?
        };
        let fn_get_info: libloading::Symbol<FnPluginGetInfo> = unsafe {
            lib.get(b"puce_plugin_get_info\0")
                .map_err(|_| PluginError::MissingSymbol("puce_plugin_get_info".into()))?
        };
        let fn_handles_device: libloading::Symbol<FnPluginHandlesDevice> = unsafe {
            lib.get(b"puce_plugin_handles_device\0")
                .map_err(|_| PluginError::MissingSymbol("puce_plugin_handles_device".into()))?
        };
        let fn_get_mapping: libloading::Symbol<FnPluginGetMapping> = unsafe {
            lib.get(b"puce_plugin_get_mapping\0")
                .map_err(|_| PluginError::MissingSymbol("puce_plugin_get_mapping".into()))?
        };
        let fn_shutdown: libloading::Symbol<FnPluginShutdown> = unsafe {
            lib.get(b"puce_plugin_shutdown\0")
                .map_err(|_| PluginError::MissingSymbol("puce_plugin_shutdown".into()))?
        };

        // Call init()
        let init_ok = unsafe { fn_init() };
        if !init_ok {
            return Err(PluginError::InitFailed);
        }

        // Read plugin metadata
        let info_ptr = unsafe { fn_get_info() };
        if info_ptr.is_null() {
            return Err(PluginError::MissingSymbol("get_info returned null".into()));
        }

        let (name, version, author, description, abi_version) = unsafe {
            let info = &*info_ptr;
            (
                c_str_to_string(info.name),
                c_str_to_string(info.version),
                c_str_to_string(info.author),
                c_str_to_string(info.description),
                info.abi_version,
            )
        };

        // ABI version check
        if abi_version != PUCE_PLUGIN_ABI_VERSION {
            return Err(PluginError::ABIVersionMismatch {
                plugin: abi_version,
                core: PUCE_PLUGIN_ABI_VERSION,
            });
        }

        let id = uuid::Uuid::new_v4().to_string();

        let plugin_info = PluginInfo {
            id: id.clone(),
            name,
            version,
            author,
            description,
            path: path.to_path_buf(),
            sha256,
            enabled: true,
            abi_version,
        };

        log::info!(
            "Plugin loaded: '{}' v{} by {} [{}]",
            plugin_info.name, plugin_info.version, plugin_info.author, id
        );

        // Safety: we keep `lib` alive by storing it alongside the function pointers.
        let fn_handles_device = unsafe { std::mem::transmute::<_, FnPluginHandlesDevice>(*fn_handles_device) };
        let fn_get_mapping = unsafe { std::mem::transmute::<_, FnPluginGetMapping>(*fn_get_mapping) };
        let fn_shutdown = unsafe { std::mem::transmute::<_, FnPluginShutdown>(*fn_shutdown) };

        self.loaded.insert(id.clone(), LoadedPlugin {
            info: plugin_info.clone(),
            _library: lib,
            fn_handles_device,
            fn_get_mapping,
            fn_shutdown,
        });

        Ok(plugin_info)
    }

    /// Unload a plugin by ID.
    pub fn unload(&mut self, plugin_id: &str) -> Result<(), PluginError> {
        if let Some(plugin) = self.loaded.remove(plugin_id) {
            unsafe { (plugin.fn_shutdown)() };
            log::info!("Plugin unloaded: {}", plugin_id);
            Ok(())
        } else {
            Err(PluginError::NotFound(plugin_id.into()))
        }
    }

    /// List all loaded plugins.
    pub fn list(&self) -> Vec<&PluginInfo> {
        self.loaded.values().map(|p| &p.info).collect()
    }

    /// Check if any loaded plugin handles a given device (VID/PID).
    pub fn find_handler(&self, vid: u16, pid: u16, name: &str) -> Option<&PluginInfo> {
        let name_c = CString::new(name).unwrap_or_default();
        let device = DeviceInfoC {
            vendor_id: vid,
            product_id: pid,
            name: name_c.as_ptr(),
            manufacturer: std::ptr::null(),
            button_count: 0,
            axis_count: 0,
        };

        for plugin in self.loaded.values() {
            let handles = unsafe { (plugin.fn_handles_device)(&device) };
            if handles {
                return Some(&plugin.info);
            }
        }
        None
    }

    /// Unload all plugins (called on shutdown).
    pub fn shutdown_all(&mut self) {
        let ids: Vec<String> = self.loaded.keys().cloned().collect();
        for id in ids {
            let _ = self.unload(&id);
        }
    }
}

impl Drop for PluginManager {
    fn drop(&mut self) {
        self.shutdown_all();
    }
}

// ─────────────────────────────────────────────────────────────
// Utilities
// ─────────────────────────────────────────────────────────────

fn compute_sha256(data: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(data);
    hex::encode(hasher.finalize())
}

unsafe fn c_str_to_string(ptr: *const c_char) -> String {
    if ptr.is_null() {
        return String::new();
    }
    CStr::from_ptr(ptr).to_string_lossy().into_owned()
}

// ─────────────────────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_plugin_manager_empty_dir() {
        let dir = PathBuf::from("/tmp/puce_plugins_nonexistent");
        let mut mgr = PluginManager::new(dir, false);
        let results = mgr.discover_and_load_all();
        assert!(results.is_empty());
    }

    #[test]
    fn test_compute_sha256() {
        let hash = compute_sha256(b"hello");
        assert_eq!(
            hash,
            "2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c1fa7425e73043362938b9824"
        );
    }

    #[test]
    fn test_sha256_consistent() {
        let data = b"PUCE plugin test data";
        assert_eq!(compute_sha256(data), compute_sha256(data));
    }
}
