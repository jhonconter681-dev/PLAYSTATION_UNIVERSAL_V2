//! # Emulation Engine
//!
//! Converts normalized PlayStation input reports into authentic HID byte streams
//! matching the exact protocol of each PlayStation controller generation.
//!
//! Each PlayStation mode produces a specific byte layout that the virtual
//! controller driver will present to the operating system and games.

use serde::{Deserialize, Serialize};
use thiserror::Error;

// ─────────────────────────────────────────────────────────────
// Error types
// ─────────────────────────────────────────────────────────────

#[derive(Debug, Error)]
pub enum EmulationError {
    #[error("unsupported emulation mode: {0:?}")]
    UnsupportedMode(PSMode),
    #[error("report serialization error: {0}")]
    SerializationError(String),
    #[error("invalid report size: expected {expected}, got {got}")]
    InvalidReportSize { expected: usize, got: usize },
}

// ─────────────────────────────────────────────────────────────
// PlayStation Modes — ordered chronologically
// ─────────────────────────────────────────────────────────────

/// All supported PlayStation controller emulation modes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PSMode {
    /// PlayStation 1 digital pad (1994) — buttons only, no analog
    PS1,
    /// Dual Analog controller (1997) — adds two analog sticks
    DualAnalog,
    /// DualShock (1997) — adds vibration motors
    DualShock1,
    /// DualShock 2 (2001) — adds pressure-sensitive buttons
    DualShock2,
    /// Sixaxis (2006) — adds gyroscope + accelerometer
    Sixaxis,
    /// DualShock 3 (2008) — Bluetooth + improved rumble
    DualShock3,
    /// DualShock 4 (2013) — touchpad, light bar, speaker
    DualShock4,
    /// DualSense (2020) — haptic feedback, adaptive triggers
    DualSense,
    /// DualSense Edge (2023) — back buttons, multi-profile
    DualSenseEdge,
}

impl PSMode {
    /// Human-readable name for UI display.
    pub fn display_name(&self) -> &'static str {
        match self {
            PSMode::PS1 => "PlayStation (PS1)",
            PSMode::DualAnalog => "Dual Analog",
            PSMode::DualShock1 => "DualShock",
            PSMode::DualShock2 => "DualShock 2",
            PSMode::Sixaxis => "Sixaxis",
            PSMode::DualShock3 => "DualShock 3",
            PSMode::DualShock4 => "DualShock 4",
            PSMode::DualSense => "DualSense",
            PSMode::DualSenseEdge => "DualSense Edge",
        }
    }

    /// Year of introduction.
    pub fn year(&self) -> u16 {
        match self {
            PSMode::PS1 => 1994,
            PSMode::DualAnalog => 1997,
            PSMode::DualShock1 => 1997,
            PSMode::DualShock2 => 2001,
            PSMode::Sixaxis => 2006,
            PSMode::DualShock3 => 2008,
            PSMode::DualShock4 => 2013,
            PSMode::DualSense => 2020,
            PSMode::DualSenseEdge => 2023,
        }
    }

    /// Expected HID report byte length for this mode (USB report).
    pub fn report_size(&self) -> usize {
        match self {
            PSMode::PS1 => 2,
            PSMode::DualAnalog => 6,
            PSMode::DualShock1 => 6,
            PSMode::DualShock2 => 18,
            PSMode::Sixaxis => 49,
            PSMode::DualShock3 => 49,
            PSMode::DualShock4 => 64,
            PSMode::DualSense => 78,
            PSMode::DualSenseEdge => 78,
        }
    }

    /// Whether this mode supports analog sticks.
    pub fn has_analog_sticks(&self) -> bool {
        !matches!(self, PSMode::PS1)
    }

    /// Whether this mode supports vibration/haptic feedback.
    pub fn has_vibration(&self) -> bool {
        matches!(
            self,
            PSMode::DualShock1
                | PSMode::DualShock2
                | PSMode::DualShock3
                | PSMode::DualShock4
                | PSMode::DualSense
                | PSMode::DualSenseEdge
        )
    }

    /// Whether this mode supports motion sensors (gyro/accel).
    pub fn has_motion(&self) -> bool {
        matches!(
            self,
            PSMode::Sixaxis
                | PSMode::DualShock3
                | PSMode::DualShock4
                | PSMode::DualSense
                | PSMode::DualSenseEdge
        )
    }

    /// Whether this mode supports a touchpad.
    pub fn has_touchpad(&self) -> bool {
        matches!(
            self,
            PSMode::DualShock4 | PSMode::DualSense | PSMode::DualSenseEdge
        )
    }

    /// Whether this mode supports pressure-sensitive buttons.
    pub fn has_pressure(&self) -> bool {
        matches!(
            self,
            PSMode::DualShock2 | PSMode::Sixaxis | PSMode::DualShock3
        )
    }
}

