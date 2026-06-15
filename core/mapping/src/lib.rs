//! # mapping — Button & axis mapping engine
//!
//! Converts raw HID reports from any controller into a normalised
//! [`PSReport`] that the emulation engine can consume.
//!
//! ## Key concepts
//! * **[`MappingProfile`]** — saved user configuration for one device.
//! * **[`MappingEngine`]** — runtime that applies a profile to raw input.
//! * **Auto-mapping** — heuristic profiles generated automatically per device
//!   class (Xbox → PS, Nintendo → PS, Keyboard → PS, etc.).

use std::collections::HashMap;
use anyhow::{bail, Result};
use log::{debug, warn};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use detection::{DeviceInfo, DeviceType};

// ─── PSButton ────────────────────────────────────────────────────────────────

/// Every button a PlayStation controller can expose, across all generations.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PSButton {
    /// × / Cross
    Cross,
    /// ○ / Circle
    Circle,
    /// □ / Square
    Square,
    /// △ / Triangle
    Triangle,
    L1,
    L2,
    R1,
    R2,
    /// Left stick click.
    L3,
    /// Right stick click.
    R3,
    DPadUp,
    DPadDown,
    DPadLeft,
    DPadRight,
    /// PS1/PS2 Start button.
    Start,
    /// PS1/PS2 Select button.
    Select,
    /// PlayStation logo button.
    PS,
    /// PS4 Share button.
    Share,
    /// PS4/PS5 Options button.
    Options,
    /// PS5 Create button (replaces Share on DualSense).
    Create,
    /// Touchpad click (DS4 / DualSense).
    TouchpadClick,
    /// Mute microphone button (DualSense).
    Mute,
}

// ─── PSAxis ──────────────────────────────────────────────────────────────────

/// Analogue axes exposed by PlayStation controllers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PSAxis {
    /// Left stick horizontal (0 = full left, 127 = centre, 255 = full right).
    LeftX,
    /// Left stick vertical (0 = full up, 127 = centre, 255 = full down).
    LeftY,
    /// Right stick horizontal.
    RightX,
    /// Right stick vertical.
    RightY,
    /// L2 analogue pressure (0 = released, 255 = fully pressed).
    L2Analog,
    /// R2 analogue pressure.
    R2Analog,
}

// ─── ModifierKey ─────────────────────────────────────────────────────────────

/// A keyboard modifier key that must be held for the mapping to fire.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ModifierKey {
    Shift,
    Ctrl,
    Alt,
    Meta,
}

// ─── VirtualButton ───────────────────────────────────────────────────────────

/// A software-synthesised button derived from axis position or gesture.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VirtualButton {
    /// Source axis index.
    pub axis:      u8,
    /// Threshold value (0-255) at which the virtual button fires.
    pub threshold: u8,
    /// Whether the button fires when axis > threshold (true) or < threshold (false).
    pub positive:  bool,
    /// The PS button to activate.
    pub target:    PSButton,
}

// ─── ButtonMapping ───────────────────────────────────────────────────────────

/// Maps one raw HID button index to a PlayStation button.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ButtonMapping {
    /// Zero-based index of the source button in the raw HID report.
    pub source_button: u8,
    /// PlayStation button to emit.
    pub target_button: PSButton,
    /// Optional keyboard modifier that must be held simultaneously.
    pub modifier:      Option<ModifierKey>,
}

// ─── AxisMapping ─────────────────────────────────────────────────────────────

/// Maps one raw HID axis to a PlayStation axis.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AxisMapping {
    /// Zero-based index of the source axis in the raw HID report.
    pub source_axis: u8,
    /// PlayStation axis to emit.
    pub target_axis: PSAxis,
    /// Scaling factor applied after deadzone removal (default 1.0).
    pub scale:       f32,
    /// Deadzone radius around the centre (0.0 – 1.0, default 0.05).
    pub deadzone:    f32,
    /// If `true`, the axis output is inverted.
    pub invert:      bool,
}

// ─── MappingProfile ──────────────────────────────────────────────────────────

