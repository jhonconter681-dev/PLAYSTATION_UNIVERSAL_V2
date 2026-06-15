# PUCE User Manual
## PlayStation Universal Controller Emulator

**Version 1.0.0** | **June 2026**

---

## Welcome to PUCE

PUCE (PlayStation Universal Controller Emulator) lets you use **any controller, keyboard, or mouse** as an authentic PlayStation controller. Whether you're playing on a PC, Android TV, or mobile device, PUCE makes your games think you have a real DualSense, DualShock 4, or any PlayStation controller of your choice.

---

## Table of Contents

1. [System Requirements](#system-requirements)
2. [Installation](#installation)
3. [First Launch](#first-launch)
4. [Connecting Your Device](#connecting-your-device)
5. [Choosing an Emulation Mode](#choosing-an-emulation-mode)
6. [Button Mapping](#button-mapping)
7. [Profiles](#profiles)
8. [AI Features](#ai-features)
9. [Settings](#settings)
10. [Troubleshooting](#troubleshooting)
11. [FAQ](#faq)

---

## 1. System Requirements

### Windows
- Windows 10 version 1903 or later
- Windows 11 (all versions)
- ViGEm Bus Driver (installed automatically)
- 100 MB free disk space
- .NET 6+ (for installer)

### Linux
- Kernel 5.4 or later (Ubuntu 20.04+, Fedora 32+, Arch)
- uinput kernel module (usually pre-installed)
- User in `input` group: `sudo usermod -aG input $USER`
- 80 MB free disk space

### macOS
- macOS 12 Monterey or later
- Apple Silicon (M1+) or Intel x64
- 100 MB free disk space

### Android
- Android 5.0 (Lollipop) API 21 or later
- Bluetooth 4.0+ or USB OTG support
- 60 MB free disk space

### Android TV / Google TV / Fire TV
- Android TV 9.0 or later
- USB or Bluetooth input device
- D-pad remote for navigation

---

## 2. Installation

### Windows

1. Download `PUCE-Setup.exe` from the official website
2. Run the installer (may require administrator permission)
3. The installer automatically installs **ViGEm Bus Driver** — the virtual controller layer
4. Follow the setup wizard
5. PUCE launches automatically after installation

> **Note:** If Windows Defender SmartScreen warns about the installer, click "More info" then "Run anyway". PUCE is digitally signed but SmartScreen may show this for new publishers.

### Linux

#### AppImage (Recommended)
```bash
# Download AppImage
chmod +x PUCE-x86_64.AppImage
./PUCE-x86_64.AppImage
```

#### Debian/Ubuntu (.deb)
```bash
sudo dpkg -i puce_1.0.0_amd64.deb
sudo apt-get install -f  # Fix any dependency issues
```

#### Fedora/RHEL (.rpm)
```bash
sudo rpm -i puce-1.0.0.x86_64.rpm
```

**Required: uinput access**
```bash
# Add yourself to the input group (requires logout/login)
sudo usermod -aG input $USER

# Or for immediate access (session only):
sudo chmod 666 /dev/uinput
```

### macOS

1. Download `PUCE.dmg`
2. Double-click the DMG to mount it
3. Drag PUCE to your Applications folder
4. Right-click PUCE in Applications → Open (first time only, to bypass Gatekeeper)
5. Go to **System Preferences → Privacy & Security → Input Monitoring** and allow PUCE

### Android

1. Download `PUCE.apk`
2. On your device: **Settings → Security → Unknown Sources** (enable)
3. Open the downloaded APK and tap Install
4. Grant USB/Bluetooth permissions when prompted

### Android TV / Google TV / Fire TV

1. Install "Downloader" app from your TV's app store
2. Open Downloader and enter the PUCE APK download URL
3. Install the APK
4. Navigate with D-pad — PUCE is fully optimized for remote control

---

## 3. First Launch

When PUCE opens for the first time, you'll see the **Setup Wizard**:

### Step 1: Platform Detection
PUCE detects your platform and configures the virtual controller backend automatically.

### Step 2: Connect a Device
Connect your controller, keyboard, or any input device. PUCE shows a live list of detected devices.

### Step 3: Choose PlayStation Mode
Select which PlayStation controller you want to emulate (PS1, DualShock 4, DualSense, etc.).

### Step 4: Start
Click **Start Emulation** — your device now appears to all games as a PlayStation controller!

---

## 4. Connecting Your Device

### USB Connection
1. Plug your controller (or keyboard) into your computer
2. PUCE detects it within 1-2 seconds
3. The device appears in the **Connected Devices** panel on the home screen
4. A green dot indicates active connection

### Bluetooth Connection
1. Put your controller in pairing mode (consult your controller's manual)
2. On your computer: **Settings → Bluetooth** → pair the device normally
3. PUCE detects the paired device automatically
4. Bluetooth devices show a BT icon with signal strength indicator

### Supported Devices (auto-detected)

| Category | Examples |
|----------|---------|
| PlayStation | DualSense, DualShock 4, DualShock 3, DualShock 2 |
| Xbox | Series X/S, Elite 2, One, 360 |
| Nintendo | Switch Pro, Joy-Con |
| 8BitDo | SN30 Pro, Pro 2, Ultimate |
| Logitech | F310, F510, F710, Gamepad F Series |
| Razer | Wolverine, Kishi, Raion |
| Keyboard | Any USB/BT keyboard (WASD mapping) |
| Mouse | Any USB/BT mouse (movement = sticks) |

### Device Info Panel

When a device is detected, the **Device Card** on the home screen shows:
- 🎮 Device name and manufacturer
- 🔋 Battery level (if supported)
- ⚡ Latency in milliseconds (color-coded: green <5ms, yellow <15ms, red >15ms)
- 📡 Connection type (USB or Bluetooth)
- 🔧 Firmware version (if available)

---

## 5. Choosing an Emulation Mode

Tap the **PlayStation Mode** selector on the home screen to choose:

### PS1 — PlayStation (1994)
- D-Pad, ✕ ○ □ △
- L1, L2, R1, R2
- Start, Select
- No analog sticks

**Best for:** Classic PS1 emulators (DuckStation, ePSXe)

### Dual Analog (1997)
- All PS1 inputs
- Left and Right analog sticks
- Analog button

**Best for:** Games that use the early analog layout

### DualShock (1997)
- All Dual Analog inputs
- Vibration/rumble support

**Best for:** Classic PS1/PS2 games with rumble

### DualShock 2 (2001)
- All DualShock inputs
- Pressure-sensitive buttons (every button has analog depth)

**Best for:** PS2 games that use button pressure (e.g., Gran Turismo)

### Sixaxis (2006)
- All DualShock inputs
- Gyroscope (tilt detection)
- Accelerometer

**Best for:** Motion-control PS3 games

### DualShock 3 (2008)
- All Sixaxis inputs
- Bluetooth connectivity
- Improved vibration

**Best for:** PS3 games via emulator (RPCS3)

### DualShock 4 (2013)
- All DS3 inputs
- **Touchpad** (swipe, click, two-finger gestures)
- **Light Bar** (color changes per game)
- Built-in speaker
- Share button
- Options button

**Best for:** PS4 games on PC (via Chiaki/PS Remote Play), emulators, games with DS4 features

### DualSense (2020)
- All DS4 inputs
- **Haptic feedback** (realistic textures, impacts)
- **Adaptive Triggers** (L2/R2 resistance simulation)
- Built-in microphone
- Create button

**Best for:** PS5 games on PC, DualSense-enhanced titles

### DualSense Edge (2023)
- All DualSense inputs
- Remappable back buttons (×4)
- Multiple saved profiles on device
- Trigger travel adjustment
- Stick dead zone profiles

**Best for:** Competitive gaming, custom layouts, esports titles

---

## 6. Button Mapping

Go to the **Mapping** screen to customize button assignments.

### Viewing Current Mapping

The left panel shows a visual PlayStation controller. The **currently assigned buttons** are highlighted in blue. Buttons with no input assigned are grayed out.

### Changing a Button Assignment

1. Tap/click the button you want to reassign in the visual controller
2. A dialog opens: "Press any button on your physical device"
3. Press the button you want to use
4. The mapping is saved automatically

### Mapping Keyboard Keys

When using a keyboard:

| PlayStation Button | Default Key |
|-------------------|-------------|
| Cross (✕) | K |
| Circle (○) | L |
| Square (□) | J |
| Triangle (△) | I |
| L1 | Q |
| R1 | E |
| L2 | 1 |
| R2 | 3 |
| D-Pad Up | W or Arrow Up |
| D-Pad Down | S or Arrow Down |
| D-Pad Left | A or Arrow Left |
| D-Pad Right | D or Arrow Right |
| Left Stick | WASD |
| Right Stick | Mouse movement |
| Start | Enter |
| Select | Backspace |
| PS Button | Escape |

### Axis Configuration

For each analog stick axis, you can configure:
- **Dead Zone:** Area around center that registers as 0 (default: 8%)
- **Saturation:** Point at which max value is reached (default: 95%)
- **Curve:** Response curve (Linear, Ease-In, Ease-Out, S-Curve)
- **Invert:** Flip the axis direction

### Virtual Inputs

If your device is missing PlayStation features, PUCE creates them virtually:

| Missing Feature | Virtual Solution |
|----------------|-----------------|
| No touchpad | Mouse movement in touchpad mode (hold Shift) |
| No gyroscope | Mouse movement simulates tilt |
| No sticks | WASD keys |
| No analog triggers | Keyboard L2/R2 keys (digital, max value) |

---

## 7. Profiles

Profiles save your complete mapping configuration so you can switch between them instantly.

### Creating a Profile

1. Go to **Profiles** screen
2. Tap **+ New Profile**
3. Name your profile (e.g., "GT7 Wheel", "Keyboard FPS")
4. Select the source device
5. Select the target PlayStation mode
6. Map your buttons
7. Tap **Save**

### Using Profiles

On the home screen, the current profile is shown below the device card. Tap it to switch profiles instantly.

### Importing/Exporting

- **Export:** Tap ··· on any profile → Export → saves as `.puce` file
- **Import:** Tap **Import** button → select `.puce` file
- Share profiles with other users!

### Auto-Profile

PUCE AI can automatically suggest a profile when you connect a new device:
1. Connect your device
2. PUCE shows "Profile Suggested" notification
3. Tap **Apply** to use the suggested profile
4. Tap **Customize** to review and edit first

---

## 8. AI Features

### Auto-Calibration

Over time, analog sticks can drift (register movement when physically centered). PUCE AI continuously monitors for drift:

1. **Detection:** If drift is detected, a warning appears: "Stick drift detected on Left Stick"
2. **Auto-correct:** PUCE automatically compensates with an opposite offset
3. **Manual calibrate:** Go to **Device → Calibrate** for manual full calibration

**Manual Calibration Steps:**
1. Place your controller on a flat surface
2. **Do not touch any inputs** for 3 seconds
3. Tap **Calibrate Center**
4. Move both sticks to all extremes slowly
5. Tap **Calibrate Range**
6. Done! Your sticks are now perfectly calibrated

### Latency Optimization

PUCE AI sets the optimal polling rate for your device:
- USB gamepads: 1000 Hz (1ms)
- Bluetooth controllers: 250 Hz (4ms)
- Keyboards/mice: 1000 Hz

You can override this in **Settings → Performance**.

### Auto-Mapping AI

When connecting a new/unknown device, PUCE AI analyzes the device's button count, axis count, and vendor to suggest the best mapping. Accuracy varies by device familiarity.

---

## 9. Settings

### Appearance
- **Theme:** Dark (default) / Light / System
- **Accent Color:** Choose from PlayStation blue, gold, purple, or custom
- **UI Scale:** Auto (recommended) / 80% / 100% / 125% / 150%

### Performance
- **Polling Rate:** Auto (recommended) / 125 Hz / 250 Hz / 500 Hz / 1000 Hz
- **Latency Mode:** Standard / Low Latency / Ultra Low Latency (disables some UI)
- **Input Buffer:** 0 / 1 / 2 frames (lower = less latency but more jitter)

### Emulation
- **Default Mode:** PS mode shown on first connection
- **AI Auto-Map:** Enable/disable profile suggestions
- **AI Drift Correction:** Enable/disable real-time drift compensation
- **Vibration:** Enable/disable output rumble
- **Haptic Intensity:** 0-100% (DualSense mode)

### Plugins
- List of installed plugins with enable/disable toggles
- **Install Plugin:** Opens file picker for `.puce_plugin` files
- **Plugin Store:** Browse community plugins (internet required)

### Updates
- **Current Version:** Shows PUCE version + HID database version
- **Check for Updates:** Manual update check
- **Auto-Update:** Enable automatic updates
- **Update Channel:** Stable / Beta / Nightly

### Advanced
- **Log Level:** Off / Error / Warning / Info / Debug
- **Export Logs:** Save logs for support/debugging
- **Reset Settings:** Reset all settings to defaults
- **Clear Database:** Remove all custom profiles (IRREVERSIBLE)

---

## 10. Troubleshooting

### Device Not Detected

**Windows:**
- Ensure ViGEm Bus Driver is installed: run `sc query ViGEmBus`
- Try a different USB port
- Check Device Manager for HID errors

**Linux:**
- Verify uinput access: `ls -la /dev/uinput`
- Add to input group: `sudo usermod -aG input $USER` then log out/in
- Check `dmesg` for USB errors

**macOS:**
- Allow Input Monitoring in System Preferences → Security & Privacy

**Android:**
- Enable USB OTG in phone settings
- Try reconnecting Bluetooth

### High Latency Warning

- Switch from Bluetooth to USB if possible
- Set Polling Rate to 1000 Hz in Settings → Performance
- Close other apps (Android)
- Disable Low Power mode

### Virtual Controller Not Appearing in Game

**Windows:**
- Confirm ViGEm Bus is running: Device Manager → Human Interface Devices
- Try "DS4Windows" compatibility mode in Settings
- Some games require XInput (Xbox mode) — enable in Settings

**Linux:**
- Check `/dev/input/event*` for new device: `ls -la /dev/input/`
- Use `jstest /dev/input/jsX` to verify buttons

### Stick Drift

1. Enable AI Drift Correction: Settings → Emulation → AI Drift Correction ✓
2. If persists: Device → Calibrate → full recalibration
3. Last resort: Settings → Advanced → Reset Calibration for this device

### App Crashes on Launch

1. Check that all permissions are granted
2. Delete app data and reinstall
3. Export logs before reinstalling: Settings → Advanced → Export Logs
4. Report the log file at github.com/puce-project/puce/issues

---

## 11. FAQ

**Q: Will PUCE ban me in online games?**
A: PUCE presents itself as a genuine PlayStation controller at the HID level. Anti-cheat systems cannot distinguish it from a real controller. However, PUCE does not provide any gameplay advantage — it only remaps inputs. Always check your game's Terms of Service.

**Q: Does PUCE work with PS Remote Play?**
A: Yes! Set PUCE to DualSense mode and PS Remote Play will detect it as a real DualSense, including haptic feedback and adaptive triggers (on supported titles).

**Q: Can I use two controllers at once?**
A: Yes. PUCE supports up to 4 simultaneous devices (Player 1-4), each in its own emulation mode.

**Q: Does the light bar/LED work on DualShock 4 mode?**
A: The Light Bar color sent by the game is received by PUCE. If your physical controller has RGB LEDs and the driver supports it, PUCE will forward the color. This varies by controller.

**Q: How do I make Gyro work on a controller that doesn't have one?**
A: Enable **Virtual Gyro** in Settings → Emulation. By default, mouse movement simulates gyro rotation. Adjust sensitivity in Settings → Emulation → Virtual Gyro Sensitivity.

**Q: Is PUCE free?**
A: PUCE core is open source and free. The Plugin Store (for community plugins) may offer both free and premium plugins.

**Q: Will PUCE support the PS6 controller when it releases?**
A: Yes. The plugin system allows adding support for new controllers without updating the main app. A PS6 plugin will be released as soon as the controller specs are publicly available.

---

*PUCE User Manual v1.0.0 — PlayStation Universal Controller Emulator*  
*© 2026 PUCE Project. Not affiliated with Sony Interactive Entertainment.*