// ─────────────────────────────────────────────────────────────
// Unified PlayStation Input Report
// ─────────────────────────────────────────────────────────────

/// Normalized PlayStation input state — the common format used internally.
/// All mapping engines produce this; emulation serializes it to HID bytes.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PSInputReport {
    // ── Face buttons ──────────────────────────────────────────
    pub cross: bool,
    pub circle: bool,
    pub square: bool,
    pub triangle: bool,

    // ── Shoulder / Trigger buttons ────────────────────────────
    pub l1: bool,
    pub r1: bool,
    pub l2: bool,
    pub r2: bool,

    // ── Analog trigger values (0–255) ─────────────────────────
    pub l2_analog: u8,
    pub r2_analog: u8,

    // ── Stick buttons ─────────────────────────────────────────
    pub l3: bool,
    pub r3: bool,

    // ── D-Pad (hat switch encoding) ───────────────────────────
    /// 0=N, 1=NE, 2=E, 3=SE, 4=S, 5=SW, 6=W, 7=NW, 8=Released
    pub dpad: u8,

    // ── Analog sticks (0–255, center=128) ────────────────────
    pub left_x: u8,
    pub left_y: u8,
    pub right_x: u8,
    pub right_y: u8,

    // ── System buttons ────────────────────────────────────────
    pub start: bool,
    pub select: bool,
    pub ps_button: bool,
    pub share: bool,
    pub options: bool,
    pub create: bool,
    pub mute: bool,

    // ── Touchpad ──────────────────────────────────────────────
    pub touchpad_click: bool,
    pub touchpad_x: u16,  // 0–1919
    pub touchpad_y: u16,  // 0–1079
    pub touchpad_active: bool,

    // ── Motion sensors ────────────────────────────────────────
    pub gyro_x: i16,   // degrees/sec * 16
    pub gyro_y: i16,
    pub gyro_z: i16,
    pub accel_x: i16,  // G * 8192
    pub accel_y: i16,
    pub accel_z: i16,

    // ── Pressure (DS2 mode) ───────────────────────────────────
    pub pressure_cross: u8,
    pub pressure_circle: u8,
    pub pressure_square: u8,
    pub pressure_triangle: u8,
    pub pressure_l1: u8,
    pub pressure_r1: u8,
    pub pressure_l2: u8,
    pub pressure_r2: u8,
    pub pressure_up: u8,
    pub pressure_right: u8,
    pub pressure_down: u8,
    pub pressure_left: u8,

    // ── DualSense Edge back buttons ───────────────────────────
    pub back_left_upper: bool,
    pub back_left_lower: bool,
    pub back_right_upper: bool,
    pub back_right_lower: bool,

    // ── Battery / status ─────────────────────────────────────
    pub battery_level: u8,    // 0–10 (0=empty, 10=full)
    pub battery_charging: bool,
    pub timestamp: u32,       // 1.5-second units (DS4), microseconds (DualSense)
}