/// A complete input mapping configuration for one device.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MappingProfile {
    /// UUID v4 string identifier.
    pub id:              String,
    /// Human-readable name shown in UI.
    pub name:            String,
    /// The device id this profile was created for.
    pub device_id:       String,
    /// Button remapping table.
    pub button_mappings: Vec<ButtonMapping>,
    /// Axis remapping table.
    pub axis_mappings:   Vec<AxisMapping>,
    /// Virtual (synthesised) buttons from axis positions.
    pub virtual_buttons: Vec<VirtualButton>,
}

// ─── RawHIDReport ────────────────────────────────────────────────────────────

/// A decoded raw HID report from any controller.
#[derive(Debug, Clone, Default)]
pub struct RawHIDReport {
    /// Boolean state for up to 32 buttons.
    pub buttons:   [bool; 32],
    /// Axis values 0-255 for up to 8 axes.
    pub axes:      [u8; 8],
    /// Hat switch value (0 = none, 1 = N, 2 = NE, …, 8 = NW).
    pub hat:       u8,
    /// Raw keyboard scan codes currently pressed (for keyboard sources).
    pub key_codes: Vec<u8>,
}

// ─── PSReport ────────────────────────────────────────────────────────────────

/// Normalised PlayStation controller report — emulation engine input.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PSReport {
    /// Set of currently active PS buttons.
    pub buttons:   std::collections::HashSet<PSButton>,
    pub left_x:    u8,
    pub left_y:    u8,
    pub right_x:   u8,
    pub right_y:   u8,
    pub l2_analog: u8,
    pub r2_analog: u8,
    /// Touchpad finger 1 position (0-1920, 0-942).
    pub touch_x:   u16,
    pub touch_y:   u16,
    pub touching:  bool,
    /// Gyroscope (deg/s, raw signed 16-bit).
    pub gyro_x:    i16,
    pub gyro_y:    i16,
    pub gyro_z:    i16,
    /// Accelerometer (raw signed 16-bit).
    pub accel_x:   i16,
    pub accel_y:   i16,
    pub accel_z:   i16,
}

// ─── Helper: apply axis mapping ──────────────────────────────────────────────

fn apply_axis(raw: u8, mapping: &AxisMapping) -> u8 {
    // Normalise to -1.0 … +1.0.
    let normalised = (raw as f32 - 127.5) / 127.5;

    // Apply deadzone.
    let after_dz = if normalised.abs() < mapping.deadzone {
        0.0_f32
    } else {
        let sign = normalised.signum();
        let scaled = (normalised.abs() - mapping.deadzone) / (1.0 - mapping.deadzone);
        sign * scaled
    };

    // Apply scale and optional inversion.
    let after_scale = after_dz * mapping.scale * if mapping.invert { -1.0 } else { 1.0 };

    // Clamp and convert back to u8.
    let out = (after_scale * 127.5 + 127.5).clamp(0.0, 255.0);
    out as u8
}

// ─── MappingEngine ───────────────────────────────────────────────────────────

/// Runtime engine that applies [`MappingProfile`]s to raw HID reports.
pub struct MappingEngine {
    /// Map of profile id → profile.
    profiles: HashMap<String, MappingProfile>,
    /// Currently active profile id (per device id → profile id).
    active:   HashMap<String, String>,
}

impl MappingEngine {
    /// Creates a new, empty `MappingEngine`.
    pub fn new() -> Result<Self> {
        Ok(Self {
            profiles: HashMap::new(),
            active:   HashMap::new(),
        })
    }

    /// Loads a profile into the engine's registry.
    pub fn load_profile(&mut self, profile: MappingProfile) {
        self.profiles.insert(profile.id.clone(), profile);
    }

    /// Sets the active profile for a device.
    ///
    /// Returns an error if the profile id is not loaded.
    pub fn set_active_profile(&mut self, device_id: &str, profile_id: &str) -> Result<()> {
        if !self.profiles.contains_key(profile_id) {
            bail!("Profile '{}' not loaded", profile_id);
        }
        self.active.insert(device_id.to_string(), profile_id.to_string());
        Ok(())
    }

    /// Removes the active profile for a device (falls back to auto-map).
    pub fn clear_active_profile(&mut self, device_id: &str) {
        self.active.remove(device_id);
    }

    /// Returns the active [`MappingProfile`] for a device, or `None`.
    pub fn get_profile(&self, device_id: &str) -> Option<&MappingProfile> {
        let id = self.active.get(device_id)?;
        self.profiles.get(id)
    }

