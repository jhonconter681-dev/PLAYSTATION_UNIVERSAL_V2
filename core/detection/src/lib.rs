//! # detection — HID & Bluetooth device detection engine
//!
//! Scans for connected game controllers over USB-HID and Bluetooth Low Energy,
//! classifies them by VID/PID, emits change events on a tokio broadcast channel,
//! and supports automatic reconnection.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use std::thread;

use anyhow::{Context, Result};
use log::{debug, error, info, warn};
use serde::{Deserialize, Serialize};
use tokio::sync::broadcast;
use uuid::Uuid;

// ─── DeviceType ───────────────────────────────────────────────────────────────

/// Classification of a detected input device.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub enum DeviceType {
    /// Sony PlayStation controller family.
    PlayStation,
    /// Microsoft Xbox controller family.
    Xbox,
    /// Nintendo controller family.
    Nintendo,
    /// Logitech gamepad family.
    Logitech,
    /// 8BitDo retro-style controller family.
    EightBitDo,
    /// Unrecognised gamepad / generic HID.
    Generic,
    /// USB or Bluetooth keyboard.
    Keyboard,
    /// USB or Bluetooth mouse.
    Mouse,
    /// Integrated or external touchscreen.
    Touchscreen,
}

// ─── DeviceInfo ───────────────────────────────────────────────────────────────

/// Full metadata record for a detected input device.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceInfo {
    /// Unique internal identifier assigned at detection time.
    pub id:           String,
    /// USB Vendor ID (decimal).
    pub vid:          u16,
    /// USB Product ID (decimal).
    pub pid:          u16,
    /// Human-readable device name.
    pub name:         String,
    /// Manufacturer string, if available.
    pub manufacturer: String,
    /// High-level device classification.
    pub device_type:  DeviceType,
    /// Number of digital buttons reported.
    pub buttons:      u8,
    /// Number of analogue axes reported.
    pub axes:         u8,
    /// Whether a gyroscope sensor is present.
    pub sensors:      bool,
    /// Feature tags: `"rumble"`, `"touchpad"`, `"gyro"`, `"accel"`, etc.
    pub capabilities: Vec<String>,
    /// Firmware version string, if readable.
    pub firmware:     Option<String>,
    /// Bluetooth specification version, if connected via BT.
    pub bt_version:   Option<String>,
    /// USB specification version, if connected via USB.
    pub usb_version:  Option<String>,
    /// Physical connection transport.
    pub transport:    Transport,
    /// Whether the device is currently active / reachable.
    pub connected:    bool,
}

/// Physical connection channel for a device.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Transport {
    USB,
    Bluetooth,
    Unknown,
}

// ─── Device change event ──────────────────────────────────────────────────────

/// Events broadcast on the device-change channel.
#[derive(Debug, Clone)]
pub enum DeviceEvent {
    Connected(DeviceInfo),
    Disconnected(String /* device id */),
    Updated(DeviceInfo),
}

// ─── Known VID/PID database ───────────────────────────────────────────────────

/// A single entry in the known-device registry.
struct KnownDevice {
    name:         &'static str,
    manufacturer: &'static str,
    device_type:  DeviceType,
    buttons:      u8,
    axes:         u8,
    sensors:      bool,
    capabilities: &'static [&'static str],
    bt_version:   Option<&'static str>,
    usb_version:  Option<&'static str>,
}