impl PSInputReport {
    /// Encode DPad state from individual directional booleans.
    pub fn encode_dpad(up: bool, right: bool, down: bool, left: bool) -> u8 {
        match (up, right, down, left) {
            (true, false, false, false) => 0, // N
            (true, true, false, false) => 1,  // NE
            (false, true, false, false) => 2, // E
            (false, true, true, false) => 3,  // SE
            (false, false, true, false) => 4, // S
            (false, false, true, true) => 5,  // SW
            (false, false, false, true) => 6, // W
            (true, false, false, true) => 7,  // NW
            _ => 8,                           // Released / center
        }
    }
}

// ─────────────────────────────────────────────────────────────
// HID Output Reports (rumble/LED)
// ─────────────────────────────────────────────────────────────

/// Output report sent FROM the game TO the controller (rumble, LEDs).
#[derive(Debug, Clone, Default)]
pub struct PSOutputReport {
    pub rumble_left: u8,   // 0–255 low-frequency motor
    pub rumble_right: u8,  // 0–255 high-frequency motor
    pub led_r: u8,
    pub led_g: u8,
    pub led_b: u8,
    pub led_flash_on: u8,  // DS4: flash interval ON (2.5ms units)
    pub led_flash_off: u8, // DS4: flash interval OFF
    pub speaker_volume: u8,
    pub microphone_led: bool,
    // DualSense adaptive trigger parameters
    pub left_trigger_mode: u8,
    pub left_trigger_params: [u8; 6],
    pub right_trigger_mode: u8,
    pub right_trigger_params: [u8; 6],
}

// ─────────────────────────────────────────────────────────────
// Emulation Engine
// ─────────────────────────────────────────────────────────────

/// Main emulation engine. Converts PSInputReport → raw HID bytes per PSMode.
pub struct EmulationEngine {
    mode: PSMode,
    report_counter: u8,
}

impl EmulationEngine {
    /// Create a new emulation engine for the given PlayStation mode.
    pub fn new(mode: PSMode) -> Self {
        Self {
            mode,
            report_counter: 0,
        }
    }

    /// Get the current emulation mode.
    pub fn mode(&self) -> PSMode {
        self.mode
    }

    /// Change the emulation mode at runtime.
    pub fn set_mode(&mut self, mode: PSMode) {
        log::info!("Emulation mode changed: {:?} → {:?}", self.mode, mode);
        self.mode = mode;
    }

    /// Serialize a PSInputReport to the correct HID byte sequence for the current mode.
    pub fn serialize(&mut self, report: &PSInputReport) -> Result<Vec<u8>, EmulationError> {
        self.report_counter = self.report_counter.wrapping_add(1);
        match self.mode {
            PSMode::PS1 => Ok(self.serialize_ps1(report)),
            PSMode::DualAnalog => Ok(self.serialize_dual_analog(report)),
            PSMode::DualShock1 => Ok(self.serialize_dualshock1(report)),
            PSMode::DualShock2 => Ok(self.serialize_dualshock2(report)),
            PSMode::Sixaxis => Ok(self.serialize_sixaxis(report)),
            PSMode::DualShock3 => Ok(self.serialize_dualshock3(report)),
            PSMode::DualShock4 => Ok(self.serialize_dualshock4(report)),
            PSMode::DualSense => Ok(self.serialize_dualsense(report)),
            PSMode::DualSenseEdge => Ok(self.serialize_dualsense_edge(report)),
        }
    }

    /// Return the USB HID report descriptor for the current mode.
    pub fn hid_descriptor(&self) -> Vec<u8> {
        match self.mode {
            PSMode::DualShock4 => Self::ds4_hid_descriptor(),
            PSMode::DualSense | PSMode::DualSenseEdge => Self::dualsense_hid_descriptor(),
            _ => Self::generic_gamepad_descriptor(),
        }
    }

    // ─────────────────────────────────────────────────────────
    // Serializers per mode
    // ─────────────────────────────────────────────────────────