    /// Applies the active profile for `device_id` to a raw HID report.
    ///
    /// If no profile is loaded for the device, falls back to the identity
    /// mapping (direct 1:1 axis/button pass-through).
    pub fn map_input(&self, device_id: &str, raw: &RawHIDReport) -> PSReport {
        match self.get_profile(device_id) {
            Some(profile) => apply_profile(profile, raw),
            None          => passthrough_map(raw),
        }
    }

    /// Generates an automatic [`MappingProfile`] for a detected device.
    ///
    /// The auto-mapping heuristics cover:
    /// * Xbox controllers → PlayStation layout
    /// * Nintendo Switch controllers → PlayStation layout
    /// * Keyboard (WASD = left stick, IJKL = d-pad, etc.)
    /// * Generic / unknown → identity mapping
    pub fn auto_map(&self, device: &DeviceInfo) -> MappingProfile {
        match device.device_type {
            DeviceType::Xbox       => xbox_to_ps_profile(device),
            DeviceType::Nintendo   => nintendo_to_ps_profile(device),
            DeviceType::Keyboard   => keyboard_to_ps_profile(device),
            DeviceType::Mouse      => mouse_to_ps_profile(device),
            DeviceType::EightBitDo => eightbitdo_to_ps_profile(device),
            DeviceType::Logitech   => logitech_to_ps_profile(device),
            _                      => generic_profile(device),
        }
    }

    /// Returns all loaded profiles.
    pub fn list_profiles(&self) -> Vec<&MappingProfile> {
        self.profiles.values().collect()
    }
}

// ─── Profile application ─────────────────────────────────────────────────────

fn apply_profile(profile: &MappingProfile, raw: &RawHIDReport) -> PSReport {
    let mut report = PSReport::default();
    report.left_x  = 127;
    report.left_y  = 127;
    report.right_x = 127;
    report.right_y = 127;

    // Button mappings.
    for bm in &profile.button_mappings {
        let idx = bm.source_button as usize;
        if idx < raw.buttons.len() && raw.buttons[idx] {
            report.buttons.insert(bm.target_button);
        }
    }

    // Axis mappings.
    for am in &profile.axis_mappings {
        let idx = am.source_axis as usize;
        if idx < raw.axes.len() {
            let value = apply_axis(raw.axes[idx], am);
            match am.target_axis {
                PSAxis::LeftX    => report.left_x    = value,
                PSAxis::LeftY    => report.left_y    = value,
                PSAxis::RightX   => report.right_x   = value,
                PSAxis::RightY   => report.right_y   = value,
                PSAxis::L2Analog => report.l2_analog  = value,
                PSAxis::R2Analog => report.r2_analog  = value,
            }
        }
    }

    // Virtual buttons from axis thresholds.
    for vb in &profile.virtual_buttons {
        let idx = vb.axis as usize;
        if idx < raw.axes.len() {
            let fires = if vb.positive {
                raw.axes[idx] > vb.threshold
            } else {
                raw.axes[idx] < vb.threshold
            };
            if fires {
                report.buttons.insert(vb.target);
            }
        }
    }

    // D-pad from hat switch.
    apply_hat(raw.hat, &mut report);

    report
}

fn passthrough_map(raw: &RawHIDReport) -> PSReport {
    let mut report = PSReport::default();
    report.left_x  = raw.axes.get(0).copied().unwrap_or(127);
    report.left_y  = raw.axes.get(1).copied().unwrap_or(127);
    report.right_x = raw.axes.get(2).copied().unwrap_or(127);
    report.right_y = raw.axes.get(3).copied().unwrap_or(127);
    report.l2_analog = raw.axes.get(4).copied().unwrap_or(0);
    report.r2_analog = raw.axes.get(5).copied().unwrap_or(0);

    let btn_map: [(usize, PSButton); 10] = [
        (0, PSButton::Cross),
        (1, PSButton::Circle),
        (2, PSButton::Square),
        (3, PSButton::Triangle),
        (4, PSButton::L1),
        (5, PSButton::R1),
        (6, PSButton::L2),
        (7, PSButton::R2),
        (8, PSButton::Start),
        (9, PSButton::Select),
    ];
    for (i, btn) in &btn_map {
        if *i < raw.buttons.len() && raw.buttons[*i] {
            report.buttons.insert(*btn);
        }
    }
    apply_hat(raw.hat, &mut report);
    report
}

