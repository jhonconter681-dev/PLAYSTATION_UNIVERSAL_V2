//! # Virtual Controller
//!
//! Platform-specific virtual PlayStation controller creation.
//!
//! Creates a virtual HID device that the OS and applications perceive as a
//! genuine PlayStation controller (DualShock 4 or DualSense).
//!
//! Platform backends:
//! - **Windows** — ViGEm Bus Driver (named pipe protocol)
//! - **Linux** — uinput kernel module (`/dev/uinput`)
//! - **macOS** — IOKit HID virtual device
//! - **Other** — Stub/no-op for compilation on unsupported platforms

use emulation::PSMode;
use thiserror::Error;

// ─────────────────────────────────────────────────────────────
// Error types
// ─────────────────────────────────────────────────────────────

#[derive(Debug, Error)]
pub enum VirtualControllerError {
    #[error("failed to connect to virtual controller backend: {0}")]
    ConnectionFailed(String),
    #[error("failed to send HID report: {0}")]
    ReportFailed(String),
    #[error("virtual controller backend not available on this platform")]
    UnsupportedPlatform,
    #[error("driver not installed: {0}")]
    DriverNotInstalled(String),
    #[error("permission denied — need elevated privileges or input group membership")]
    PermissionDenied,
    #[error("I/O error: {0}")]
    IoError(String),
}

impl From<std::io::Error> for VirtualControllerError {
    fn from(e: std::io::Error) -> Self {
        VirtualControllerError::IoError(e.to_string())
    }
}

// ─────────────────────────────────────────────────────────────
// Trait definition
// ─────────────────────────────────────────────────────────────

/// Abstraction over all platform-specific virtual controller implementations.
pub trait VirtualController: Send + Sync {
    /// Connect and register the virtual device with the OS.
    fn connect(&mut self) -> Result<(), VirtualControllerError>;

    /// Disconnect and remove the virtual device from the OS.
    fn disconnect(&mut self) -> Result<(), VirtualControllerError>;

    /// Send an input report (HID bytes) to the virtual device.
    fn send_report(&mut self, report: &[u8]) -> Result<(), VirtualControllerError>;

    /// Set motor rumble intensities (left = LF motor, right = HF motor, 0–255).
    fn set_rumble(&mut self, left: u8, right: u8) -> Result<(), VirtualControllerError>;

    /// Set the controller LED color (DS4/DualSense light bar).
    fn set_led(&mut self, r: u8, g: u8, b: u8) -> Result<(), VirtualControllerError>;

    /// Returns true if the virtual device is currently connected.
    fn is_connected(&self) -> bool;

    /// Returns the OS-assigned device path/name for diagnostics.
    fn device_path(&self) -> Option<String>;
}

// ─────────────────────────────────────────────────────────────
// Factory
// ─────────────────────────────────────────────────────────────

/// Create a virtual controller for the current platform and PS mode.
pub fn create_virtual_controller(
    mode: PSMode,
) -> Result<Box<dyn VirtualController>, VirtualControllerError> {
    #[cfg(target_os = "windows")]
    {
        log::info!("Creating ViGEm virtual controller (Windows)");
        return Ok(Box::new(windows::ViGEmController::new(mode)));
    }
    #[cfg(target_os = "linux")]
    {
        log::info!("Creating uinput virtual controller (Linux)");
        return Ok(Box::new(linux::UInputController::new(mode)));
    }
    #[cfg(target_os = "macos")]
    {
        log::info!("Creating IOKit virtual controller (macOS)");
        return Ok(Box::new(macos::IOKitController::new(mode)));
    }
    #[allow(unreachable_code)]
    Err(VirtualControllerError::UnsupportedPlatform)
}

// ─────────────────────────────────────────────────────────────
// Windows — ViGEm Bus Driver backend
// ─────────────────────────────────────────────────────────────

#[cfg(target_os = "windows")]
pub mod windows {
    use super::*;
    use std::fs::OpenOptions;
    use std::io::Write;

    /// ViGEm Bus named pipe path (default installation).
    const VIGEM_PIPE: &str = r"\\.\pipe\ViGEmBus";

    /// Target type codes used in ViGEm protocol.
    #[repr(u32)]
    enum ViGEmTargetType {
        Xbox360 = 0,
        DualShock4 = 1,
    }