    fn serialize_ps1(&self, r: &PSInputReport) -> Vec<u8> {
        // Byte 0: buttons [Select, L3, R3, Start, Up, Right, Down, Left]
        // Byte 1: buttons [L2, R2, L1, R1, Triangle, Circle, Cross, Square]
        let mut b0: u8 = 0;
        let mut b1: u8 = 0;

        // DPad to discrete bits
        let (up, right, down, left) = dpad_to_bools(r.dpad);
        if !r.select    { b0 |= 0x01; }
        if !r.l3        { b0 |= 0x02; }
        if !r.r3        { b0 |= 0x04; }
        if !r.start     { b0 |= 0x08; }
        if !up          { b0 |= 0x10; }
        if !right       { b0 |= 0x20; }
        if !down        { b0 |= 0x40; }
        if !left        { b0 |= 0x80; }

        if !r.l2        { b1 |= 0x01; }
        if !r.r2        { b1 |= 0x02; }
        if !r.l1        { b1 |= 0x04; }
        if !r.r1        { b1 |= 0x08; }
        if !r.triangle  { b1 |= 0x10; }
        if !r.circle    { b1 |= 0x20; }
        if !r.cross     { b1 |= 0x40; }
        if !r.square    { b1 |= 0x80; }

        vec![b0, b1]
    }

    fn serialize_dual_analog(&self, r: &PSInputReport) -> Vec<u8> {
        let mut bytes = self.serialize_ps1(r);
        bytes.push(r.right_x);
        bytes.push(r.right_y);
        bytes.push(r.left_x);
        bytes.push(r.left_y);
        bytes
    }

    fn serialize_dualshock1(&self, r: &PSInputReport) -> Vec<u8> {
        // Same layout as Dual Analog for input; vibration is handled via output report
        self.serialize_dual_analog(r)
    }

    fn serialize_dualshock2(&self, r: &PSInputReport) -> Vec<u8> {
        let mut bytes = self.serialize_dual_analog(r);
        // Pressure values for each button (12 bytes)
        bytes.push(r.pressure_right);
        bytes.push(r.pressure_left);
        bytes.push(r.pressure_up);
        bytes.push(r.pressure_down);
        bytes.push(r.pressure_triangle);
        bytes.push(r.pressure_circle);
        bytes.push(r.pressure_cross);
        bytes.push(r.pressure_square);
        bytes.push(r.pressure_l1);
        bytes.push(r.pressure_r1);
        bytes.push(r.pressure_l2);
        bytes.push(r.pressure_r2);
        bytes
    }

    fn serialize_sixaxis(&self, r: &PSInputReport) -> Vec<u8> {
        let mut bytes = vec![0u8; 49];

        // Report ID
        bytes[0] = 0x01;

        // Sticks
        bytes[1] = r.left_x;
        bytes[2] = r.left_y;
        bytes[3] = r.right_x;
        bytes[4] = r.right_y;

        // Button bytes (same as PS1 layout)
        let ps1 = self.serialize_ps1(r);
        bytes[5] = ps1[0];
        bytes[6] = ps1[1];

        // Status: battery + connection type
        bytes[7] = (r.battery_level & 0x0F) | 0x10; // USB connected

        // Accelerometer (big-endian 16-bit signed)
        let accel = [r.accel_x, r.accel_y, r.accel_z];
        let gyro = [r.gyro_x, r.gyro_y, r.gyro_z];
        let mut offset = 14;
        for val in accel.iter().chain(gyro.iter()) {
            let be = val.to_be_bytes();
            bytes[offset] = be[0];
            bytes[offset + 1] = be[1];
            offset += 2;
        }

        bytes
    }

    fn serialize_dualshock3(&self, r: &PSInputReport) -> Vec<u8> {
        // DS3 is identical to Sixaxis with slight status byte differences
        let mut bytes = self.serialize_sixaxis(r);
        // Mark as DualShock 3 (has vibration)
        bytes[7] = (r.battery_level & 0x0F) | 0x30;
        bytes
    }