fn apply_hat(hat: u8, report: &mut PSReport) {
    // Hat values: 0=none, 1=N, 2=NE, 3=E, 4=SE, 5=S, 6=SW, 7=W, 8=NW
    match hat {
        1 | 2 | 8 => { report.buttons.insert(PSButton::DPadUp); }
        4 | 5 | 6 => { report.buttons.insert(PSButton::DPadDown); }
        _ => {}
    }
    match hat {
        2 | 3 | 4 => { report.buttons.insert(PSButton::DPadRight); }
        6 | 7 | 8 => { report.buttons.insert(PSButton::DPadLeft); }
        _ => {}
    }
}

// ─── Auto-mapping heuristics ─────────────────────────────────────────────────

/// Xbox One/360 → PlayStation button mapping.
///
/// Layout correspondence:
/// A→Cross, B→Circle, X→Square, Y→Triangle,
/// LB→L1, RB→R1, LT→L2, RT→R2,
/// Back/View→Select, Start/Menu→Options,
/// Xbox→PS, L3→L3, R3→R3
fn xbox_to_ps_profile(device: &DeviceInfo) -> MappingProfile {
    MappingProfile {
        id:        Uuid::new_v4().to_string(),
        name:      format!("Auto: {} → PlayStation", device.name),
        device_id: device.id.clone(),
        button_mappings: vec![
            ButtonMapping { source_button: 0,  target_button: PSButton::Cross,    modifier: None },
            ButtonMapping { source_button: 1,  target_button: PSButton::Circle,   modifier: None },
            ButtonMapping { source_button: 2,  target_button: PSButton::Square,   modifier: None },
            ButtonMapping { source_button: 3,  target_button: PSButton::Triangle, modifier: None },
            ButtonMapping { source_button: 4,  target_button: PSButton::L1,       modifier: None },
            ButtonMapping { source_button: 5,  target_button: PSButton::R1,       modifier: None },
            ButtonMapping { source_button: 6,  target_button: PSButton::Select,   modifier: None },
            ButtonMapping { source_button: 7,  target_button: PSButton::Options,  modifier: None },
            ButtonMapping { source_button: 8,  target_button: PSButton::L3,       modifier: None },
            ButtonMapping { source_button: 9,  target_button: PSButton::R3,       modifier: None },
            ButtonMapping { source_button: 10, target_button: PSButton::PS,       modifier: None },
        ],
        axis_mappings: vec![
            AxisMapping { source_axis: 0, target_axis: PSAxis::LeftX,    scale: 1.0, deadzone: 0.05, invert: false },
            AxisMapping { source_axis: 1, target_axis: PSAxis::LeftY,    scale: 1.0, deadzone: 0.05, invert: false },
            AxisMapping { source_axis: 2, target_axis: PSAxis::RightX,   scale: 1.0, deadzone: 0.05, invert: false },
            AxisMapping { source_axis: 3, target_axis: PSAxis::RightY,   scale: 1.0, deadzone: 0.05, invert: false },
            AxisMapping { source_axis: 4, target_axis: PSAxis::L2Analog, scale: 1.0, deadzone: 0.02, invert: false },
            AxisMapping { source_axis: 5, target_axis: PSAxis::R2Analog, scale: 1.0, deadzone: 0.02, invert: false },
        ],
        virtual_buttons: vec![
            // LT/RT digital click when analog > 200.
            VirtualButton { axis: 4, threshold: 200, positive: true, target: PSButton::L2 },
            VirtualButton { axis: 5, threshold: 200, positive: true, target: PSButton::R2 },
        ],
    }
}