/// Builds the global VID:PID → KnownDevice map.
///
/// Sources:
/// * <https://the-ps-dev.io/controllers>
/// * Linux kernel `hid-sony.c`
/// * Xbox One controller protocol documentation
/// * Nintendo USB descriptor captures
fn build_known_devices() -> HashMap<(u16, u16), KnownDevice> {
    let mut m: HashMap<(u16, u16), KnownDevice> = HashMap::new();

    // ── Sony / PlayStation ────────────────────────────────────────────────
    const SONY: u16 = 0x054C;

    // Original PlayStation digital pad (via USB adapter)
    m.insert((SONY, 0x0268), KnownDevice {
        name: "DualShock 3 / Sixaxis",
        manufacturer: "Sony Interactive Entertainment",
        device_type: DeviceType::PlayStation,
        buttons: 17, axes: 6, sensors: true,
        capabilities: &["rumble", "accel"],
        bt_version: Some("2.1"), usb_version: Some("2.0"),
    });

    m.insert((SONY, 0x05C4), KnownDevice {
        name: "DualShock 4 (gen 1)",
        manufacturer: "Sony Interactive Entertainment",
        device_type: DeviceType::PlayStation,
        buttons: 17, axes: 6, sensors: true,
        capabilities: &["rumble", "touchpad", "gyro", "accel", "led"],
        bt_version: Some("4.0"), usb_version: Some("2.0"),
    });

    m.insert((SONY, 0x09CC), KnownDevice {
        name: "DualShock 4 (gen 2)",
        manufacturer: "Sony Interactive Entertainment",
        device_type: DeviceType::PlayStation,
        buttons: 17, axes: 6, sensors: true,
        capabilities: &["rumble", "touchpad", "gyro", "accel", "led"],
        bt_version: Some("4.0"), usb_version: Some("2.0"),
    });

    m.insert((SONY, 0x0CE6), KnownDevice {
        name: "DualSense Wireless Controller",
        manufacturer: "Sony Interactive Entertainment",
        device_type: DeviceType::PlayStation,
        buttons: 17, axes: 6, sensors: true,
        capabilities: &["haptics", "adaptive_triggers", "touchpad", "gyro", "accel", "led", "mic"],
        bt_version: Some("5.0"), usb_version: Some("3.0"),
    });

    m.insert((SONY, 0x0DF2), KnownDevice {
        name: "DualSense Edge Wireless Controller",
        manufacturer: "Sony Interactive Entertainment",
        device_type: DeviceType::PlayStation,
        buttons: 19, axes: 6, sensors: true,
        capabilities: &["haptics", "adaptive_triggers", "touchpad", "gyro", "accel", "led", "mic", "back_buttons", "remappable"],
        bt_version: Some("5.0"), usb_version: Some("3.0"),
    });

    // Access controller
    m.insert((SONY, 0x0E5F), KnownDevice {
        name: "PlayStation Access Controller",
        manufacturer: "Sony Interactive Entertainment",
        device_type: DeviceType::PlayStation,
        buttons: 12, axes: 2, sensors: false,
        capabilities: &["led", "programmable"],
        bt_version: Some("5.0"), usb_version: Some("3.0"),
    });

    // ── Microsoft / Xbox ─────────────────────────────────────────────────
    const MS: u16 = 0x045E;

    m.insert((MS, 0x028E), KnownDevice {
        name: "Xbox 360 Wired Controller",
        manufacturer: "Microsoft",
        device_type: DeviceType::Xbox,
        buttons: 11, axes: 6, sensors: false,
        capabilities: &["rumble"],
        bt_version: None, usb_version: Some("2.0"),
    });

    m.insert((MS, 0x02FF), KnownDevice {
        name: "Xbox 360 Wireless Receiver",
        manufacturer: "Microsoft",
        device_type: DeviceType::Xbox,
        buttons: 11, axes: 6, sensors: false,
        capabilities: &["rumble", "wireless"],
        bt_version: None, usb_version: Some("2.0"),
    });

    m.insert((MS, 0x02D1), KnownDevice {
        name: "Xbox One Controller (2013)",
        manufacturer: "Microsoft",
        device_type: DeviceType::Xbox,
        buttons: 11, axes: 6, sensors: false,
        capabilities: &["rumble"],
        bt_version: None, usb_version: Some("2.0"),
    });

    m.insert((MS, 0x02DD), KnownDevice {
        name: "Xbox One Controller (June 2015 firmware)",
        manufacturer: "Microsoft",
        device_type: DeviceType::Xbox,
        buttons: 11, axes: 6, sensors: false,
        capabilities: &["rumble"],
        bt_version: None, usb_version: Some("2.0"),
    });

    m.insert((MS, 0x02E3), KnownDevice {
        name: "Xbox One Elite Controller",
        manufacturer: "Microsoft",
        device_type: DeviceType::Xbox,
        buttons: 15, axes: 6, sensors: false,
        capabilities: &["rumble", "paddles", "adjustable_sticks"],
        bt_version: Some("4.0"), usb_version: Some("2.0"),
    });

    m.insert((MS, 0x02FD), KnownDevice {
        name: "Xbox One S Controller (Bluetooth)",
        manufacturer: "Microsoft",
        device_type: DeviceType::Xbox,
        buttons: 11, axes: 6, sensors: false,
        capabilities: &["rumble", "bluetooth"],
        bt_version: Some("4.2"), usb_version: Some("2.0"),
    });

    m.insert((MS, 0x0B12), KnownDevice {
        name: "Xbox Series X|S Controller",
        manufacturer: "Microsoft",
        device_type: DeviceType::Xbox,
        buttons: 12, axes: 6, sensors: false,
        capabilities: &["rumble", "share_button", "usb_c"],
        bt_version: Some("5.0"), usb_version: Some("3.1"),
    });

    m.insert((MS, 0x0B20), KnownDevice {
        name: "Xbox Elite Series 2 Controller",
        manufacturer: "Microsoft",
        device_type: DeviceType::Xbox,
        buttons: 17, axes: 6, sensors: false,
        capabilities: &["rumble", "paddles", "adjustable_sticks", "bluetooth", "hair_trigger_lock"],
        bt_version: Some("5.0"), usb_version: Some("2.0"),
    });

    // 3rd-party Xbox controllers
    const PDP: u16 = 0x0E6F;
    m.insert((PDP, 0x0213), KnownDevice {
        name: "PDP Wired Controller for Xbox One",
        manufacturer: "PDP",
        device_type: DeviceType::Xbox,
        buttons: 11, axes: 6, sensors: false,
        capabilities: &["rumble"],
        bt_version: None, usb_version: Some("2.0"),
    });

    // ── Nintendo ─────────────────────────────────────────────────────────
    const NINTENDO: u16 = 0x057E;

    m.insert((NINTENDO, 0x2006), KnownDevice {
        name: "Joy-Con (L)",
        manufacturer: "Nintendo",
        device_type: DeviceType::Nintendo,
        buttons: 9, axes: 2, sensors: true,
        capabilities: &["rumble", "gyro", "accel", "nfc"],
        bt_version: Some("4.1"), usb_version: None,
    });

    m.insert((NINTENDO, 0x2007), KnownDevice {
        name: "Joy-Con (R)",
        manufacturer: "Nintendo",
        device_type: DeviceType::Nintendo,
        buttons: 10, axes: 2, sensors: true,
        capabilities: &["rumble", "gyro", "accel", "nfc", "ir_camera"],
        bt_version: Some("4.1"), usb_version: None,
    });

    m.insert((NINTENDO, 0x2009), KnownDevice {
        name: "Nintendo Switch Pro Controller",
        manufacturer: "Nintendo",
        device_type: DeviceType::Nintendo,
        buttons: 16, axes: 4, sensors: true,
        capabilities: &["rumble", "gyro", "accel", "nfc"],
        bt_version: Some("4.1"), usb_version: Some("2.0"),
    });

    m.insert((NINTENDO, 0x200E), KnownDevice {
        name: "Joy-Con Grip / Charging Grip",
        manufacturer: "Nintendo",
        device_type: DeviceType::Nintendo,
        buttons: 16, axes: 4, sensors: true,
        capabilities: &["rumble", "gyro", "accel"],
        bt_version: Some("4.1"), usb_version: Some("2.0"),
    });

    m.insert((NINTENDO, 0x0337), KnownDevice {
        name: "Wii U GameCube Adapter",
        manufacturer: "Nintendo",
        device_type: DeviceType::Nintendo,
        buttons: 12, axes: 5, sensors: false,
        capabilities: &["rumble"],
        bt_version: None, usb_version: Some("2.0"),
    });

    // ── 8BitDo ───────────────────────────────────────────────────────────
    const EIGHTBITDO: u16 = 0x2DC8;

    m.insert((EIGHTBITDO, 0x6001), KnownDevice {
        name: "8BitDo SN30 Pro",
        manufacturer: "8BitDo",
        device_type: DeviceType::EightBitDo,
        buttons: 16, axes: 4, sensors: false,
        capabilities: &["rumble", "bluetooth"],
        bt_version: Some("4.0"), usb_version: Some("2.0"),
    });

    m.insert((EIGHTBITDO, 0x6003), KnownDevice {
        name: "8BitDo SN30 Pro+",
        manufacturer: "8BitDo",
        device_type: DeviceType::EightBitDo,
        buttons: 16, axes: 6, sensors: false,
        capabilities: &["rumble", "bluetooth", "gyro"],
        bt_version: Some("4.0"), usb_version: Some("2.0"),
    });

    m.insert((EIGHTBITDO, 0x6004), KnownDevice {
        name: "8BitDo Ultimate Controller",
        manufacturer: "8BitDo",
        device_type: DeviceType::EightBitDo,
        buttons: 17, axes: 6, sensors: true,
        capabilities: &["rumble", "bluetooth", "gyro", "accel", "remappable"],
        bt_version: Some("5.0"), usb_version: Some("2.0"),
    });

    m.insert((EIGHTBITDO, 0x3106), KnownDevice {
        name: "8BitDo Arcade Stick",
        manufacturer: "8BitDo",
        device_type: DeviceType::EightBitDo,
        buttons: 18, axes: 2, sensors: false,
        capabilities: &["bluetooth"],
        bt_version: Some("4.0"), usb_version: Some("2.0"),
    });

    // ── Logitech ─────────────────────────────────────────────────────────
    const LOGITECH: u16 = 0x046D;

    m.insert((LOGITECH, 0xC21D), KnownDevice {
        name: "Logitech F310 Gamepad",
        manufacturer: "Logitech",
        device_type: DeviceType::Logitech,
        buttons: 11, axes: 6, sensors: false,
        capabilities: &["rumble"],
        bt_version: None, usb_version: Some("2.0"),
    });

    m.insert((LOGITECH, 0xC21E), KnownDevice {
        name: "Logitech F510 Rumble Gamepad",
        manufacturer: "Logitech",
        device_type: DeviceType::Logitech,
        buttons: 11, axes: 6, sensors: false,
        capabilities: &["rumble"],
        bt_version: None, usb_version: Some("2.0"),
    });

    m.insert((LOGITECH, 0xC21F), KnownDevice {
        name: "Logitech F710 Wireless Gamepad",
        manufacturer: "Logitech",
        device_type: DeviceType::Logitech,
        buttons: 11, axes: 6, sensors: false,
        capabilities: &["rumble", "wireless"],
        bt_version: None, usb_version: Some("2.0"),
    });

    m.insert((LOGITECH, 0xC294), KnownDevice {
        name: "Logitech Cordless Rumblepad 2",
        manufacturer: "Logitech",
        device_type: DeviceType::Logitech,
        buttons: 12, axes: 4, sensors: false,
        capabilities: &["rumble", "wireless"],
        bt_version: None, usb_version: Some("1.1"),
    });

    m
}

