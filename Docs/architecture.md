# PUCE — Architecture Document
## PlayStation Universal Controller Emulator — Technical Architecture

**Version:** 1.0.0  
**Date:** 2026-06-15  
**Status:** Production Blueprint

---

## Table of Contents

1. [System Overview](#1-system-overview)
2. [Layered Architecture](#2-layered-architecture)
3. [Core Engine Modules](#3-core-engine-modules)
4. [Data Flow](#4-data-flow)
5. [Virtual Controller Stack](#5-virtual-controller-stack)
6. [AI Subsystem](#6-ai-subsystem)
7. [Plugin Architecture](#7-plugin-architecture)
8. [Security Architecture](#8-security-architecture)
9. [Database Architecture](#9-database-architecture)
10. [UI Architecture](#10-ui-architecture)
11. [Platform Matrix](#11-platform-matrix)
12. [Performance Design](#12-performance-design)
13. [Inter-Process Communication](#13-inter-process-communication)
14. [Module Dependency Graph](#14-module-dependency-graph)

---

## 1. System Overview

PUCE operates as a **transparent input translation layer** between any physical input device and any software expecting a PlayStation controller. It achieves this through three fundamental operations:

```
Physical Device  ──▶  PUCE Core  ──▶  Virtual PS Controller
 (any HID device)        (Rust)         (OS-native virtual device)
```

The architecture prioritizes:
- **Sub-millisecond latency** — zero-copy input pipelines, lock-free ring buffers
- **Universal compatibility** — abstract device model, HID-standard compliance
- **Extensibility** — plugin ABI, OTA database updates, modular engines
- **Security** — signed binaries, signed plugins, verified OTA updates

---

## 2. Layered Architecture

```
╔══════════════════════════════════════════════════════════════════╗
║  LAYER 6 — USER INTERFACE (Flutter)                              ║
║  Platforms: Windows, Linux, macOS, Android, Android TV, iOS      ║
╠══════════════════════════════════════════════════════════════════╣
║  LAYER 5 — FFI BRIDGE (dart:ffi / cbindgen)                      ║
║  C-compatible ABI, JSON message protocol                         ║
╠══════════════════════════════════════════════════════════════════╣
║  LAYER 4 — ORCHESTRATION (puce_core)                             ║
║  Engine lifecycle, event bus, cross-module coordination          ║
╠═════════════════╦════════════════╦═════════════════╦════════════╣
║  LAYER 3A       ║  LAYER 3B      ║  LAYER 3C       ║ LAYER 3D  ║
║  DETECTION      ║  MAPPING       ║  EMULATION      ║ AI ENGINE ║
║  ENGINE         ║  ENGINE        ║  ENGINE         ║           ║
╠═════════════════╩════════════════╩═════════════════╩════════════╣
║  LAYER 2 — INFRASTRUCTURE                                        ║
║  SQLite DB │ Plugin System │ Security Module │ Event Bus         ║
╠══════════════════════════════════════════════════════════════════╣
║  LAYER 1 — HARDWARE ABSTRACTION                                  ║
║  HID (hidapi) │ Bluetooth (btleplug) │ USB raw │ Virtual Output  ║
╠══════════════════════════════════════════════════════════════════╣
║  LAYER 0 — OS / KERNEL                                           ║
║  ViGEm (Win) │ uinput (Linux) │ IOKit (macOS) │ InputMgr (And)  ║
╚══════════════════════════════════════════════════════════════════╝
```

---

## 3. Core Engine Modules

### 3.1 Detection Engine (`core/detection`)

**Responsibility:** Enumerate, identify, and monitor all input devices.

**Architecture:**
```
DeviceScanner
    ├── HIDScanner          (hidapi polling, 16ms interval)
    │     ├── USBHIDScanner
    │     └── BLEHIDScanner
    ├── BluetoothScanner    (btleplug, event-driven)
    └── DeviceRegistry      (in-memory + SQLite cache)
```

**Key data structures:**
```rust
pub struct DeviceInfo {
    pub id: String,               // UUID assigned by PUCE
    pub vendor_id: u16,
    pub product_id: u16,
    pub name: String,
    pub manufacturer: String,
    pub device_type: DeviceType,
    pub button_count: u8,
    pub axis_count: u8,
    pub sensors: Vec<SensorType>,
    pub capabilities: Vec<Capability>,
    pub firmware: Option<String>,
    pub bt_version: Option<String>,
    pub usb_version: Option<String>,
    pub battery_level: Option<f32>,  // 0.0..1.0
    pub latency_ms: f32,
    pub connection_type: ConnectionType,
}
```

**Detection algorithm:**
1. Query HID bus for all enumerated devices
2. Match VID+PID against local SQLite database
3. If no match, attempt generic HID classification
4. Emit `DeviceConnected` event via tokio broadcast channel
5. Start polling goroutine for device data
6. Emit `DeviceDisconnected` on removal

### 3.2 Mapping Engine (`core/mapping`)

**Responsibility:** Translate raw device HID reports into normalized PS input reports.

**Architecture:**
```
MappingEngine
    ├── ProfileLoader       (SQLite ↔ MappingProfile)
    ├── AutoMapper          (heuristic + AI-assisted)
    ├── InputProcessor      (raw bytes → normalized values)
    │     ├── ButtonProcessor
    │     ├── AxisProcessor   (deadzone, curve, calibration)
    │     └── SensorProcessor (gyro, accel)
    └── VirtualInputGenerator  (generates missing capabilities)
          ├── VirtualTouchpad   (from mouse/trackpad)
          ├── VirtualGyro       (from mouse movement)
          └── VirtualStick      (from keyboard WASD)
```

**Mapping data flow:**
```
Raw HID bytes
     │
     ▼ InputProcessor.parse()
Normalized HID values {buttons: u64, axes: [f32;8], sensors: [f32;6]}
     │
     ▼ MappingEngine.apply_profile()
PS Input Report {cross, circle, square, triangle, l1, r1, l2, r2, ...}
     │
     ▼ VirtualInputGenerator.fill_missing()
Complete PS Report (all fields populated, even virtual ones)
```

### 3.3 Emulation Engine (`core/emulation`)

**Responsibility:** Format the PS input report as authentic PlayStation HID bytes for the virtual controller.

**Mode state machine:**
```
PS1 ──▶ DualAnalog ──▶ DualShock ──▶ DualShock2
                                          │
                                          ▼
                              Sixaxis ──▶ DualShock3
                                          │
                                          ▼
                                      DualShock4
                                          │
                                          ▼
                                      DualSense
                                          │
                                          ▼
                                    DualSense Edge
```

Each mode adds fields to the report. Higher modes are strict supersets of lower modes.

**HID Report structure per mode:**

| Mode | Report Size | Key Additions |
|------|-------------|---------------|
| PS1 | 2 bytes | 8 buttons + DPad |
| DualAnalog | 6 bytes | + 2 sticks |
| DualShock | 6 bytes | + vibration output |
| DualShock 2 | 18 bytes | + pressure per button |
| Sixaxis | 49 bytes | + gyro + accel |
| DualShock 3 | 49 bytes | + USB vibration |
| DualShock 4 | 64 bytes | + touchpad + lightbar |
| DualSense | 78 bytes | + haptics + adaptive triggers |
| DualSense Edge | 78+ bytes | + back buttons + profiles |

### 3.4 Virtual Controller (`core/virtual_controller`)

**Responsibility:** Present the emulated PS controller to the OS as a real HID device.

**Platform implementations:**

| Platform | Backend | Method |
|----------|---------|--------|
| Windows | ViGEm Bus | Named pipe protocol to ViGEm driver |
| Linux | uinput | `/dev/uinput` ioctl + write() |
| macOS | IOKit | HIDDeviceClient virtual device |
| Android | InputManager | UinputManager (API 31+) |
| iOS | MFi / Entitlement | Requires Apple entitlement |

---

## 4. Data Flow

### 4.1 Input Pipeline (Physical → Virtual)

```
[Physical Device]
      │ USB/BT HID Report (raw bytes, ~1-8ms)
      ▼
[Detection Engine] — assigns DeviceInfo, starts polling
      │ RawHIDReport { device_id, bytes: Vec<u8>, timestamp }
      ▼
[Mapping Engine] — applies MappingProfile
      │ PSInputReport { all PS buttons/axes/sensors }
      ▼
[AI Engine] — applies calibration, drift correction
      │ CalibratedPSReport (corrected analog values)
      ▼
[Emulation Engine] — serializes to authentic HID bytes
      │ Vec<u8> (mode-specific PS HID report)
      ▼
[Virtual Controller] — OS write
      │ OS HID report submission (~0.1ms)
      ▼
[Game / Emulator]
```

### 4.2 Output Pipeline (Rumble/Haptics)

```
[Game]
      │ Rumble/LED output report
      ▼
[Virtual Controller] — receives output report
      │ OutputReport { rumble_left, rumble_right, led_color }
      ▼
[Emulation Engine] — decodes output
      │ HapticCommand { left_motor, right_motor }
      ▼
[Physical Device] — forwards rumble to real device
```

### 4.3 Event Bus

All engines communicate via a centralized `tokio::broadcast` channel:

```rust
pub enum PuceEvent {
    DeviceConnected(DeviceInfo),
    DeviceDisconnected(String),       // device_id
    DeviceUpdated(DeviceInfo),
    InputReport(RawHIDReport),
    EmulationModeChanged(String, PSMode),  // device_id, new_mode
    CalibrationComplete(String, CalibrationData),
    DriftDetected(String, Vec2),
    ProfileLoaded(MappingProfile),
    PluginLoaded(String),
    UpdateAvailable(UpdateInfo),
    Error(PuceError),
}
```

---

## 5. Virtual Controller Stack

### Windows — ViGEm

```
PUCE (user space)
    │
    │ Named pipe: \\.\pipe\ViGEmBus
    ▼
ViGEm Bus Driver (kernel driver: ViGEmBus.sys)
    │
    ▼
Virtual HID Device (appears as "Sony DualShock 4" or "Sony DualSense")
    │
    ▼
Game / Application (via DirectInput / XInput / Raw HID)
```

ViGEm supports two target types:
- **XUSB** (Xbox 360 compatible) — used as intermediate on some titles
- **DS4** (DualShock 4) — primary target for PS emulation

### Linux — uinput

```
PUCE (user space)
    │
    │ write() to /dev/uinput
    ▼
uinput kernel module
    │
    ▼
/dev/input/eventN (virtual input device)
    │
    ▼
Game / Application (via evdev / SDL2 / libinput)
```

Button keycode mapping:
```
Cross     → BTN_SOUTH  (304)
Circle    → BTN_EAST   (305)
Square    → BTN_WEST   (308)
Triangle  → BTN_NORTH  (307)
L1        → BTN_TL     (310)
R1        → BTN_TR     (311)
L2        → BTN_TL2    (312) + ABS_Z
R2        → BTN_TR2    (313) + ABS_RZ
L3        → BTN_THUMBL (317)
R3        → BTN_THUMBR (318)
Share     → BTN_SELECT (314)
Options   → BTN_START  (315)
PS        → BTN_MODE   (316)
Left Stick X → ABS_X
Left Stick Y → ABS_Y
Right Stick X → ABS_RX
Right Stick Y → ABS_RY
DPad X    → ABS_HAT0X
DPad Y    → ABS_HAT0Y
```

### macOS — IOKit

```
PUCE (user space)
    │
    │ IOKit HIDDeviceClient API
    ▼
HID Event System
    │
    ▼
Virtual HID Device (appears in IORegistry)
    │
    ▼
Game (via IOKit / GameController.framework / SDL2)
```

---

## 6. AI Subsystem

```
AI Engine (core/ai_engine)
    ├── StickCalibrator
    │     ├── CircularBufferSampler (1024 samples)
    │     ├── CenterCalculator (moving average)
    │     └── DeadzoneOptimizer (adaptive threshold)
    │
    ├── DriftDetector
    │     ├── BaselineEstimator (resting position)
    │     ├── DriftVelocityCalculator (rate of drift)
    │     └── DriftCompensator (real-time correction)
    │
    ├── LatencyOptimizer
    │     ├── PollingRateSelector (125/250/500/1000 Hz)
    │     ├── JitterFilter (exponential moving average)
    │     └── PredictiveBuffering (Kalman filter)
    │
    ├── ProfileSuggester
    │     ├── DeviceClassifier (classify by capabilities)
    │     ├── GameTypeDetector (detect by window/process name)
    │     └── MappingRecommender (lookup table + heuristics)
    │
    └── AutoMapper
          ├── ButtonCountHeuristic
          ├── AxisCountHeuristic
          └── VendorHeuristic (brand-specific mappings)
```

**Drift Detection Algorithm:**
1. Collect 500ms of stick samples at rest
2. Compute centroid of samples
3. If centroid deviates >2% from center: drift detected
4. Apply real-time correction vector = (detected_drift - true_center)

**Kalman Filter for Gyro:**
State vector: `[angle, angular_velocity]`  
Measurement: raw gyro reading  
Prediction: physics model (angle += velocity * dt)  
Update: weighted average of prediction and measurement

---

## 7. Plugin Architecture

### Plugin ABI (C-compatible)

```c
// Every plugin must export these symbols:
bool         puce_plugin_init(void);
void         puce_plugin_shutdown(void);
PucePluginInfo* puce_plugin_get_info(void);
bool         puce_plugin_handles_device(DeviceInfoC* device);
MappingProfileC* puce_plugin_get_mapping(DeviceInfoC* device);
```

### Plugin Loading Process

```
1. Discovery: scan ~/.puce/plugins/*.puce_plugin
2. Signature check: verify .sig sidecar with embedded public key
3. Load: dlopen() / LoadLibrary()
4. Version check: PUCE_PLUGIN_VERSION compatibility
5. Init: call puce_plugin_init()
6. Register: add to plugin registry (SQLite)
7. Active: plugin now available for device matching
```

### Plugin Lifecycle

```
Discovered ──▶ Verified ──▶ Loaded ──▶ Initialized ──▶ Active
                                                          │
                                        Shutdown ◀────────┘
                                             │
                                         Unloaded
```

---

## 8. Security Architecture

### Signature Chain

```
PUCE Root CA (offline, air-gapped)
    │ signs
    ▼
Plugin Signing Key (ed25519 keypair)
    │ signs
    ▼
plugin.puce_plugin + plugin.puce_plugin.sig
```

Verification at load time:
```rust
let public_key = include_bytes!("../keys/puce_public.key");
let signature = fs::read(plugin_path.with_extension("sig"))?;
let plugin_bytes = fs::read(plugin_path)?;
verifier.verify(public_key, &plugin_bytes, &signature)?;
```

### OTA Update Security

```
Update Server
    │ HTTPS (TLS 1.3 minimum)
    ▼
Update Manifest (JSON + ed25519 signature)
    │ Verify manifest signature
    ▼
Delta Package Download
    │ Verify package SHA-256 + signature
    ▼
Apply Update (atomic rename)
    │ Keep backup of previous version
    ▼
Restart Service
```

---

## 9. Database Architecture

### Schema

```sql
-- Devices HID database (500+ entries, updatable via OTA)
CREATE TABLE devices (
    id              INTEGER PRIMARY KEY AUTOINCREMENT,
    vendor_id       INTEGER NOT NULL,
    product_id      INTEGER NOT NULL,
    name            TEXT NOT NULL,
    manufacturer    TEXT NOT NULL,
    device_type     TEXT NOT NULL,
    button_count    INTEGER DEFAULT 0,
    axis_count      INTEGER DEFAULT 0,
    sensors         TEXT DEFAULT '[]',  -- JSON array
    capabilities    TEXT DEFAULT '[]',  -- JSON array
    firmware        TEXT,
    bt_version      TEXT,
    usb_version     TEXT,
    created_at      DATETIME DEFAULT CURRENT_TIMESTAMP,
    updated_at      DATETIME DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(vendor_id, product_id)
);

-- User mapping profiles
CREATE TABLE mapping_profiles (
    id              TEXT PRIMARY KEY,  -- UUID
    name            TEXT NOT NULL,
    device_vid      INTEGER,
    device_pid      INTEGER,
    ps_mode         TEXT NOT NULL,
    button_mappings TEXT DEFAULT '[]',  -- JSON
    axis_mappings   TEXT DEFAULT '[]',  -- JSON
    virtual_buttons TEXT DEFAULT '[]',  -- JSON
    is_default      INTEGER DEFAULT 0,
    created_at      DATETIME DEFAULT CURRENT_TIMESTAMP,
    updated_at      DATETIME DEFAULT CURRENT_TIMESTAMP
);

-- App settings key-value store
CREATE TABLE user_settings (
    key         TEXT PRIMARY KEY,
    value       TEXT NOT NULL,
    updated_at  DATETIME DEFAULT CURRENT_TIMESTAMP
);

-- Plugin registry
CREATE TABLE plugin_registry (
    id              TEXT PRIMARY KEY,
    name            TEXT NOT NULL,
    version         TEXT NOT NULL,
    author          TEXT,
    path            TEXT NOT NULL,
    signature_hash  TEXT NOT NULL,
    enabled         INTEGER DEFAULT 1,
    installed_at    DATETIME DEFAULT CURRENT_TIMESTAMP
);

-- Per-device calibration
CREATE TABLE calibration_data (
    device_id       TEXT PRIMARY KEY,
    center_x        REAL DEFAULT 0.0,
    center_y        REAL DEFAULT 0.0,
    dead_zone       REAL DEFAULT 0.05,
    max_radius      REAL DEFAULT 1.0,
    right_center_x  REAL DEFAULT 0.0,
    right_center_y  REAL DEFAULT 0.0,
    calibrated_at   DATETIME DEFAULT CURRENT_TIMESTAMP
);

-- Migration tracking
CREATE TABLE schema_migrations (
    version     INTEGER PRIMARY KEY,
    applied_at  DATETIME DEFAULT CURRENT_TIMESTAMP
);
```

### Database Update Strategy

The device HID database can be updated independently from the app:
1. Server publishes `hid_db_v{N}.json` + signature
2. App downloads on launch (if update available)
3. Merges new entries into local SQLite (UPSERT by VID+PID)
4. User custom profiles never overwritten

---

## 10. UI Architecture

### Flutter Widget Tree

```
PuceApp (MaterialApp.router)
    └── GoRouter
          ├── HomeScreen
          │     ├── AppBar (PUCE logo, connection badge)
          │     ├── DeviceCard (current device)
          │     ├── PSModeSelector (horizontal scroll)
          │     ├── ControllerVisualizer (CustomPainter)
          │     ├── LatencyChart (fl_chart)
          │     └── EmulationToggleButton
          │
          ├── MappingScreen
          │     ├── ControllerVisualizer (interactive)
          │     ├── MappingList (scrollable)
          │     │     └── MappingRow × N
          │     └── ProfileActions (save/load/auto)
          │
          ├── ProfilesScreen
          │     ├── SearchBar
          │     ├── ProfileGrid
          │     │     └── ProfileCard × N
          │     └── CreateProfileFAB
          │
          ├── SettingsScreen
          │     ├── AppearanceSection
          │     ├── PerformanceSection
          │     ├── EmulationSection
          │     ├── PluginsSection
          │     └── AboutSection
          │
          └── DeviceDetailScreen
                ├── DeviceHeader
                ├── InfoGrid
                ├── SensorsList
                ├── BatteryIndicator
                └── LatencyChart
```

### State Management (Riverpod)

```dart
// Provider hierarchy
deviceListProvider          → StreamProvider<List<DeviceInfo>>
selectedDeviceProvider      → StateProvider<DeviceInfo?>
emulationModeProvider       → StateNotifierProvider<PSMode>
mappingProfileProvider      → FutureProvider<MappingProfile>
calibrationDataProvider     → StateNotifierProvider<CalibrationData>
settingsProvider            → StateNotifierProvider<AppSettings>
pluginListProvider          → FutureProvider<List<PluginInfo>>
latencyStreamProvider       → StreamProvider<double>
```

### TV Navigation

PUCE supports full D-pad navigation for Android TV / Google TV / Fire TV:
- `FocusTraversalGroup` wraps all interactive sections
- `FocusNode` assigned to all buttons, cards, and list items
- `RawKeyboardListener` intercepts DPAD_UP/DOWN/LEFT/RIGHT/CENTER
- Visual focus indicator: blue glow border on focused element
- `NavigationRail` replaced by bottom drawer on TV (D-pad accessible)

---

## 11. Platform Matrix

| Feature | Windows | Linux | macOS | Android | iOS | Android TV |
|---------|---------|-------|-------|---------|-----|------------|
| USB HID | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| Bluetooth | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| Virtual DS4 | ✅ ViGEm | ✅ uinput | ✅ IOKit | ✅ InputMgr | ⚠️ MFi | ✅ uinput |
| Virtual DualSense | ✅ ViGEm | ✅ uinput | ✅ IOKit | ⚠️ API31+ | ❌ | ⚠️ |
| Rumble output | ✅ | ✅ | ✅ | ✅ | ❌ | ✅ |
| Adaptive triggers | ✅ DS5Windows | ❌ | ❌ | ❌ | ❌ | ❌ |
| Touchpad virtual | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| AI engine | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| Plugins | ✅ | ✅ | ✅ | ⚠️ | ❌ | ⚠️ |
| OTA updates | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |

---

## 12. Performance Design

### Zero-Copy Input Pipeline

```rust
// Ring buffer for input reports (lock-free)
struct InputRingBuffer {
    buffer: [RawHIDReport; 256],  // Power of 2 for mask optimization
    write_idx: AtomicUsize,
    read_idx: AtomicUsize,
}
```

### Polling Architecture

```
Main thread: UI polling (60 fps)
     ├── HID polling thread (1000 Hz = 1ms)
     │     └── Writes to InputRingBuffer
     ├── Processing thread (priority: THREAD_PRIORITY_TIME_CRITICAL)
     │     └── Reads InputRingBuffer → processes → writes output
     └── Output thread (priority: THREAD_PRIORITY_TIME_CRITICAL)
           └── Reads output buffer → submits to virtual controller
```

### Latency Budget

```
Physical device USB polling:        1.0 ms  (1000 Hz mode)
HID read syscall:                   0.1 ms
Mapping + emulation processing:     0.1 ms
Virtual controller write syscall:   0.1 ms
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
Total USB path:                    ~1.3 ms

Bluetooth HCI stack:                2-4 ms  (Bluetooth 5.0)
BT HID polling:                     5-10 ms (Connection Interval)
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
Total BT path:                     ~7-14 ms
```

---

## 13. Inter-Process Communication

### FFI Protocol (Rust ↔ Flutter)

All communication uses C-compatible function calls:

```c
// Synchronous calls (return immediately)
const char* puce_get_version(void);
int         puce_init(void);
void        puce_shutdown(void);

// Async queries (returns JSON string, caller must free)
const char* puce_get_devices(void);
const char* puce_get_status(void);
const char* puce_get_profiles(void);

// Commands (returns 0 on success, error code on failure)
int puce_start_detection(void);
int puce_stop_detection(void);
int puce_set_emulation_mode(const char* device_id, int mode);
int puce_apply_profile(const char* device_id, const char* profile_id);
int puce_start_calibration(const char* device_id);

// Memory management
void puce_free_string(const char* ptr);
```

### JSON Message Format

```json
// puce_get_devices() response
{
  "devices": [
    {
      "id": "550e8400-e29b-41d4-a716-446655440000",
      "vendor_id": 1356,
      "product_id": 3302,
      "name": "DualSense Wireless Controller",
      "manufacturer": "Sony Interactive Entertainment",
      "device_type": "PlayStation",
      "button_count": 17,
      "axis_count": 6,
      "sensors": ["gyroscope", "accelerometer", "touchpad"],
      "capabilities": ["haptics", "adaptive_triggers", "microphone"],
      "firmware": "0142",
      "bt_version": "5.0",
      "usb_version": "USB 3.0",
      "battery_level": 0.87,
      "latency_ms": 1.2,
      "is_connected": true,
      "connection_type": "USB"
    }
  ]
}
```

---

## 14. Module Dependency Graph

```
puce_core
    ├── detection
    │     ├── hidapi (external)
    │     ├── btleplug (external)
    │     └── database
    ├── mapping
    │     ├── detection
    │     ├── ai_engine
    │     └── database
    ├── emulation
    │     └── mapping
    ├── virtual_controller
    │     └── emulation
    ├── ai_engine
    │     └── database
    ├── plugin_system
    │     ├── security
    │     └── database
    └── security
          └── (ed25519-dalek, sha2, external)
```

All external dependencies are managed via `[workspace.dependencies]` in the root `Cargo.toml` with pinned versions for reproducible builds.

---

*Architecture document version 1.0.0 — PlayStation Universal Controller Emulator*
