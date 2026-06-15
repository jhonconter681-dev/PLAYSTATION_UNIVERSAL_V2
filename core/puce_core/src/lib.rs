//! # puce_core — Public C FFI API
//!
//! This crate exposes a `#[no_mangle] pub extern "C"` surface that can be
//! consumed from C, C++, Python (ctypes/cffi), C#  (P/Invoke), Java (JNA),
//! or any other language with a foreign-function interface.
//!
//! All strings returned from functions are **heap-allocated**.  Callers MUST
//! call [`puce_free_string`] when they are done with the pointer, otherwise
//! they will leak memory.
//!
//! ## Thread-safety
//! The global engine state is protected by a `std::sync::Mutex`.  Calls from
//! multiple threads are safe; they will serialize on the lock.

use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_int};
use std::sync::{Mutex, OnceLock};

use anyhow::Context;
use log::{error, info};
use serde_json::json;

use detection::DetectionEngine;
use emulation::{EmulationEngine, PSMode};
use mapping::MappingEngine;
use plugin_system::PluginManager;
use database::Database;
use virtual_controller::create_virtual_controller;

// ─── Version constants ───────────────────────────────────────────────────────

/// Semantic version of the puce_core library.
const PUCE_VERSION: &str = "0.1.0";

// ─── Global engine state ─────────────────────────────────────────────────────

/// All runtime state is bundled into a single struct and stored behind a
/// `Mutex<Option<_>>` so we can lazily initialise and later cleanly shut down.
struct PuceState {
    detection:  DetectionEngine,
    mapping:    MappingEngine,
    emulation:  EmulationEngine,
    plugins:    PluginManager,
    database:   Database,
    running:    bool,
}

static PUCE_STATE: OnceLock<Mutex<Option<PuceState>>> = OnceLock::new();

fn global_state() -> &'static Mutex<Option<PuceState>> {
    PUCE_STATE.get_or_init(|| Mutex::new(None))
}

// ─── Helper: convert a Rust &str to an owned C string ptr ────────────────────

/// Converts a Rust string into a `*const c_char` that the caller owns.
/// Returns a null pointer on interior-nul errors (should never happen for
/// well-formed input).
fn to_c_string(s: &str) -> *const c_char {
    match CString::new(s) {
        Ok(cs) => cs.into_raw() as *const c_char,
        Err(e) => {
            error!("to_c_string: interior nul in string: {e}");
            std::ptr::null()
        }
    }
}

/// Converts an `Option<String>` — returns empty-string ptr on `None`.
fn to_c_string_opt(s: Option<String>) -> *const c_char {
    to_c_string(s.as_deref().unwrap_or(""))
}

// ─── Public FFI ──────────────────────────────────────────────────────────────

/// Initialises all PUCE sub-engines.
///
/// Must be called **once** before any other function.  Calling it a second
/// time while the engine is already running is a no-op and returns `0`.
///
/// # Returns
/// * `0` — success
/// * `-1` — failure (check logs)
#[no_mangle]
pub extern "C" fn puce_init() -> c_int {
    // Initialise the logger (ignore error if already initialised).
    let _ = env_logger::try_init();
    info!("puce_init: starting PUCE v{PUCE_VERSION}");

    let mut guard = match global_state().lock() {
        Ok(g) => g,
        Err(e) => { error!("puce_init: mutex poisoned: {e}"); return -1; }
    };

    if guard.is_some() {
        info!("puce_init: already initialised — ignoring");
        return 0;
    }

    let result: anyhow::Result<PuceState> = (|| {
        let db_path = std::env::var("PUCE_DB_PATH")
            .unwrap_or_else(|_| "puce.db".to_string());

        let database = Database::init(&db_path)
            .context("database init")?;

        let detection = DetectionEngine::new()
            .context("detection engine init")?;

        let mapping = MappingEngine::new()
            .context("mapping engine init")?;

        // EmulationEngine::new() returns Self directly (not Result)
        let emulation = EmulationEngine::new(PSMode::DualSense);

        // PluginManager::new() returns Self directly — takes (plugins_dir, require_signatures)
        let plugins_dir = std::env::var("PUCE_PLUGINS_DIR")
            .map(std::path::PathBuf::from)
            .unwrap_or_else(|_| std::path::PathBuf::from("plugins"));
        let require_sig = std::env::var("PUCE_REQUIRE_SIGNED_PLUGINS")
            .map(|v| v != "0")
            .unwrap_or(true);
        let plugins = PluginManager::new(plugins_dir, require_sig);

        Ok(PuceState {
            detection,
            mapping,
            emulation,
            plugins,
            database,
            running: false,
        })
    })();

    match result {
        Ok(state) => {
            *guard = Some(state);
            info!("puce_init: success");
            0
        }
        Err(e) => {
            error!("puce_init: {e:#}");
            -1
        }
    }
}