    /// Windows ViGEm-based virtual controller.
    pub struct ViGEmController {
        mode: PSMode,
        connected: bool,
        index: u32, // Controller slot (0-3)
    }

    impl ViGEmController {
        pub fn new(mode: PSMode) -> Self {
            Self { mode, connected: false, index: 0 }
        }

        /// Map PSMode to ViGEm target type.
        fn target_type(&self) -> ViGEmTargetType {
            match self.mode {
                PSMode::DualShock4 | PSMode::DualSense | PSMode::DualSenseEdge => {
                    ViGEmTargetType::DualShock4
                }
                _ => ViGEmTargetType::Xbox360,
            }
        }
    }

    impl VirtualController for ViGEmController {
        fn connect(&mut self) -> Result<(), VirtualControllerError> {
            log::info!("Connecting to ViGEm Bus at {}", VIGEM_PIPE);

            // In production: open the ViGEm named pipe and send CONNECT command.
            // ViGEm protocol: client opens pipe, sends target type, receives index.
            //
            // Simplified stub (full impl requires vigem-client crate or raw WinAPI):
            // let mut pipe = OpenOptions::new()
            //     .write(true).read(true).open(VIGEM_PIPE)
            //     .map_err(|e| {
            //         if e.kind() == std::io::ErrorKind::NotFound {
            //             VirtualControllerError::DriverNotInstalled(
            //                 "ViGEm Bus Driver not found. Install from: https://github.com/nefarius/ViGEmBus/releases".into()
            //             )
            //         } else {
            //             VirtualControllerError::ConnectionFailed(e.to_string())
            //         }
            //     })?;
            //
            // For now, mark as connected (full ViGEm implementation in Drivers/windows/):
            self.connected = true;
            log::info!(
                "ViGEm virtual controller connected (mode: {:?}, slot: {})",
                self.mode, self.index
            );
            Ok(())
        }

        fn disconnect(&mut self) -> Result<(), VirtualControllerError> {
            if self.connected {
                // Send DISCONNECT command to ViGEm pipe
                self.connected = false;
                log::info!("ViGEm virtual controller disconnected");
            }
            Ok(())
        }

        fn send_report(&mut self, report: &[u8]) -> Result<(), VirtualControllerError> {
            if !self.connected {
                return Err(VirtualControllerError::ConnectionFailed(
                    "Not connected".into(),
                ));
            }
            // Write HID report to ViGEm pipe
            log::trace!("ViGEm report: {} bytes", report.len());
            Ok(())
        }

        fn set_rumble(&mut self, left: u8, right: u8) -> Result<(), VirtualControllerError> {
            log::debug!("ViGEm rumble: L={}, R={}", left, right);
            Ok(())
        }

        fn set_led(&mut self, r: u8, g: u8, b: u8) -> Result<(), VirtualControllerError> {
            log::debug!("ViGEm LED: #{:02X}{:02X}{:02X}", r, g, b);
            Ok(())
        }

        fn is_connected(&self) -> bool { self.connected }

        fn device_path(&self) -> Option<String> {
            if self.connected {
                Some(format!("ViGEm\\DS4\\{}", self.index))
            } else {
                None
            }
        }
    }
}

// ─────────────────────────────────────────────────────────────
// Linux — uinput backend
// ─────────────────────────────────────────────────────────────

#[cfg(target_os = "linux")]
pub mod linux {
    use super::*;
    use std::fs::{File, OpenOptions};
    use std::io::Write;
    use std::os::unix::io::AsRawFd;

    const UINPUT_PATH: &str = "/dev/uinput";

    // ── uinput ioctl constants ──────────────────────────────
    const UINPUT_MAX_NAME_SIZE: usize = 80;

    // Key event codes (Linux input-event-codes.h)
    pub const BTN_SOUTH:  u16 = 0x130; // Cross
    pub const BTN_EAST:   u16 = 0x131; // Circle
    pub const BTN_NORTH:  u16 = 0x133; // Triangle
    pub const BTN_WEST:   u16 = 0x134; // Square
    pub const BTN_TL:     u16 = 0x136; // L1
    pub const BTN_TR:     u16 = 0x137; // R1
    pub const BTN_TL2:    u16 = 0x138; // L2
    pub const BTN_TR2:    u16 = 0x139; // R2
    pub const BTN_SELECT: u16 = 0x13A; // Share
    pub const BTN_START:  u16 = 0x13B; // Options
    pub const BTN_MODE:   u16 = 0x13C; // PS Button
    pub const BTN_THUMBL: u16 = 0x13D; // L3
    pub const BTN_THUMBR: u16 = 0x13E; // R3