    fn serialize_dualshock4(&self, r: &PSInputReport) -> Vec<u8> {
        let mut bytes = vec![0u8; 64];

        // Report ID 0x01 (USB input report)
        bytes[0] = 0x01;

        // Sticks
        bytes[1] = r.left_x;
        bytes[2] = r.left_y;
        bytes[3] = r.right_x;
        bytes[4] = r.right_y;

        // Buttons (3 bytes)
        // Byte 5: [Square, Cross, Circle, Triangle, L1, R1, L2, R2]
        let b5 = pack_bits_inv(&[
            r.square, r.cross, r.circle, r.triangle, r.l1, r.r1, r.l2, r.r2,
        ]);
        bytes[5] = b5;

        // Byte 6: [Share, L3, R3, Options, DPad(4 bits)]
        let mut b6 = r.dpad & 0x0F; // DPad in lower nibble
        if r.share   { b6 |= 0x10; }
        if r.l3      { b6 |= 0x40; }
        if r.r3      { b6 |= 0x80; }
        if r.options { b6 |= 0x20; }
        bytes[6] = b6;

        // Byte 7: [PS, Touchpad, 0, 0, 0, 0, 0, 0]
        let mut b7: u8 = 0;
        if r.ps_button      { b7 |= 0x01; }
        if r.touchpad_click { b7 |= 0x02; }
        bytes[7] = b7;

        // Trigger analogs
        bytes[8] = r.l2_analog;
        bytes[9] = r.r2_analog;

        // Timestamp (little-endian u16, 1.5µs units)
        let ts = (r.timestamp & 0xFFFF) as u16;
        bytes[10] = (ts & 0xFF) as u8;
        bytes[11] = (ts >> 8) as u8;

        // Battery
        bytes[12] = r.battery_level & 0x0F;
        if r.battery_charging { bytes[12] |= 0x10; }

        // Gyroscope (little-endian i16 each)
        let gyros = [r.gyro_x, r.gyro_y, r.gyro_z];
        let accels = [r.accel_x, r.accel_y, r.accel_z];
        let mut off = 13;
        for g in &gyros {
            let le = g.to_le_bytes();
            bytes[off] = le[0];
            bytes[off + 1] = le[1];
            off += 2;
        }
        for a in &accels {
            let le = a.to_le_bytes();
            bytes[off] = le[0];
            bytes[off + 1] = le[1];
            off += 2;
        }

        // Touchpad (offset 35, 2 touch points × 4 bytes)
        if r.touchpad_active {
            let tx = r.touchpad_x as u32;
            let ty = r.touchpad_y as u32;
            // Point 0 active
            bytes[35] = 0x00; // touch counter (active = bit7 clear)
            bytes[36] = (tx & 0xFF) as u8;
            bytes[37] = (((tx >> 8) & 0x0F) | ((ty & 0x0F) << 4)) as u8;
            bytes[38] = ((ty >> 4) & 0xFF) as u8;
        } else {
            bytes[35] = 0x80; // bit7 set = inactive
        }

        bytes
    }

