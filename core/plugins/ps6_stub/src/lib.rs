//! # PS6 Stub Plugin
//! 
//! Example plugin demonstrating how to use the PUCE C-ABI from Rust.
//! It registers a hypothetical future "PlayStation 6 DualSense II" controller.

use std::os::raw::{c_char, c_float};
use std::ptr;

// ─────────────────────────────────────────────────────────────
// C-ABI Equivalents
// ─────────────────────────────────────────────────────────────

pub const PUCE_PLUGIN_ABI_VERSION: u32 = 1;

#[repr(C)]
pub struct PucePluginInfoC {
    pub name: *const c_char,
    pub version: *const c_char,
    pub author: *const c_char,
    pub description: *const c_char,
    pub abi_version: u32,
}

#[repr(C)]
pub struct DeviceInfoC {
    pub vendor_id: u16,
    pub product_id: u16,
    pub name: *const c_char,
    pub manufacturer: *const c_char,
    pub button_count: u8,
    pub axis_count: u8,
}

#[repr(C)]
pub struct ButtonMappingC {
    pub source_button: u8,
    pub target_button: u8,
    pub modifier: u8,
}

#[repr(C)]
pub struct AxisMappingC {
    pub source_axis: u8,
    pub target_axis: u8,
    pub scale: c_float,
    pub dead_zone: c_float,
    pub invert: u8,
}

#[repr(C)]
pub struct MappingProfileC {
    pub name: *const c_char,
    pub ps_mode: u8,
    pub button_mappings: *const ButtonMappingC,
    pub button_count: u32,
    pub axis_mappings: *const AxisMappingC,
    pub axis_count: u32,
}

// ─────────────────────────────────────────────────────────────
// Plugin Data
// ─────────────────────────────────────────────────────────────

static PLUGIN_NAME: &[u8] = b"PS6 DualSense II Stub\0";
static PLUGIN_VERSION: &[u8] = b"1.0.0\0";
static PLUGIN_AUTHOR: &[u8] = b"PUCE Team\0";
static PLUGIN_DESC: &[u8] = b"Adds hypothetical support for next-gen controllers\0";
static PROFILE_NAME: &[u8] = b"PS6 Native Mapping\0";

static INFO: PucePluginInfoC = PucePluginInfoC {
    name: PLUGIN_NAME.as_ptr() as *const c_char,
    version: PLUGIN_VERSION.as_ptr() as *const c_char,
    author: PLUGIN_AUTHOR.as_ptr() as *const c_char,
    description: PLUGIN_DESC.as_ptr() as *const c_char,
    abi_version: PUCE_PLUGIN_ABI_VERSION,
};

// Example mappings
static BUTTONS: [ButtonMappingC; 2] = [
    ButtonMappingC { source_button: 0, target_button: 1, modifier: 0 }, // Cross
    ButtonMappingC { source_button: 1, target_button: 2, modifier: 0 }, // Circle
];

static AXES: [AxisMappingC; 2] = [
    AxisMappingC { source_axis: 0, target_axis: 0, scale: 1.0, dead_zone: 0.05, invert: 0 },
    AxisMappingC { source_axis: 1, target_axis: 1, scale: 1.0, dead_zone: 0.05, invert: 1 },
];

static mut PROFILE: MappingProfileC = MappingProfileC {
    name: PROFILE_NAME.as_ptr() as *const c_char,
    ps_mode: 7, // DualSense enum equivalent
    button_mappings: ptr::null(),
    button_count: 0,
    axis_mappings: ptr::null(),
    axis_count: 0,
};

// ─────────────────────────────────────────────────────────────
// Exported Functions
// ─────────────────────────────────────────────────────────────

#[no_mangle]
pub extern "C" fn puce_plugin_init() -> bool {
    // Setup the pointers once during init
    unsafe {
        PROFILE.button_mappings = BUTTONS.as_ptr();
        PROFILE.button_count = BUTTONS.len() as u32;
        PROFILE.axis_mappings = AXES.as_ptr();
        PROFILE.axis_count = AXES.len() as u32;
    }
    true
}

#[no_mangle]
pub extern "C" fn puce_plugin_shutdown() {
    // Cleanup if necessary
}

#[no_mangle]
pub extern "C" fn puce_plugin_get_info() -> *const PucePluginInfoC {
    &INFO
}

#[no_mangle]
pub extern "C" fn puce_plugin_handles_device(device: *const DeviceInfoC) -> bool {
    if device.is_null() {
        return false;
    }
    let dev = unsafe { &*device };
    // Hypothetical PS6 Controller VID/PID (Sony VID is 0x054C)
    dev.vendor_id == 0x054C && dev.product_id == 0x0FFF
}

#[no_mangle]
pub extern "C" fn puce_plugin_get_mapping(_device: *const DeviceInfoC) -> *const MappingProfileC {
    unsafe { &PROFILE }
}