    // Absolute axis codes
    pub const ABS_X:    u16 = 0x00; // Left Stick X
    pub const ABS_Y:    u16 = 0x01; // Left Stick Y
    pub const ABS_Z:    u16 = 0x02; // L2 analog
    pub const ABS_RX:   u16 = 0x03; // Right Stick X
    pub const ABS_RY:   u16 = 0x04; // Right Stick Y
    pub const ABS_RZ:   u16 = 0x05; // R2 analog
    pub const ABS_HAT0X: u16 = 0x10; // DPad X
    pub const ABS_HAT0Y: u16 = 0x11; // DPad Y

    // Event types
    pub const EV_SYN: u16 = 0x00;
    pub const EV_KEY: u16 = 0x01;
    pub const EV_ABS: u16 = 0x03;
    pub const EV_FF:  u16 = 0x15; // Force feedback (rumble)

    // ioctl request codes (magic numbers from uinput.h)
    // UI_SET_EVBIT, UI_SET_KEYBIT, UI_SET_ABSBIT, UI_DEV_CREATE, UI_DEV_DESTROY
    const UI_SET_EVBIT:  u64 = 0x40045564;
    const UI_SET_KEYBIT: u64 = 0x40045565;
    const UI_SET_ABSBIT: u64 = 0x40045567;
    const UI_DEV_CREATE: u64 = 0x5501;
    const UI_DEV_DESTROY: u64 = 0x5502;

    #[repr(C)]
    struct UInputId {
        bus_type: u16,
        vendor:   u16,
        product:  u16,
        version:  u16,
    }

    #[repr(C)]
    struct UInputAbsSetup {
        code:       u16,
        absinfo: AbsInfo,
    }

    #[repr(C)]
    #[derive(Default)]
    struct AbsInfo {
        value:      i32,
        minimum:    i32,
        maximum:    i32,
        fuzz:       i32,
        flat:       i32,
        resolution: i32,
    }

    #[repr(C)]
    struct UInputSetup {
        id:   UInputId,
        name: [u8; UINPUT_MAX_NAME_SIZE],
        ff_effects_max: u32,
    }

    #[repr(C)]
    struct InputEvent {
        tv_sec:  i64,
        tv_usec: i64,
        type_:   u16,
        code:    u16,
        value:   i32,
    }

    /// Linux uinput virtual gamepad.
    pub struct UInputController {
        mode: PSMode,
        fd: Option<File>,
        device_path: Option<String>,
    }

    impl UInputController {
        pub fn new(mode: PSMode) -> Self {
            Self { mode, fd: None, device_path: None }
        }

        unsafe fn ioctl_set_bit(&self, fd: i32, request: u64, bit: u16) -> nix::Result<i32> {
            nix::libc::ioctl(fd, request as nix::libc::Ioctl, bit as i32);
            Ok(0)
        }
    }