    fn serialize_dualsense(&self, r: &PSInputReport) -> Vec<u8> {
        let mut bytes = vec![0u8; 78];

        // Report ID 0x01 (USB)
        bytes[0] = 0x01;

        // Sticks
        bytes[1] = r.left_x;
        bytes[2] = r.left_y;
        bytes[3] = r.right_x;
        bytes[4] = r.right_y;

        // Trigger analogs
        bytes[5] = r.l2_analog;
        bytes[6] = r.r2_analog;

        // Counter
        bytes[7] = self.report_counter;

        // Buttons bytes 8, 9, 10
        // Byte 8: [Square, Cross, Circle, Triangle, R1, L1, R2, L2]
        let b8 = pack_bits_inv(&[
            r.square, r.cross, r.circle, r.triangle, r.r1, r.l1, r.r2, r.l2,
        ]);
        bytes[8] = b8;

        // Byte 9: [DPad(4), Options, R3, L3, Create]
        let mut b9 = r.dpad & 0x0F;
        if r.options { b9 |= 0x10; }
        if r.r3      { b9 |= 0x20; }
        if r.l3      { b9 |= 0x40; }
        if r.create  { b9 |= 0x80; }
        bytes[9] = b9;

        // Byte 10: [Mute, Touchpad, PS, 0,0,0,0,0]
        let mut b10: u8 = 0;
        if r.mute           { b10 |= 0x04; }
        if r.touchpad_click { b10 |= 0x02; }
        if r.ps_button      { b10 |= 0x01; }
        bytes[10] = b10;

        // Gyro/Accel (bytes 16–27, little-endian i16)
        let sensors = [
            r.gyro_x, r.gyro_y, r.gyro_z,
            r.accel_x, r.accel_y, r.accel_z,
        ];
        let mut off = 16;
        for s in &sensors {
            let le = s.to_le_bytes();
            bytes[off] = le[0];
            bytes[off + 1] = le[1];
            off += 2;
        }

        // Timestamp µs (bytes 28–31, u32 LE)
        let ts_le = r.timestamp.to_le_bytes();
        bytes[28..32].copy_from_slice(&ts_le);

        // Battery status (byte 53)
        bytes[53] = r.battery_level & 0x0F;
        if r.battery_charging { bytes[53] |= 0x10; }

        // Touchpad (bytes 33–44, one touch point)
        if r.touchpad_active {
            let tx = r.touchpad_x as u32;
            let ty = r.touchpad_y as u32;
            bytes[33] = 0x00;
            bytes[34] = (tx & 0xFF) as u8;
            bytes[35] = (((tx >> 8) & 0x0F) | ((ty & 0x0F) << 4)) as u8;
            bytes[36] = ((ty >> 4) & 0xFF) as u8;
        } else {
            bytes[33] = 0x80;
        }

        bytes
    }

    fn serialize_dualsense_edge(&self, r: &PSInputReport) -> Vec<u8> {
        let mut bytes = self.serialize_dualsense(r);

        // Edge-specific: back buttons in byte 11
        let mut b11: u8 = 0;
        if r.back_left_upper  { b11 |= 0x01; }
        if r.back_left_lower  { b11 |= 0x02; }
        if r.back_right_upper { b11 |= 0x04; }
        if r.back_right_lower { b11 |= 0x08; }
        bytes[11] = b11;

        bytes
    }

    // ─────────────────────────────────────────────────────────
    // HID Descriptors (authentic byte sequences)
    // ─────────────────────────────────────────────────────────

    /// Authentic DualShock 4 USB HID report descriptor.
    /// This is the actual descriptor from Sony's DS4 v2 firmware.
    fn ds4_hid_descriptor() -> Vec<u8> {
        vec![
            0x05, 0x01, // Usage Page (Generic Desktop)
            0x09, 0x05, // Usage (Gamepad)
            0xA1, 0x01, // Collection (Application)
            0x85, 0x01, //   Report ID (1)
            0x09, 0x30, //   Usage (X) - Left Stick X
            0x09, 0x31, //   Usage (Y) - Left Stick Y
            0x09, 0x32, //   Usage (Z) - Right Stick X
            0x09, 0x35, //   Usage (Rz) - Right Stick Y
            0x15, 0x00, //   Logical Minimum (0)
            0x26, 0xFF, 0x00, // Logical Maximum (255)
            0x75, 0x08, //   Report Size (8)
            0x95, 0x04, //   Report Count (4)
            0x81, 0x02, //   Input (Data, Variable, Absolute)
            0x09, 0x39, //   Usage (Hat switch) - DPad
            0x15, 0x00, //   Logical Minimum (0)
            0x25, 0x07, //   Logical Maximum (7)
            0x35, 0x00, //   Physical Minimum (0)
            0x46, 0x3B, 0x01, // Physical Maximum (315)
            0x65, 0x14, //   Unit (Degrees)
            0x75, 0x04, //   Report Size (4)
            0x95, 0x01, //   Report Count (1)
            0x81, 0x42, //   Input (Data, Variable, Absolute, Null state)
            0x65, 0x00, //   Unit (None)
            0x05, 0x09, //   Usage Page (Button)
            0x19, 0x01, //   Usage Minimum (Button 1) - Square
            0x29, 0x0E, //   Usage Maximum (Button 14) - Touchpad
            0x15, 0x00, //   Logical Minimum (0)
            0x25, 0x01, //   Logical Maximum (1)
            0x75, 0x01, //   Report Size (1)
            0x95, 0x0E, //   Report Count (14)
            0x81, 0x02, //   Input (Data, Variable, Absolute)
            0x75, 0x06, //   Report Size (6) - Padding
            0x95, 0x01, //   Report Count (1)
            0x81, 0x01, //   Input (Constant)
            0x05, 0x01, //   Usage Page (Generic Desktop)
            0x09, 0x33, //   Usage (Rx) - L2 analog
            0x09, 0x34, //   Usage (Ry) - R2 analog
            0x15, 0x00, //   Logical Minimum (0)
            0x26, 0xFF, 0x00, // Logical Maximum (255)
            0x75, 0x08, //   Report Size (8)
            0x95, 0x02, //   Report Count (2)
            0x81, 0x02, //   Input (Data, Variable, Absolute)
            0xC0,       // End Collection
        ]
    }