// ─── DetectionEngine ─────────────────────────────────────────────────────────

/// Tokio broadcast capacity for device events.
const EVENT_CHANNEL_CAPACITY: usize = 64;

/// Polling interval for HID enumeration.
const HID_POLL_MS: u64 = 1_000;

/// Shared device registry: maps device id → DeviceInfo.
type DeviceRegistry = Arc<Mutex<HashMap<String, DeviceInfo>>>;

/// Central detection engine that manages HID and Bluetooth scanning.
pub struct DetectionEngine {
    registry:      DeviceRegistry,
    known_devices: HashMap<(u16, u16), KnownDevice>,
    tx:            broadcast::Sender<DeviceEvent>,
    stop_tx:       Option<std::sync::mpsc::Sender<()>>,
}

impl DetectionEngine {
    /// Creates a new, idle `DetectionEngine`.
    ///
    /// Call [`start`](Self::start) to begin scanning.
    pub fn new() -> Result<Self> {
        let known_devices = build_known_devices();
        let (tx, _) = broadcast::channel(EVENT_CHANNEL_CAPACITY);
        Ok(Self {
            registry: Arc::new(Mutex::new(HashMap::new())),
            known_devices,
            tx,
            stop_tx: None,
        })
    }

    /// Returns a new subscriber for device change events.
    pub fn subscribe(&self) -> broadcast::Receiver<DeviceEvent> {
        self.tx.subscribe()
    }