/// Nintendo Switch Pro Controller → PlayStation mapping.
///
/// B→Cross, A→Circle, Y→Square, X→Triangle
fn nintendo_to_ps_profile(device: &DeviceInfo) -> MappingProfile {
    MappingProfile {
        id:        Uuid::new_v4().to_string(),
        name:      format!("Auto: {} → PlayStation", device.name),
        device_id: device.id.clone(),
        button_mappings: vec![
            ButtonMapping { source_button: 0,  target_button: PSButton::Cross,    modifier: None }, // B
            ButtonMapping { source_button: 1,  target_button: PSButton::Circle,   modifier: None }, // A
            ButtonMapping { source_button: 2,  target_button: PSButton::Square,   modifier: None }, // Y
            ButtonMapping { source_button: 3,  target_button: PSButton::Triangle, modifier: None }, // X
            ButtonMapping { source_button: 4,  target_button: PSButton::L1,       modifier: None }, // L
            ButtonMapping { source_button: 5,  target_button: PSButton::R1,       modifier: None }, // R
            ButtonMapping { source_button: 6,  target_button: PSButton::L2,       modifier: None }, // ZL
            ButtonMapping { source_button: 7,  target_button: PSButton::R2,       modifier: None }, // ZR
            ButtonMapping { source_button: 8,  target_button: PSButton::Select,   modifier: None }, // Minus
            ButtonMapping { source_button: 9,  target_button: PSButton::Options,  modifier: None }, // Plus
            ButtonMapping { source_button: 10, target_button: PSButton::L3,       modifier: None },
            ButtonMapping { source_button: 11, target_button: PSButton::R3,       modifier: None },
            ButtonMapping { source_button: 12, target_button: PSButton::PS,       modifier: None }, // Home
            ButtonMapping { source_button: 13, target_button: PSButton::TouchpadClick, modifier: None }, // Capture
        ],
        axis_mappings: vec![
            AxisMapping { source_axis: 0, target_axis: PSAxis::LeftX,    scale: 1.0, deadzone: 0.08, invert: false },
            AxisMapping { source_axis: 1, target_axis: PSAxis::LeftY,    scale: 1.0, deadzone: 0.08, invert: false },
            AxisMapping { source_axis: 2, target_axis: PSAxis::RightX,   scale: 1.0, deadzone: 0.08, invert: false },
            AxisMapping { source_axis: 3, target_axis: PSAxis::RightY,   scale: 1.0, deadzone: 0.08, invert: false },
        ],
        virtual_buttons: vec![],
    }
}

/// Keyboard → PlayStation mapping.
///
/// Key codes (HID usage page 0x07):
/// * WASD (0x1A, 0x04, 0x16, 0x07) = Left stick
/// * IJKL (0x0C, 0x0D, 0x0E, 0x0F) = D-pad
/// * Space = Cross, Enter = Options, Escape = PS
/// * ZXCV = Triangle, Square, Circle, Cross (fighting layout)
/// * QE = L1/R1, RF = L2/R2 (shooters)
fn keyboard_to_ps_profile(device: &DeviceInfo) -> MappingProfile {
    // Keyboard uses key_codes, not buttons[]. Virtual buttons map key codes
    // to PS buttons. We use axis 6 and 7 as virtual "keyboard axes".
    MappingProfile {
        id:        Uuid::new_v4().to_string(),
        name:      format!("Auto: Keyboard → PlayStation"),
        device_id: device.id.clone(),
        button_mappings: vec![
            // Space → Cross
            ButtonMapping { source_button: 44, target_button: PSButton::Cross,    modifier: None },
            // Enter → Options
            ButtonMapping { source_button: 40, target_button: PSButton::Options,  modifier: None },
            // Escape → PS
            ButtonMapping { source_button: 41, target_button: PSButton::PS,       modifier: None },
            // Z → Triangle
            ButtonMapping { source_button: 29, target_button: PSButton::Triangle, modifier: None },
            // X → Square
            ButtonMapping { source_button: 27, target_button: PSButton::Square,   modifier: None },
            // C → Circle
            ButtonMapping { source_button: 6,  target_button: PSButton::Circle,   modifier: None },
            // V → Cross
            ButtonMapping { source_button: 25, target_button: PSButton::Cross,    modifier: None },
            // Q → L1
            ButtonMapping { source_button: 20, target_button: PSButton::L1,       modifier: None },
            // E → R1
            ButtonMapping { source_button: 8,  target_button: PSButton::R1,       modifier: None },
            // R → L2
            ButtonMapping { source_button: 21, target_button: PSButton::L2,       modifier: None },
            // F → R2
            ButtonMapping { source_button: 9,  target_button: PSButton::R2,       modifier: None },
            // Arrow Up → DPadUp
            ButtonMapping { source_button: 82, target_button: PSButton::DPadUp,   modifier: None },
            // Arrow Down → DPadDown
            ButtonMapping { source_button: 81, target_button: PSButton::DPadDown, modifier: None },
            // Arrow Left → DPadLeft
            ButtonMapping { source_button: 80, target_button: PSButton::DPadLeft, modifier: None },
            // Arrow Right → DPadRight
            ButtonMapping { source_button: 79, target_button: PSButton::DPadRight,modifier: None },
            // Tab → Select/Share
            ButtonMapping { source_button: 43, target_button: PSButton::Select,   modifier: None },
        ],
        axis_mappings: vec![],
        // WASD virtual stick via threshold virtual buttons.
        virtual_buttons: vec![
            // W (0x1A = 26) → left stick up (axis 1 < 50)
            VirtualButton { axis: 1, threshold: 50,  positive: false, target: PSButton::DPadUp   },
            // S (0x16 = 22) → left stick down (axis 1 > 200)
            VirtualButton { axis: 1, threshold: 200, positive: true,  target: PSButton::DPadDown },
            // A (0x04 = 4) → left stick left (axis 0 < 50)
            VirtualButton { axis: 0, threshold: 50,  positive: false, target: PSButton::DPadLeft },
            // D (0x07 = 7) → left stick right (axis 0 > 200)
            VirtualButton { axis: 0, threshold: 200, positive: true,  target: PSButton::DPadRight},
        ],
    }
}