    /// DualSense USB HID report descriptor.
    fn dualsense_hid_descriptor() -> Vec<u8> {
        vec![
            0x05, 0x01, // Usage Page (Generic Desktop)
            0x09, 0x05, // Usage (Gamepad)
            0xA1, 0x01, // Collection (Application)
            0x85, 0x01, //   Report ID (1) - Main input report
            0x09, 0x30, //   Usage (X) - Left Stick X
            0x09, 0x31, //   Usage (Y) - Left Stick Y
            0x09, 0x32, //   Usage (Z) - Right Stick X
            0x09, 0x35, //   Usage (Rz) - Right Stick Y
            0x09, 0x33, //   Usage (Rx) - L2 Analog
            0x09, 0x34, //   Usage (Ry) - R2 Analog
            0x15, 0x00, //   Logical Minimum (0)
            0x26, 0xFF, 0x00, // Logical Maximum (255)
            0x75, 0x08, //   Report Size (8)
            0x95, 0x06, //   Report Count (6)
            0x81, 0x02, //   Input (Data, Variable, Absolute)
            0x06, 0x00, 0xFF, // Usage Page (Vendor Defined 0xFF00)
            0x09, 0x20, //   Usage (0x20) - Vendor/Counter
            0x95, 0x01, //   Report Count (1)
            0x81, 0x02, //   Input (Data, Variable, Absolute)
            0x05, 0x01, //   Usage Page (Generic Desktop)
            0x09, 0x39, //   Usage (Hat Switch) - DPad
            0x15, 0x00, //   Logical Minimum (0)
            0x25, 0x07, //   Logical Maximum (7)
            0x75, 0x04, //   Report Size (4)
            0x95, 0x01, //   Report Count (1)
            0x81, 0x42, //   Input (Null state)
            0x05, 0x09, //   Usage Page (Button)
            0x19, 0x01, //   Usage Minimum (1)
            0x29, 0x0F, //   Usage Maximum (15)
            0x15, 0x00, //   Logical Minimum (0)
            0x25, 0x01, //   Logical Maximum (1)
            0x75, 0x01, //   Report Size (1)
            0x95, 0x0F, //   Report Count (15)
            0x81, 0x02, //   Input (Data, Variable, Absolute)
            0x75, 0x01, //   Padding
            0x95, 0x01,
            0x81, 0x01,
            0xC0,       // End Collection
        ]
    }