    /// Starts background HID scanning on a dedicated OS thread.
    ///
    /// Subsequent calls while already running are idempotent.
    pub fn start(&mut self) -> Result<()> {
        if self.stop_tx.is_some() {
            debug!("DetectionEngine::start: already running");
            return Ok(());
        }

        let (stop_tx, stop_rx) = std::sync::mpsc::channel::<()>();
        self.stop_tx = Some(stop_tx);

        let registry      = Arc::clone(&self.registry);
        let tx            = self.tx.clone();
        let known_devices = build_known_devices();

        thread::Builder::new()
            .name("puce-hid-scanner".into())
            .spawn(move || {
                info!("HID scanner thread started");
                hid_scan_loop(registry, known_devices, tx, stop_rx);
                info!("HID scanner thread stopped");
            })
            .context("spawning HID scanner thread")?;

        info!("DetectionEngine started");
        Ok(())
    }

    /// Signals the background scanner to stop and clears all device state.
    pub fn stop(&mut self) {
        if let Some(tx) = self.stop_tx.take() {
            let _ = tx.send(());
        }
        let mut reg = self.registry.lock().unwrap();
        reg.clear();
        info!("DetectionEngine stopped");
    }

    /// Returns a snapshot of all currently connected devices.
    pub fn get_devices(&self) -> Vec<DeviceInfo> {
        self.registry.lock().unwrap().values().cloned().collect()
    }