/// Mouse → PlayStation touchpad & right-stick mapping.
fn mouse_to_ps_profile(device: &DeviceInfo) -> MappingProfile {
    MappingProfile {
        id:        Uuid::new_v4().to_string(),
        name:      "Auto: Mouse → PlayStation Touchpad".to_string(),
        device_id: device.id.clone(),
        button_mappings: vec![
            // Left button → Cross
            ButtonMapping { source_button: 0, target_button: PSButton::Cross,       modifier: None },
            // Right button → Circle
            ButtonMapping { source_button: 1, target_button: PSButton::Circle,      modifier: None },
            // Middle click → TouchpadClick
            ButtonMapping { source_button: 2, target_button: PSButton::TouchpadClick, modifier: None },
        ],
        axis_mappings: vec![
            // Mouse X → Right stick X
            AxisMapping { source_axis: 0, target_axis: PSAxis::RightX, scale: 0.8, deadzone: 0.01, invert: false },
            // Mouse Y → Right stick Y
            AxisMapping { source_axis: 1, target_axis: PSAxis::RightY, scale: 0.8, deadzone: 0.01, invert: false },
        ],
        virtual_buttons: vec![],
    }
}

/// 8BitDo → PlayStation mapping (similar to Xbox layout).
fn eightbitdo_to_ps_profile(device: &DeviceInfo) -> MappingProfile {
    let mut profile = xbox_to_ps_profile(device);
    profile.name = format!("Auto: {} → PlayStation", device.name);
    profile
}

/// Logitech → PlayStation mapping (similar to Xbox layout, some differences).
fn logitech_to_ps_profile(device: &DeviceInfo) -> MappingProfile {
    let mut profile = xbox_to_ps_profile(device);
    profile.name = format!("Auto: {} → PlayStation", device.name);
    profile
}