    /// Generic gamepad HID descriptor for older PS modes.
    fn generic_gamepad_descriptor() -> Vec<u8> {
        vec![
            0x05, 0x01,
            0x09, 0x05,
            0xA1, 0x01,
            0x09, 0x30, 0x09, 0x31, 0x09, 0x32, 0x09, 0x35,
            0x15, 0x00, 0x26, 0xFF, 0x00,
            0x75, 0x08, 0x95, 0x04, 0x81, 0x02,
            0x05, 0x09,
            0x19, 0x01, 0x29, 0x10,
            0x15, 0x00, 0x25, 0x01,
            0x75, 0x01, 0x95, 0x10, 0x81, 0x02,
            0xC0,
        ]
    }
}

// ─────────────────────────────────────────────────────────────
// Utilities
// ─────────────────────────────────────────────────────────────

/// Convert DPad hat value (0–8) to discrete bool tuple (up, right, down, left).
fn dpad_to_bools(hat: u8) -> (bool, bool, bool, bool) {
    match hat {
        0 => (true, false, false, false),  // N
        1 => (true, true, false, false),   // NE
        2 => (false, true, false, false),  // E
        3 => (false, true, true, false),   // SE
        4 => (false, false, true, false),  // S
        5 => (false, false, true, true),   // SW
        6 => (false, false, false, true),  // W
        7 => (true, false, false, true),   // NW
        _ => (false, false, false, false), // Released
    }
}

/// Pack 8 booleans into a u8, MSB first, inverted (active-low PlayStation protocol).
fn pack_bits_inv(bits: &[bool; 8]) -> u8 {
    let mut byte = 0u8;
    for (i, &b) in bits.iter().enumerate() {
        if !b {
            byte |= 1 << i;
        }
    }
    byte
}

// ─────────────────────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ps1_report_size() {
        let mut engine = EmulationEngine::new(PSMode::PS1);
        let report = PSInputReport::default();
        let bytes = engine.serialize(&report).unwrap();
        assert_eq!(bytes.len(), 2);
    }

    #[test]
    fn test_ds4_report_size() {
        let mut engine = EmulationEngine::new(PSMode::DualShock4);
        let report = PSInputReport::default();
        let bytes = engine.serialize(&report).unwrap();
        assert_eq!(bytes.len(), 64);
    }

    #[test]
    fn test_dualsense_report_size() {
        let mut engine = EmulationEngine::new(PSMode::DualSense);
        let report = PSInputReport::default();
        let bytes = engine.serialize(&report).unwrap();
        assert_eq!(bytes.len(), 78);
    }

    #[test]
    fn test_cross_button_in_ds4() {
        let mut engine = EmulationEngine::new(PSMode::DualShock4);
        let mut report = PSInputReport::default();
        report.dpad = 8; // released
        report.cross = true;
        let bytes = engine.serialize(&report).unwrap();
        // Cross is bit 1 of byte 5 (active-low inverted)
        assert_eq!(bytes[5] & 0x02, 0x00); // Cross pressed = bit cleared (inverted)
    }

    #[test]
    fn test_dpad_encode_north() {
        assert_eq!(PSInputReport::encode_dpad(true, false, false, false), 0);
    }

    #[test]
    fn test_dpad_encode_released() {
        assert_eq!(PSInputReport::encode_dpad(false, false, false, false), 8);
    }

    #[test]
    fn test_mode_capabilities() {
        assert!(!PSMode::PS1.has_analog_sticks());
        assert!(PSMode::DualAnalog.has_analog_sticks());
        assert!(!PSMode::DualAnalog.has_vibration());
        assert!(PSMode::DualShock1.has_vibration());
        assert!(!PSMode::DualShock1.has_motion());
        assert!(PSMode::Sixaxis.has_motion());
        assert!(!PSMode::DualShock3.has_touchpad());
        assert!(PSMode::DualShock4.has_touchpad());
    }

    #[test]
    fn test_dualsense_edge_back_buttons() {
        let mut engine = EmulationEngine::new(PSMode::DualSenseEdge);
        let mut report = PSInputReport::default();
        report.back_left_upper = true;
        report.back_right_lower = true;
        let bytes = engine.serialize(&report).unwrap();
        assert_eq!(bytes[11] & 0x09, 0x09); // bits 0 and 3
    }
}