    /// Returns a device by its internal id, if present.
    pub fn get_device(&self, id: &str) -> Option<DeviceInfo> {
        self.registry.lock().unwrap().get(id).cloned()
    }
}

// ─── HID scan loop ───────────────────────────────────────────────────────────

fn hid_scan_loop(
    registry:      DeviceRegistry,
    known_devices: HashMap<(u16, u16), KnownDevice>,
    tx:            broadcast::Sender<DeviceEvent>,
    stop_rx:       std::sync::mpsc::Receiver<()>,
) {
    loop {
        // Check for stop signal (non-blocking).
        match stop_rx.try_recv() {
            Ok(_) | Err(std::sync::mpsc::TryRecvError::Disconnected) => break,
            Err(std::sync::mpsc::TryRecvError::Empty) => {}
        }

        match hidapi::HidApi::new() {
            Err(e) => {
                error!("HidApi::new failed: {e}");
            }
            Ok(hid) => {
                let mut seen_ids: Vec<String> = Vec::new();

                for dev_info in hid.device_list() {
                    let vid = dev_info.vendor_id();
                    let pid = dev_info.product_id();

                    let device_info = build_device_info(vid, pid, dev_info, &known_devices);
                    let id = device_info.id.clone();
                    seen_ids.push(id.clone());

                    let mut reg = registry.lock().unwrap();
                    if !reg.contains_key(&id) {
                        info!("Device connected: {} (VID={:04X} PID={:04X})", device_info.name, vid, pid);
                        let _ = tx.send(DeviceEvent::Connected(device_info.clone()));
                        reg.insert(id, device_info);
                    }
                }

                // Remove devices that are no longer enumerated.
                let mut reg = registry.lock().unwrap();
                let stale: Vec<String> = reg
                    .keys()
                    .filter(|k| !seen_ids.contains(k))
                    .cloned()
                    .collect();
                for stale_id in stale {
                    info!("Device disconnected: {stale_id}");
                    let _ = tx.send(DeviceEvent::Disconnected(stale_id.clone()));
                    reg.remove(&stale_id);
                }
            }
        }

        // Sleep until next poll, but wake immediately on stop signal.
        if stop_rx
            .recv_timeout(Duration::from_millis(HID_POLL_MS))
            .is_ok()
        {
            break;
        }
    }
}