/// Returns a **null-terminated** UTF-8 string containing the PUCE version.
///
/// The caller owns the returned pointer and must free it with
/// [`puce_free_string`].
#[no_mangle]
pub extern "C" fn puce_get_version() -> *const c_char {
    to_c_string(PUCE_VERSION)
}

/// Starts background HID and Bluetooth device scanning.
///
/// # Returns
/// * `0` — success
/// * `-1` — engine not initialised or scan failed to start
#[no_mangle]
pub extern "C" fn puce_start_detection() -> c_int {
    let mut guard = match global_state().lock() {
        Ok(g) => g,
        Err(e) => { error!("puce_start_detection: mutex poisoned: {e}"); return -1; }
    };

    match guard.as_mut() {
        None => { error!("puce_start_detection: not initialised"); -1 }
        Some(state) => match state.detection.start() {
            Ok(()) => { state.running = true; 0 }
            Err(e) => { error!("puce_start_detection: {e:#}"); -1 }
        }
    }
}

/// Stops background device scanning.
#[no_mangle]
pub extern "C" fn puce_stop_detection() {
    let mut guard = match global_state().lock() {
        Ok(g) => g,
        Err(_) => return,
    };
    if let Some(state) = guard.as_mut() {
        state.detection.stop();
        state.running = false;
    }
}

/// Returns a JSON array of currently detected devices.
///
/// Example:
/// ```json
/// [
///   {
///     "vid": 1356, "pid": 3302, "name": "DualSense Wireless Controller",
///     "manufacturer": "Sony Interactive Entertainment",
///     "device_type": "PlayStation",
///     "buttons": 17, "axes": 6,
///     "capabilities": ["rumble","touchpad","gyro","accel","led","mic"],
///     "firmware": "01.00", "bt_version": "5.0", "usb_version": "3.0"
///   }
/// ]
/// ```
///
/// The caller owns the returned pointer and must free it with
/// [`puce_free_string`].
#[no_mangle]
pub extern "C" fn puce_get_devices() -> *const c_char {
    let guard = match global_state().lock() {
        Ok(g) => g,
        Err(e) => { error!("puce_get_devices: {e}"); return to_c_string("[]"); }
    };

    match guard.as_ref() {
        None => to_c_string("[]"),
        Some(state) => {
            let devices = state.detection.get_devices();
            match serde_json::to_string(&devices) {
                Ok(json) => to_c_string(&json),
                Err(e) => {
                    error!("puce_get_devices: serialisation error: {e}");
                    to_c_string("[]")
                }
            }
        }
    }
}

/// Sets the PlayStation emulation mode for a specific device.
///
/// # Parameters
/// * `device_id` — UTF-8 device identifier string (from device list)
/// * `mode`      — one of:
///   `"PS1"`, `"DualAnalog"`, `"DualShock1"`, `"DualShock2"`,
///   `"Sixaxis"`, `"DualShock3"`, `"DualShock4"`,
///   `"DualSense"`, `"DualSenseEdge"`
///
/// # Returns
/// * `0` — success
/// * `-1` — invalid parameters or engine not initialised
#[no_mangle]
pub extern "C" fn puce_set_emulation_mode(
    device_id: *const c_char,
    mode:       *const c_char,
) -> c_int {
    if device_id.is_null() || mode.is_null() {
        error!("puce_set_emulation_mode: null pointer argument");
        return -1;
    }

    let device_id_str = unsafe {
        match CStr::from_ptr(device_id).to_str() {
            Ok(s) => s.to_owned(),
            Err(_) => { error!("puce_set_emulation_mode: invalid device_id utf-8"); return -1; }
        }
    };

    let mode_str = unsafe {
        match CStr::from_ptr(mode).to_str() {
            Ok(s) => s.to_owned(),
            Err(_) => { error!("puce_set_emulation_mode: invalid mode utf-8"); return -1; }
        }
    };

    let ps_mode = match mode_str.as_str() {
        "PS1"          => PSMode::PS1,
        "DualAnalog"   => PSMode::DualAnalog,
        "DualShock1"   => PSMode::DualShock1,
        "DualShock2"   => PSMode::DualShock2,
        "Sixaxis"      => PSMode::Sixaxis,
        "DualShock3"   => PSMode::DualShock3,
        "DualShock4"   => PSMode::DualShock4,
        "DualSense"    => PSMode::DualSense,
        "DualSenseEdge"=> PSMode::DualSenseEdge,
        other => {
            error!("puce_set_emulation_mode: unknown mode '{other}'");
            return -1;
        }
    };

    let mut guard = match global_state().lock() {
        Ok(g) => g,
        Err(e) => { error!("puce_set_emulation_mode: {e}"); return -1; }
    };

    match guard.as_mut() {
        None => { error!("puce_set_emulation_mode: not initialised"); -1 }
        Some(state) => {
            info!("puce_set_emulation_mode: device={device_id_str} mode={mode_str}");
            state.emulation.set_mode(ps_mode);
            0
        }
    }
}