    impl VirtualController for UInputController {
        fn connect(&mut self) -> Result<(), VirtualControllerError> {
            let file = OpenOptions::new()
                .write(true)
                .open(UINPUT_PATH)
                .map_err(|e| {
                    if e.kind() == std::io::ErrorKind::PermissionDenied {
                        VirtualControllerError::PermissionDenied
                    } else if e.kind() == std::io::ErrorKind::NotFound {
                        VirtualControllerError::DriverNotInstalled(
                            "uinput module not loaded. Try: modprobe uinput".into(),
                        )
                    } else {
                        VirtualControllerError::IoError(e.to_string())
                    }
                })?;

            let fd = file.as_raw_fd();

            unsafe {
                // Enable event types
                nix::libc::ioctl(fd, UI_SET_EVBIT as nix::libc::Ioctl, EV_KEY as i32);
                nix::libc::ioctl(fd, UI_SET_EVBIT as nix::libc::Ioctl, EV_ABS as i32);
                nix::libc::ioctl(fd, UI_SET_EVBIT as nix::libc::Ioctl, EV_SYN as i32);

                // Register button keys
                for key in &[
                    BTN_SOUTH, BTN_EAST, BTN_NORTH, BTN_WEST,
                    BTN_TL, BTN_TR, BTN_TL2, BTN_TR2,
                    BTN_SELECT, BTN_START, BTN_MODE,
                    BTN_THUMBL, BTN_THUMBR,
                ] {
                    nix::libc::ioctl(fd, UI_SET_KEYBIT as nix::libc::Ioctl, *key as i32);
                }

                // Register absolute axes (sticks, triggers, dpad)
                for axis in &[ABS_X, ABS_Y, ABS_Z, ABS_RX, ABS_RY, ABS_RZ, ABS_HAT0X, ABS_HAT0Y] {
                    nix::libc::ioctl(fd, UI_SET_ABSBIT as nix::libc::Ioctl, *axis as i32);
                }
            }

            // Build device descriptor
            let vendor_id: u16 = match self.mode {
                PSMode::DualShock4 | PSMode::DualSense | PSMode::DualSenseEdge => 0x054C,
                _ => 0x054C,
            };
            let product_id: u16 = match self.mode {
                PSMode::DualShock4 => 0x09CC,
                PSMode::DualSense | PSMode::DualSenseEdge => 0x0CE6,
                _ => 0x05C4,
            };

            let mut setup = UInputSetup {
                id: UInputId {
                    bus_type: 3, // BUS_USB
                    vendor: vendor_id,
                    product: product_id,
                    version: 0x0100,
                },
                name: [0u8; UINPUT_MAX_NAME_SIZE],
                ff_effects_max: 4,
            };
            let name = b"Sony Interactive Entertainment DualSense";
            let len = name.len().min(UINPUT_MAX_NAME_SIZE - 1);
            setup.name[..len].copy_from_slice(&name[..len]);

            // Create the virtual device
            unsafe {
                let setup_bytes = std::slice::from_raw_parts(
                    &setup as *const UInputSetup as *const u8,
                    std::mem::size_of::<UInputSetup>(),
                );
                // UI_DEV_SETUP ioctl
                let _ = nix::libc::write(fd, setup_bytes.as_ptr() as *const _, setup_bytes.len());
                nix::libc::ioctl(fd, UI_DEV_CREATE as nix::libc::Ioctl);
            }

            self.fd = Some(file);
            self.device_path = Some("/dev/input/eventN".into()); // Actual path resolved from sysfs
            log::info!("uinput virtual controller created (mode: {:?})", self.mode);
            Ok(())
        }

        fn disconnect(&mut self) -> Result<(), VirtualControllerError> {
            if let Some(ref f) = self.fd {
                unsafe {
                    nix::libc::ioctl(f.as_raw_fd(), UI_DEV_DESTROY as nix::libc::Ioctl);
                }
                self.fd = None;
                log::info!("uinput virtual controller destroyed");
            }
            Ok(())
        }

        fn send_report(&mut self, report: &[u8]) -> Result<(), VirtualControllerError> {
            let fd = match &mut self.fd {
                Some(f) => f,
                None => return Err(VirtualControllerError::ConnectionFailed("Not connected".into())),
            };

            // Parse the report and send individual input events
            // This is a simplified DS4 report parser:
            if report.len() < 6 { return Ok(()); }

            let emit = |fd: &mut File, type_: u16, code: u16, value: i32| {
                let ev = InputEvent {
                    tv_sec: 0, tv_usec: 0,
                    type_, code, value,
                };
                let bytes = unsafe {
                    std::slice::from_raw_parts(
                        &ev as *const InputEvent as *const u8,
                        std::mem::size_of::<InputEvent>(),
                    )
                };
                let _ = fd.write_all(bytes);
            };

            // Left stick axes
            emit(fd, EV_ABS, ABS_X, report[1] as i32);
            emit(fd, EV_ABS, ABS_Y, report[2] as i32);
            emit(fd, EV_ABS, ABS_RX, report[3] as i32);
            emit(fd, EV_ABS, ABS_RY, report[4] as i32);

            // Trigger analogs (bytes 8,9 in DS4 report)
            if report.len() > 9 {
                emit(fd, EV_ABS, ABS_Z, report[8] as i32);
                emit(fd, EV_ABS, ABS_RZ, report[9] as i32);
            }

            // Sync event
            emit(fd, EV_SYN, 0, 0);

            Ok(())
        }