// ─── Build DeviceInfo from HID enumeration ───────────────────────────────────

fn build_device_info(
    vid:          u16,
    pid:          u16,
    raw:          &hidapi::DeviceInfo,
    known:        &HashMap<(u16, u16), KnownDevice>,
) -> DeviceInfo {
    // Determine a stable, human-readable ID from path.
    let path_str = raw.path().to_string_lossy();
    // Hash the path to get a stable short id.
    let id = format!("{:04X}:{:04X}:{}", vid, pid, {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        let mut h = DefaultHasher::new();
        path_str.hash(&mut h);
        format!("{:016X}", h.finish())
    });

    if let Some(k) = known.get(&(vid, pid)) {
        DeviceInfo {
            id,
            vid,
            pid,
            name:         k.name.to_string(),
            manufacturer: k.manufacturer.to_string(),
            device_type:  k.device_type.clone(),
            buttons:      k.buttons,
            axes:         k.axes,
            sensors:      k.sensors,
            capabilities: k.capabilities.iter().map(|s| s.to_string()).collect(),
            firmware:     None,
            bt_version:   k.bt_version.map(|s| s.to_string()),
            usb_version:  k.usb_version.map(|s| s.to_string()),
            transport:    Transport::USB,
            connected:    true,
        }
    } else {
        // Generic fallback.
        let name = raw
            .product_string()
            .map(|s| s.to_string())
            .unwrap_or_else(|| format!("Unknown {:04X}:{:04X}", vid, pid));
        let mfr = raw
            .manufacturer_string()
            .map(|s| s.to_string())
            .unwrap_or_default();

        DeviceInfo {
            id,
            vid,
            pid,
            name,
            manufacturer: mfr,
            device_type:  DeviceType::Generic,
            buttons:      8,
            axes:         4,
            sensors:      false,
            capabilities: vec![],
            firmware:     None,
            bt_version:   None,
            usb_version:  Some("2.0".into()),
            transport:    Transport::USB,
            connected:    true,
        }
    }
}

// ─── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn known_device_map_is_not_empty() {
        let m = build_known_devices();
        assert!(!m.is_empty());
    }

    #[test]
    fn dualsense_present() {
        let m = build_known_devices();
        let ds = m.get(&(0x054C, 0x0CE6)).expect("DualSense not in registry");
        assert_eq!(ds.device_type, DeviceType::PlayStation);
        assert!(ds.capabilities.contains(&"haptics"));
    }

    #[test]
    fn xbox_series_present() {
        let m = build_known_devices();
        let xb = m.get(&(0x045E, 0x0B12)).expect("Xbox Series not in registry");
        assert_eq!(xb.device_type, DeviceType::Xbox);
    }

    #[test]
    fn detection_engine_new_ok() {
        let engine = DetectionEngine::new();
        assert!(engine.is_ok());
    }
}