/// Returns a comprehensive JSON status object.
///
/// ```json
/// {
///   "version": "0.1.0",
///   "running": true,
///   "device_count": 1,
///   "emulation_mode": "DualSense",
///   "plugin_count": 0,
///   "uptime_secs": 42
/// }
/// ```
///
/// The caller owns the returned pointer and must free it with
/// [`puce_free_string`].
#[no_mangle]
pub extern "C" fn puce_get_status() -> *const c_char {
    let guard = match global_state().lock() {
        Ok(g) => g,
        Err(e) => {
            error!("puce_get_status: {e}");
            let j = json!({ "error": e.to_string() });
            return to_c_string(&j.to_string());
        }
    };

    match guard.as_ref() {
        None => {
            let j = json!({ "status": "not_initialised" });
            to_c_string(&j.to_string())
        }
        Some(state) => {
            let devices = state.detection.get_devices();
            let plugins = state.plugins.list();
            // PSMode exposes display_name() directly on the enum
            let mode    = state.emulation.mode().display_name();

            let status = json!({
                "version":        PUCE_VERSION,
                "running":        state.running,
                "device_count":   devices.len(),
                "emulation_mode": mode,
                "plugin_count":   plugins.len(),
            });

            to_c_string(&status.to_string())
        }
    }
}

/// Shuts down all engines and releases all resources.
///
/// After calling this function you may call [`puce_init`] again to restart.
#[no_mangle]
pub extern "C" fn puce_shutdown() {
    info!("puce_shutdown: shutting down");
    let mut guard = match global_state().lock() {
        Ok(g) => g,
        Err(_) => return,
    };

    if let Some(mut state) = guard.take() {
        state.detection.stop();
        // Remaining resources are dropped here via `Drop`.
        info!("puce_shutdown: complete");
    }
}

/// Frees a string previously returned by any `puce_*` function.
///
/// # Safety
/// `ptr` MUST have been obtained from a PUCE function.  Passing any other
/// pointer is **undefined behaviour**.  Passing a null pointer is a no-op.
#[no_mangle]
pub unsafe extern "C" fn puce_free_string(ptr: *mut c_char) {
    if !ptr.is_null() {
        drop(CString::from_raw(ptr));
    }
}

/// Loads a plugin from the given filesystem path.
///
/// # Returns
/// * `0` — success
/// * `-1` — load failure (check logs)
#[no_mangle]
pub extern "C" fn puce_load_plugin(path: *const c_char) -> c_int {
    if path.is_null() {
        error!("puce_load_plugin: null path");
        return -1;
    }
    let path_str = unsafe {
        match CStr::from_ptr(path).to_str() {
            Ok(s) => std::path::Path::new(s).to_path_buf(),
            Err(_) => { error!("puce_load_plugin: invalid utf-8"); return -1; }
        }
    };

    let mut guard = match global_state().lock() {
        Ok(g) => g,
        Err(e) => { error!("puce_load_plugin: {e}"); return -1; }
    };

    match guard.as_mut() {
        None => { error!("puce_load_plugin: not initialised"); -1 }
        Some(state) => match state.plugins.load(&path_str) {
            // load() returns Result<PluginInfo, PluginError> — we only care about success/failure
            Ok(info) => {
                info!("puce_load_plugin: loaded plugin '{}' v{}", info.name, info.version);
                0
            }
            Err(e) => { error!("puce_load_plugin: {e:#}"); -1 }
        }
    }
}

/// Returns a JSON array of loaded plugin metadata.
///
/// The caller owns the returned pointer and must free it with
/// [`puce_free_string`].
#[no_mangle]
pub extern "C" fn puce_list_plugins() -> *const c_char {
    let guard = match global_state().lock() {
        Ok(g) => g,
        Err(_) => return to_c_string("[]"),
    };
    match guard.as_ref() {
        None => to_c_string("[]"),
        Some(state) => {
            let plugins = state.plugins.list();
            match serde_json::to_string(&plugins) {
                Ok(j) => to_c_string(&j),
                Err(_) => to_c_string("[]"),
            }
        }
    }
}