/// Generic / unknown device → identity mapping.
fn generic_profile(device: &DeviceInfo) -> MappingProfile {
    MappingProfile {
        id:        Uuid::new_v4().to_string(),
        name:      format!("Auto: {} (generic)", device.name),
        device_id: device.id.clone(),
        button_mappings: (0u8..16).map(|i| {
            let btn = match i {
                0  => PSButton::Cross,
                1  => PSButton::Circle,
                2  => PSButton::Square,
                3  => PSButton::Triangle,
                4  => PSButton::L1,
                5  => PSButton::R1,
                6  => PSButton::L2,
                7  => PSButton::R2,
                8  => PSButton::Select,
                9  => PSButton::Start,
                10 => PSButton::L3,
                11 => PSButton::R3,
                12 => PSButton::DPadUp,
                13 => PSButton::DPadDown,
                14 => PSButton::DPadLeft,
                _  => PSButton::DPadRight,
            };
            ButtonMapping { source_button: i, target_button: btn, modifier: None }
        }).collect(),
        axis_mappings: vec![
            AxisMapping { source_axis: 0, target_axis: PSAxis::LeftX,    scale: 1.0, deadzone: 0.05, invert: false },
            AxisMapping { source_axis: 1, target_axis: PSAxis::LeftY,    scale: 1.0, deadzone: 0.05, invert: false },
            AxisMapping { source_axis: 2, target_axis: PSAxis::RightX,   scale: 1.0, deadzone: 0.05, invert: false },
            AxisMapping { source_axis: 3, target_axis: PSAxis::RightY,   scale: 1.0, deadzone: 0.05, invert: false },
            AxisMapping { source_axis: 4, target_axis: PSAxis::L2Analog, scale: 1.0, deadzone: 0.02, invert: false },
            AxisMapping { source_axis: 5, target_axis: PSAxis::R2Analog, scale: 1.0, deadzone: 0.02, invert: false },
        ],
        virtual_buttons: vec![],
    }
}

// ─── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use detection::{DeviceInfo, DeviceType, Transport};

    fn dummy_device(dt: DeviceType) -> DeviceInfo {
        DeviceInfo {
            id: "test-device".into(),
            vid: 0x045E, pid: 0x0B12,
            name: "Test Controller".into(),
            manufacturer: "Test".into(),
            device_type: dt,
            buttons: 12, axes: 6, sensors: false,
            capabilities: vec![],
            firmware: None, bt_version: None, usb_version: None,
            transport: Transport::USB, connected: true,
        }
    }

    #[test]
    fn mapping_engine_new_ok() {
        let engine = MappingEngine::new();
        assert!(engine.is_ok());
    }

    #[test]
    fn auto_map_xbox() {
        let engine = MappingEngine::new().unwrap();
        let device = dummy_device(DeviceType::Xbox);
        let profile = engine.auto_map(&device);
        assert!(!profile.button_mappings.is_empty());
    }

    #[test]
    fn passthrough_neutral_axes() {
        let engine = MappingEngine::new().unwrap();
        let raw = RawHIDReport {
            axes: [127, 127, 127, 127, 0, 0, 127, 127],
            ..Default::default()
        };
        let report = engine.map_input("unknown-device", &raw);
        assert_eq!(report.left_x, 127);
        assert_eq!(report.left_y, 127);
    }

    #[test]
    fn deadzone_removes_small_deflection() {
        let mapping = AxisMapping {
            source_axis: 0,
            target_axis: PSAxis::LeftX,
            scale: 1.0,
            deadzone: 0.1,
            invert: false,
        };
        // 127 ± 5 is within 10% deadzone → should map to centre.
        let result = apply_axis(132, &mapping);
        assert_eq!(result, 127);
    }

    #[test]
    fn simulate_xbox_to_playstation_buttons() {
        let mut engine = MappingEngine::new().unwrap();
        
        // 1. We mock an Xbox device connecting
        let xbox_device = dummy_device(DeviceType::Xbox);
        
        // 2. We generate an auto-map profile for it and load it into the engine
        let profile = engine.auto_map(&xbox_device);
        let profile_id = profile.id.clone();
        engine.load_profile(profile);
        engine.set_active_profile(&xbox_device.id, &profile_id).unwrap();
        
        // 3. We simulate a raw HID report from an Xbox controller.
        // Let's press: 'A' (button 0), 'X' (button 2), and 'Start' (button 7)
        let mut raw_report = RawHIDReport::default();
        raw_report.buttons[0] = true; // A (Cross)
        raw_report.buttons[2] = true; // X (Square)
        raw_report.buttons[7] = true; // Start (Options)
        
        // 4. Send the raw input to the engine
        let ps_report = engine.map_input(&xbox_device.id, &raw_report);
        
        // 5. Verify the translation is 100% functional
        assert!(ps_report.buttons.contains(&PSButton::Cross), "Button A did not map to Cross");
        assert!(ps_report.buttons.contains(&PSButton::Square), "Button X did not map to Square");
        assert!(ps_report.buttons.contains(&PSButton::Options), "Button Start did not map to Options");
        assert!(!ps_report.buttons.contains(&PSButton::Circle), "Circle should not be pressed");
    }
}
