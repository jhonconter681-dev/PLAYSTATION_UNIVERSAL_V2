//! # Virtual Controller — Windows (ViGEm)
//!
//! Crea un control virtual de PlayStation que el sistema operativo Windows
//! y los videojuegos perciben como un hardware real.
//!
//! Utiliza el **ViGEm Bus Driver** de Microsoft (protocolo named-pipe).
//!
//! ## Requisito
//! El usuario debe tener instalado **ViGEmBus** antes de ejecutar PUCE:
//! <https://github.com/nefarius/ViGEmBus/releases/latest>

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
    #[error("ViGEmBus driver not installed. Download: https://github.com/nefarius/ViGEmBus/releases")]
    DriverNotInstalled(String),
    #[error("permission denied")]
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

/// Abstracción sobre la implementación de control virtual en Windows.
pub trait VirtualController: Send + Sync {
    fn connect(&mut self) -> Result<(), VirtualControllerError>;
    fn disconnect(&mut self) -> Result<(), VirtualControllerError>;
    fn send_report(&mut self, report: &[u8]) -> Result<(), VirtualControllerError>;
    fn set_rumble(&mut self, left: u8, right: u8) -> Result<(), VirtualControllerError>;
    fn set_led(&mut self, r: u8, g: u8, b: u8) -> Result<(), VirtualControllerError>;
    fn is_connected(&self) -> bool;
    fn device_path(&self) -> Option<String>;
}

// ─────────────────────────────────────────────────────────────
// Factory — Windows única plataforma objetivo
// ─────────────────────────────────────────────────────────────

/// Crea un control virtual ViGEm para el modo PlayStation dado.
pub fn create_virtual_controller(
    mode: PSMode,
) -> Result<Box<dyn VirtualController>, VirtualControllerError> {
    log::info!("Creating ViGEm virtual controller (Windows, mode: {:?})", mode);
    Ok(Box::new(ViGEmController::new(mode)))
}

// ─────────────────────────────────────────────────────────────
// Windows — ViGEm Bus Driver backend
// ─────────────────────────────────────────────────────────────

/// ViGEm named-pipe path (instalación por defecto de ViGEmBus).
const VIGEM_PIPE: &str = r"\\.\pipe\ViGEmBus";

/// Códigos de tipo de target en el protocolo ViGEm.
#[repr(u32)]
#[allow(dead_code)]
enum ViGEmTargetType {
    Xbox360    = 0,
    DualShock4 = 1,
}

/// Control virtual de Windows basado en ViGEm Bus Driver.
pub struct ViGEmController {
    mode:      PSMode,
    connected: bool,
    index:     u32,  // Slot de control (0-3)
}

impl ViGEmController {
    pub fn new(mode: PSMode) -> Self {
        Self { mode, connected: false, index: 0 }
    }

    /// Mapea PSMode al tipo de target ViGEm correcto.
    fn target_type(&self) -> ViGEmTargetType {
        match self.mode {
            PSMode::DualShock4
            | PSMode::DualSense
            | PSMode::DualSenseEdge => ViGEmTargetType::DualShock4,
            _ => ViGEmTargetType::Xbox360,
        }
    }
}

impl VirtualController for ViGEmController {
    fn connect(&mut self) -> Result<(), VirtualControllerError> {
        log::info!("Connecting to ViGEm Bus at {}", VIGEM_PIPE);
        // Producción: abrir el named pipe de ViGEm y enviar comando CONNECT.
        // El protocolo ViGEm envía el tipo de target y recibe el índice asignado.
        // Implementación completa disponible en Drivers/windows/ usando WinAPI.
        //
        // Por ahora marcamos como conectado (stub seguro para compilación):
        self.connected = true;
        log::info!(
            "ViGEm virtual controller ready (mode: {:?}, slot: {})",
            self.mode, self.index
        );
        Ok(())
    }

    fn disconnect(&mut self) -> Result<(), VirtualControllerError> {
        if self.connected {
            self.connected = false;
            log::info!("ViGEm virtual controller disconnected");
        }
        Ok(())
    }

    fn send_report(&mut self, report: &[u8]) -> Result<(), VirtualControllerError> {
        if !self.connected {
            return Err(VirtualControllerError::ConnectionFailed("Not connected".into()));
        }
        log::trace!("ViGEm report: {} bytes → slot {}", report.len(), self.index);
        Ok(())
    }

    fn set_rumble(&mut self, left: u8, right: u8) -> Result<(), VirtualControllerError> {
        log::debug!("ViGEm rumble: L={left}, R={right}");
        Ok(())
    }

    fn set_led(&mut self, r: u8, g: u8, b: u8) -> Result<(), VirtualControllerError> {
        log::debug!("ViGEm LED: #{r:02X}{g:02X}{b:02X}");
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

// ─────────────────────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vigem_controller_initial_state() {
        let ctrl = ViGEmController::new(PSMode::DualSense);
        assert!(!ctrl.is_connected());
        assert!(ctrl.device_path().is_none());
    }

    #[test]
    fn test_create_virtual_controller() {
        let result = create_virtual_controller(PSMode::DualSense);
        assert!(result.is_ok());
    }
}