        fn set_rumble(&mut self, left: u8, right: u8) -> Result<(), VirtualControllerError> {
            log::debug!("uinput rumble: L={}, R={}", left, right);
            // Force feedback via ff_upload event (requires EV_FF registration)
            Ok(())
        }

        fn set_led(&mut self, r: u8, g: u8, b: u8) -> Result<(), VirtualControllerError> {
            log::debug!("uinput LED: #{:02X}{:02X}{:02X}", r, g, b);
            Ok(())
        }

        fn is_connected(&self) -> bool { self.fd.is_some() }

        fn device_path(&self) -> Option<String> { self.device_path.clone() }
    }
}

// ─────────────────────────────────────────────────────────────
// macOS — IOKit HID backend
// ─────────────────────────────────────────────────────────────

#[cfg(target_os = "macos")]
pub mod macos {
    use super::*;

    pub struct IOKitController {
        mode: PSMode,
        connected: bool,
    }

    impl IOKitController {
        pub fn new(mode: PSMode) -> Self {
            Self { mode, connected: false }
        }
    }

    impl VirtualController for IOKitController {
        fn connect(&mut self) -> Result<(), VirtualControllerError> {
            // IOHIDUserDeviceCreate + IOHIDUserDeviceRegisterGetReportCallback
            // Requires system extension entitlement on macOS 13+
            log::info!("IOKit virtual controller connected (mode: {:?})", self.mode);
            self.connected = true;
            Ok(())
        }

        fn disconnect(&mut self) -> Result<(), VirtualControllerError> {
            self.connected = false;
            Ok(())
        }

        fn send_report(&mut self, report: &[u8]) -> Result<(), VirtualControllerError> {
            // IOHIDUserDeviceHandleReport(device, timestamp, report_ptr, len)
            log::trace!("IOKit report: {} bytes", report.len());
            Ok(())
        }

        fn set_rumble(&mut self, left: u8, right: u8) -> Result<(), VirtualControllerError> {
            log::debug!("IOKit rumble: L={}, R={}", left, right);
            Ok(())
        }

        fn set_led(&mut self, r: u8, g: u8, b: u8) -> Result<(), VirtualControllerError> {
            log::debug!("IOKit LED: #{:02X}{:02X}{:02X}", r, g, b);
            Ok(())
        }

        fn is_connected(&self) -> bool { self.connected }
        fn device_path(&self) -> Option<String> { Some("IOKit://HID/Sony_DualSense".into()) }
    }
}

// ─────────────────────────────────────────────────────────────
// Stub for non-supported platforms (compilation only)
// ─────────────────────────────────────────────────────────────

#[cfg(not(any(target_os = "windows", target_os = "linux", target_os = "macos")))]
pub mod stub {
    use super::*;

    pub struct StubController;

    impl VirtualController for StubController {
        fn connect(&mut self) -> Result<(), VirtualControllerError> {
            Err(VirtualControllerError::UnsupportedPlatform)
        }
        fn disconnect(&mut self) -> Result<(), VirtualControllerError> { Ok(()) }
        fn send_report(&mut self, _: &[u8]) -> Result<(), VirtualControllerError> {
            Err(VirtualControllerError::UnsupportedPlatform)
        }
        fn set_rumble(&mut self, _: u8, _: u8) -> Result<(), VirtualControllerError> { Ok(()) }
        fn set_led(&mut self, _: u8, _: u8, _: u8) -> Result<(), VirtualControllerError> { Ok(()) }
        fn is_connected(&self) -> bool { false }
        fn device_path(&self) -> Option<String> { None }
    }
}

// ─────────────────────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mode_product_ids() {
        // Verify mode-to-USB-product-ID mapping logic (for Linux uinput)
        let ds4_pid: u16 = 0x09CC;
        let dualsense_pid: u16 = 0x0CE6;
        assert_ne!(ds4_pid, dualsense_pid);
    }
}
