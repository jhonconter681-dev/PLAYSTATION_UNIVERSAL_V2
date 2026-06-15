//! # Database Layer
//!
//! Provides all SQLite persistence for PUCE:
//! - HID device registry (500+ known devices, updatable via OTA)
//! - User mapping profiles
//! - Application settings (key-value store)
//! - Plugin registry
//! - Per-device calibration data
//! - Schema migration system

use chrono::{DateTime, Utc};
use rusqlite::{params, Connection, OptionalExtension, Result as SqlResult};
use serde::{Deserialize, Serialize};
use std::path::Path;
use thiserror::Error;

// ─────────────────────────────────────────────────────────────
// Error types
// ─────────────────────────────────────────────────────────────

#[derive(Debug, Error)]
pub enum DatabaseError {
    #[error("SQLite error: {0}")]
    Sqlite(#[from] rusqlite::Error),
    #[error("JSON serialization error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("record not found: {0}")]
    NotFound(String),
    #[error("migration failed at version {version}: {reason}")]
    MigrationFailed { version: u32, reason: String },
}

// ─────────────────────────────────────────────────────────────
// Domain types
// ─────────────────────────────────────────────────────────────

/// Device type classification.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum DeviceType {
    PlayStation,
    Xbox,
    Nintendo,
    Logitech,
    EightBitDo,
    Razer,
    HyperX,
    Hori,
    Thrustmaster,
    Gamesir,
    Flydigi,
    Redragon,
    Generic,
    Keyboard,
    Mouse,
    FlightStick,
    RacingWheel,
    FightStick,
    Touchscreen,
}

impl DeviceType {
    pub fn as_str(&self) -> &'static str {
        match self {
            DeviceType::PlayStation => "PlayStation",
            DeviceType::Xbox => "Xbox",
            DeviceType::Nintendo => "Nintendo",
            DeviceType::Logitech => "Logitech",
            DeviceType::EightBitDo => "8BitDo",
            DeviceType::Razer => "Razer",
            DeviceType::HyperX => "HyperX",
            DeviceType::Hori => "Hori",
            DeviceType::Thrustmaster => "Thrustmaster",
            DeviceType::Gamesir => "Gamesir",
            DeviceType::Flydigi => "Flydigi",
            DeviceType::Redragon => "Redragon",
            DeviceType::Generic => "Generic",
            DeviceType::Keyboard => "Keyboard",
            DeviceType::Mouse => "Mouse",
            DeviceType::FlightStick => "FlightStick",
            DeviceType::RacingWheel => "RacingWheel",
            DeviceType::FightStick => "FightStick",
            DeviceType::Touchscreen => "Touchscreen",
        }
    }

    pub fn from_str(s: &str) -> DeviceType {
        match s {
            "PlayStation" => DeviceType::PlayStation,
            "Xbox" => DeviceType::Xbox,
            "Nintendo" => DeviceType::Nintendo,
            "Logitech" => DeviceType::Logitech,
            "8BitDo" => DeviceType::EightBitDo,
            "Razer" => DeviceType::Razer,
            "HyperX" => DeviceType::HyperX,
            "Hori" => DeviceType::Hori,
            "Thrustmaster" => DeviceType::Thrustmaster,
            "Gamesir" => DeviceType::Gamesir,
            "Flydigi" => DeviceType::Flydigi,
            "Redragon" => DeviceType::Redragon,
            "Keyboard" => DeviceType::Keyboard,
            "Mouse" => DeviceType::Mouse,
            "FlightStick" => DeviceType::FlightStick,
            "RacingWheel" => DeviceType::RacingWheel,
            "FightStick" => DeviceType::FightStick,
            "Touchscreen" => DeviceType::Touchscreen,
            _ => DeviceType::Generic,
        }
    }
}

/// Full HID device record stored in the database.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceRecord {
    pub id: Option<i64>,
    pub vendor_id: u16,
    pub product_id: u16,
    pub name: String,
    pub manufacturer: String,
    pub device_type: DeviceType,
    pub button_count: u8,
    pub axis_count: u8,
    /// JSON array of sensor strings: ["gyroscope", "accelerometer", "touchpad"]
    pub sensors: Vec<String>,
    /// JSON array of capability strings: ["haptics", "adaptive_triggers", "vibration"]
    pub capabilities: Vec<String>,
    pub firmware: Option<String>,
    pub bt_version: Option<String>,
    pub usb_version: Option<String>,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
}

impl DeviceRecord {
    pub fn new(
        vid: u16, pid: u16,
        name: &str, manufacturer: &str, device_type: DeviceType,
        buttons: u8, axes: u8,
    ) -> Self {
        Self {
            id: None,
            vendor_id: vid,
            product_id: pid,
            name: name.into(),
            manufacturer: manufacturer.into(),
            device_type,
            button_count: buttons,
            axis_count: axes,
            sensors: vec![],
            capabilities: vec![],
            firmware: None,
            bt_version: None,
            usb_version: None,
            created_at: None,
            updated_at: None,
        }
    }
}

/// User mapping profile stored in the database.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProfileRecord {
    pub id: String, // UUID
    pub name: String,
    pub device_vid: Option<u16>,
    pub device_pid: Option<u16>,
    pub ps_mode: String, // PSMode string
    pub button_mappings: serde_json::Value,
    pub axis_mappings: serde_json::Value,
    pub virtual_buttons: serde_json::Value,
    pub is_default: bool,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
}

impl ProfileRecord {
    pub fn new(name: &str, ps_mode: &str) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            name: name.into(),
            device_vid: None,
            device_pid: None,
            ps_mode: ps_mode.into(),
            button_mappings: serde_json::json!([]),
            axis_mappings: serde_json::json!([]),
            virtual_buttons: serde_json::json!([]),
            is_default: false,
            created_at: None,
            updated_at: None,
        }
    }
}

/// Per-device analog stick calibration data.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CalibrationRecord {
    pub device_id: String,
    pub left_center_x: f32,
    pub left_center_y: f32,
    pub left_dead_zone: f32,
    pub left_max_radius: f32,
    pub right_center_x: f32,
    pub right_center_y: f32,
    pub right_dead_zone: f32,
    pub right_max_radius: f32,
    pub l2_min: u8,
    pub l2_max: u8,
    pub r2_min: u8,
    pub r2_max: u8,
    pub calibrated_at: Option<DateTime<Utc>>,
}

// ─────────────────────────────────────────────────────────────
// Database
// ─────────────────────────────────────────────────────────────

pub struct Database {
    conn: Connection,
}

impl Database {
    /// Open (or create) a PUCE database at the given path.
    /// Runs all pending migrations automatically.
    pub fn open(path: &str) -> Result<Self, DatabaseError> {
        let conn = Connection::open(path)?;

        // Enable WAL mode for better concurrent access
        conn.execute_batch("PRAGMA journal_mode=WAL; PRAGMA foreign_keys=ON;")?;

        let mut db = Self { conn };
        db.run_migrations()?;

        log::info!("Database opened: {}", path);
        Ok(db)
    }

    /// Open an in-memory database (for testing).
    pub fn open_in_memory() -> Result<Self, DatabaseError> {
        let conn = Connection::open_in_memory()?;
        conn.execute_batch("PRAGMA foreign_keys=ON;")?;
        let mut db = Self { conn };
        db.run_migrations()?;
        Ok(db)
    }

    // ─────────────────────────────────────────────────────────
    // Migrations
    // ─────────────────────────────────────────────────────────

    fn run_migrations(&mut self) -> Result<(), DatabaseError> {
        // Create migrations tracking table first
        self.conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS schema_migrations (
                version    INTEGER PRIMARY KEY,
                applied_at DATETIME DEFAULT CURRENT_TIMESTAMP
            );"
        )?;

        let current_version: u32 = self.conn
            .query_row(
                "SELECT COALESCE(MAX(version), 0) FROM schema_migrations",
                [],
                |r| r.get(0),
            )
            .unwrap_or(0);

        let migrations: &[(u32, &str)] = &[
            (1, include_str!("../../../Database/migrations/001_initial.sql")),
            (2, include_str!("../../../Database/migrations/002_profiles_v2.sql")),
        ];

        for (version, sql) in migrations {
            if *version > current_version {
                log::info!("Running database migration v{}", version);
                self.conn.execute_batch(sql).map_err(|e| {
                    DatabaseError::MigrationFailed {
                        version: *version,
                        reason: e.to_string(),
                    }
                })?;
                self.conn.execute(
                    "INSERT INTO schema_migrations (version) VALUES (?1)",
                    params![version],
                )?;
            }
        }

        Ok(())
    }

    // ─────────────────────────────────────────────────────────
    // Device Registry
    // ─────────────────────────────────────────────────────────

    /// Insert or update a device record (UPSERT by VID+PID).
    pub fn upsert_device(&self, device: &DeviceRecord) -> Result<i64, DatabaseError> {
        let sensors_json = serde_json::to_string(&device.sensors)?;
        let caps_json = serde_json::to_string(&device.capabilities)?;

        self.conn.execute(
            "INSERT INTO devices (
                vendor_id, product_id, name, manufacturer, device_type,
                button_count, axis_count, sensors, capabilities,
                firmware, bt_version, usb_version, updated_at
             ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, CURRENT_TIMESTAMP)
             ON CONFLICT(vendor_id, product_id) DO UPDATE SET
                name        = excluded.name,
                manufacturer = excluded.manufacturer,
                device_type = excluded.device_type,
                button_count = excluded.button_count,
                axis_count  = excluded.axis_count,
                sensors     = excluded.sensors,
                capabilities = excluded.capabilities,
                firmware    = excluded.firmware,
                bt_version  = excluded.bt_version,
                usb_version = excluded.usb_version,
                updated_at  = CURRENT_TIMESTAMP",
            params![
                device.vendor_id, device.product_id, device.name, device.manufacturer,
                device.device_type.as_str(), device.button_count, device.axis_count,
                sensors_json, caps_json, device.firmware, device.bt_version, device.usb_version,
            ],
        )?;

        Ok(self.conn.last_insert_rowid())
    }

    /// Find a device by Vendor ID + Product ID.
    pub fn find_device(&self, vid: u16, pid: u16) -> Result<Option<DeviceRecord>, DatabaseError> {
        let result = self.conn.query_row(
            "SELECT id, vendor_id, product_id, name, manufacturer, device_type,
                    button_count, axis_count, sensors, capabilities,
                    firmware, bt_version, usb_version, created_at, updated_at
             FROM devices WHERE vendor_id = ?1 AND product_id = ?2",
            params![vid, pid],
            |row| {
                Ok((
                    row.get::<_, i64>(0)?,
                    row.get::<_, u16>(1)?,
                    row.get::<_, u16>(2)?,
                    row.get::<_, String>(3)?,
                    row.get::<_, String>(4)?,
                    row.get::<_, String>(5)?,
                    row.get::<_, u8>(6)?,
                    row.get::<_, u8>(7)?,
                    row.get::<_, String>(8)?,  // sensors JSON
                    row.get::<_, String>(9)?,  // caps JSON
                    row.get::<_, Option<String>>(10)?,
                    row.get::<_, Option<String>>(11)?,
                    row.get::<_, Option<String>>(12)?,
                ))
            },
        ).optional()?;

        match result {
            None => Ok(None),
            Some((id, vid, pid, name, mfr, dtype, btns, axes, sensors_str, caps_str,
                  fw, bt_ver, usb_ver)) => {
                let sensors: Vec<String> = serde_json::from_str(&sensors_str).unwrap_or_default();
                let capabilities: Vec<String> = serde_json::from_str(&caps_str).unwrap_or_default();
                Ok(Some(DeviceRecord {
                    id: Some(id),
                    vendor_id: vid,
                    product_id: pid,
                    name,
                    manufacturer: mfr,
                    device_type: DeviceType::from_str(&dtype),
                    button_count: btns,
                    axis_count: axes,
                    sensors,
                    capabilities,
                    firmware: fw,
                    bt_version: bt_ver,
                    usb_version: usb_ver,
                    created_at: None,
                    updated_at: None,
                }))
            }
        }
    }

    /// Get all device records.
    pub fn get_all_devices(&self) -> Result<Vec<DeviceRecord>, DatabaseError> {
        let mut stmt = self.conn.prepare(
            "SELECT id, vendor_id, product_id, name, manufacturer, device_type,
                    button_count, axis_count, sensors, capabilities,
                    firmware, bt_version, usb_version
             FROM devices ORDER BY manufacturer, name"
        )?;

        let records = stmt.query_map([], |row| {
            Ok((
                row.get::<_, i64>(0)?,
                row.get::<_, u16>(1)?,
                row.get::<_, u16>(2)?,
                row.get::<_, String>(3)?,
                row.get::<_, String>(4)?,
                row.get::<_, String>(5)?,
                row.get::<_, u8>(6)?,
                row.get::<_, u8>(7)?,
                row.get::<_, String>(8)?,
                row.get::<_, String>(9)?,
                row.get::<_, Option<String>>(10)?,
                row.get::<_, Option<String>>(11)?,
                row.get::<_, Option<String>>(12)?,
            ))
        })?;

        let mut devices = Vec::new();
        for row in records {
            let (id, vid, pid, name, mfr, dtype, btns, axes, sens_str, caps_str, fw, bt_ver, usb_ver) = row?;
            let sensors: Vec<String> = serde_json::from_str(&sens_str).unwrap_or_default();
            let capabilities: Vec<String> = serde_json::from_str(&caps_str).unwrap_or_default();
            devices.push(DeviceRecord {
                id: Some(id),
                vendor_id: vid,
                product_id: pid,
                name,
                manufacturer: mfr,
                device_type: DeviceType::from_str(&dtype),
                button_count: btns,
                axis_count: axes,
                sensors,
                capabilities,
                firmware: fw,
                bt_version: bt_ver,
                usb_version: usb_ver,
                created_at: None,
                updated_at: None,
            });
        }

        Ok(devices)
    }

    /// Count devices in the registry.
    pub fn count_devices(&self) -> Result<i64, DatabaseError> {
        Ok(self.conn.query_row(
            "SELECT COUNT(*) FROM devices",
            [],
            |r| r.get(0),
        )?)
    }

    // ─────────────────────────────────────────────────────────
    // Profiles
    // ─────────────────────────────────────────────────────────

    /// Save a mapping profile (insert or replace by ID).
    pub fn save_profile(&self, profile: &ProfileRecord) -> Result<(), DatabaseError> {
        self.conn.execute(
            "INSERT INTO mapping_profiles (
                id, name, device_vid, device_pid, ps_mode,
                button_mappings, axis_mappings, virtual_buttons, is_default, updated_at
             ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, CURRENT_TIMESTAMP)
             ON CONFLICT(id) DO UPDATE SET
                name            = excluded.name,
                device_vid      = excluded.device_vid,
                device_pid      = excluded.device_pid,
                ps_mode         = excluded.ps_mode,
                button_mappings = excluded.button_mappings,
                axis_mappings   = excluded.axis_mappings,
                virtual_buttons = excluded.virtual_buttons,
                is_default      = excluded.is_default,
                updated_at      = CURRENT_TIMESTAMP",
            params![
                profile.id, profile.name, profile.device_vid, profile.device_pid,
                profile.ps_mode,
                profile.button_mappings.to_string(),
                profile.axis_mappings.to_string(),
                profile.virtual_buttons.to_string(),
                profile.is_default as i32,
            ],
        )?;
        log::debug!("Profile saved: '{}' [{}]", profile.name, profile.id);
        Ok(())
    }

    /// Load a profile by ID.
    pub fn load_profile(&self, id: &str) -> Result<ProfileRecord, DatabaseError> {
        let result = self.conn.query_row(
            "SELECT id, name, device_vid, device_pid, ps_mode,
                    button_mappings, axis_mappings, virtual_buttons, is_default
             FROM mapping_profiles WHERE id = ?1",
            params![id],
            |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, Option<u16>>(2)?,
                    row.get::<_, Option<u16>>(3)?,
                    row.get::<_, String>(4)?,
                    row.get::<_, String>(5)?,
                    row.get::<_, String>(6)?,
                    row.get::<_, String>(7)?,
                    row.get::<_, bool>(8)?,
                ))
            },
        ).optional()?;

        match result {
            None => Err(DatabaseError::NotFound(format!("Profile not found: {}", id))),
            Some((id, name, vid, pid, mode, btns, axes, virt, is_def)) => {
                Ok(ProfileRecord {
                    id,
                    name,
                    device_vid: vid,
                    device_pid: pid,
                    ps_mode: mode,
                    button_mappings: serde_json::from_str(&btns).unwrap_or(serde_json::json!([])),
                    axis_mappings: serde_json::from_str(&axes).unwrap_or(serde_json::json!([])),
                    virtual_buttons: serde_json::from_str(&virt).unwrap_or(serde_json::json!([])),
                    is_default: is_def,
                    created_at: None,
                    updated_at: None,
                })
            }
        }
    }

    /// List all profiles.
    pub fn list_profiles(&self) -> Result<Vec<ProfileRecord>, DatabaseError> {
        let mut stmt = self.conn.prepare(
            "SELECT id, name, device_vid, device_pid, ps_mode,
                    button_mappings, axis_mappings, virtual_buttons, is_default
             FROM mapping_profiles ORDER BY updated_at DESC"
        )?;

        let rows = stmt.query_map([], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, Option<u16>>(2)?,
                row.get::<_, Option<u16>>(3)?,
                row.get::<_, String>(4)?,
                row.get::<_, String>(5)?,
                row.get::<_, String>(6)?,
                row.get::<_, String>(7)?,
                row.get::<_, bool>(8)?,
            ))
        })?;

        let mut profiles = Vec::new();
        for row in rows {
            let (id, name, vid, pid, mode, btns, axes, virt, is_def) = row?;
            profiles.push(ProfileRecord {
                id,
                name,
                device_vid: vid,
                device_pid: pid,
                ps_mode: mode,
                button_mappings: serde_json::from_str(&btns).unwrap_or(serde_json::json!([])),
                axis_mappings: serde_json::from_str(&axes).unwrap_or(serde_json::json!([])),
                virtual_buttons: serde_json::from_str(&virt).unwrap_or(serde_json::json!([])),
                is_default: is_def,
                created_at: None,
                updated_at: None,
            });
        }

        Ok(profiles)
    }

    /// Delete a profile by ID.
    pub fn delete_profile(&self, id: &str) -> Result<bool, DatabaseError> {
        let rows = self.conn.execute(
            "DELETE FROM mapping_profiles WHERE id = ?1",
            params![id],
        )?;
        Ok(rows > 0)
    }

    // ─────────────────────────────────────────────────────────
    // Settings
    // ─────────────────────────────────────────────────────────

    pub fn set_setting(&self, key: &str, value: &str) -> Result<(), DatabaseError> {
        self.conn.execute(
            "INSERT INTO user_settings (key, value, updated_at)
             VALUES (?1, ?2, CURRENT_TIMESTAMP)
             ON CONFLICT(key) DO UPDATE SET
                value = excluded.value,
                updated_at = CURRENT_TIMESTAMP",
            params![key, value],
        )?;
        Ok(())
    }

    pub fn get_setting(&self, key: &str) -> Result<Option<String>, DatabaseError> {
        Ok(self.conn.query_row(
            "SELECT value FROM user_settings WHERE key = ?1",
            params![key],
            |r| r.get(0),
        ).optional()?)
    }

    pub fn get_setting_or_default<'a>(&self, key: &str, default: &'a str) -> String {
        self.get_setting(key).ok().flatten().unwrap_or_else(|| default.to_string())
    }

    // ─────────────────────────────────────────────────────────
    // Calibration
    // ─────────────────────────────────────────────────────────

    pub fn save_calibration(&self, cal: &CalibrationRecord) -> Result<(), DatabaseError> {
        self.conn.execute(
            "INSERT INTO calibration_data (
                device_id, left_center_x, left_center_y, left_dead_zone, left_max_radius,
                right_center_x, right_center_y, right_dead_zone, right_max_radius,
                l2_min, l2_max, r2_min, r2_max, calibrated_at
             ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, CURRENT_TIMESTAMP)
             ON CONFLICT(device_id) DO UPDATE SET
                left_center_x = excluded.left_center_x,
                left_center_y = excluded.left_center_y,
                left_dead_zone = excluded.left_dead_zone,
                left_max_radius = excluded.left_max_radius,
                right_center_x = excluded.right_center_x,
                right_center_y = excluded.right_center_y,
                right_dead_zone = excluded.right_dead_zone,
                right_max_radius = excluded.right_max_radius,
                l2_min = excluded.l2_min,
                l2_max = excluded.l2_max,
                r2_min = excluded.r2_min,
                r2_max = excluded.r2_max,
                calibrated_at = CURRENT_TIMESTAMP",
            params![
                cal.device_id,
                cal.left_center_x, cal.left_center_y, cal.left_dead_zone, cal.left_max_radius,
                cal.right_center_x, cal.right_center_y, cal.right_dead_zone, cal.right_max_radius,
                cal.l2_min, cal.l2_max, cal.r2_min, cal.r2_max,
            ],
        )?;
        Ok(())
    }

    pub fn load_calibration(&self, device_id: &str) -> Result<Option<CalibrationRecord>, DatabaseError> {
        Ok(self.conn.query_row(
            "SELECT device_id, left_center_x, left_center_y, left_dead_zone, left_max_radius,
                    right_center_x, right_center_y, right_dead_zone, right_max_radius,
                    l2_min, l2_max, r2_min, r2_max
             FROM calibration_data WHERE device_id = ?1",
            params![device_id],
            |row| {
                Ok(CalibrationRecord {
                    device_id: row.get(0)?,
                    left_center_x: row.get(1)?,
                    left_center_y: row.get(2)?,
                    left_dead_zone: row.get(3)?,
                    left_max_radius: row.get(4)?,
                    right_center_x: row.get(5)?,
                    right_center_y: row.get(6)?,
                    right_dead_zone: row.get(7)?,
                    right_max_radius: row.get(8)?,
                    l2_min: row.get(9)?,
                    l2_max: row.get(10)?,
                    r2_min: row.get(11)?,
                    r2_max: row.get(12)?,
                    calibrated_at: None,
                })
            },
        ).optional()?)
    }
}

// ─────────────────────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn make_db() -> Database {
        Database::open_in_memory().expect("in-memory DB")
    }

    #[test]
    fn test_open_in_memory() {
        let _db = make_db();
    }

    #[test]
    fn test_upsert_and_find_device() {
        let db = make_db();
        let device = DeviceRecord::new(
            0x054C, 0x0CE6,
            "DualSense Wireless Controller",
            "Sony Interactive Entertainment",
            DeviceType::PlayStation,
            17, 6,
        );
        db.upsert_device(&device).expect("upsert");
        let found = db.find_device(0x054C, 0x0CE6).expect("find");
        assert!(found.is_some());
        let found = found.unwrap();
        assert_eq!(found.name, "DualSense Wireless Controller");
        assert_eq!(found.vendor_id, 0x054C);
        assert_eq!(found.product_id, 0x0CE6);
    }

    #[test]
    fn test_upsert_updates_existing() {
        let db = make_db();
        let mut device = DeviceRecord::new(0x045E, 0x028E, "Xbox 360", "Microsoft", DeviceType::Xbox, 14, 6);
        db.upsert_device(&device).expect("insert");

        device.name = "Xbox 360 Controller (Updated)".into();
        db.upsert_device(&device).expect("update");

        let found = db.find_device(0x045E, 0x028E).unwrap().unwrap();
        assert_eq!(found.name, "Xbox 360 Controller (Updated)");
    }

    #[test]
    fn test_save_and_load_profile() {
        let db = make_db();
        let profile = ProfileRecord::new("My DS4 Profile", "DualShock4");
        db.save_profile(&profile).expect("save");
        let loaded = db.load_profile(&profile.id).expect("load");
        assert_eq!(loaded.name, "My DS4 Profile");
        assert_eq!(loaded.ps_mode, "DualShock4");
    }

    #[test]
    fn test_delete_profile() {
        let db = make_db();
        let profile = ProfileRecord::new("Temp Profile", "PS1");
        db.save_profile(&profile).expect("save");
        let deleted = db.delete_profile(&profile.id).expect("delete");
        assert!(deleted);
        let result = db.load_profile(&profile.id);
        assert!(matches!(result, Err(DatabaseError::NotFound(_))));
    }

    #[test]
    fn test_list_profiles() {
        let db = make_db();
        db.save_profile(&ProfileRecord::new("Profile A", "DualShock4")).unwrap();
        db.save_profile(&ProfileRecord::new("Profile B", "DualSense")).unwrap();
        let profiles = db.list_profiles().unwrap();
        assert_eq!(profiles.len(), 2);
    }

    #[test]
    fn test_settings() {
        let db = make_db();
        db.set_setting("theme", "dark").unwrap();
        db.set_setting("polling_rate", "1000").unwrap();

        assert_eq!(db.get_setting("theme").unwrap(), Some("dark".into()));
        assert_eq!(db.get_setting("polling_rate").unwrap(), Some("1000".into()));
        assert_eq!(db.get_setting("nonexistent").unwrap(), None);
        assert_eq!(db.get_setting_or_default("nonexistent", "default"), "default");
    }

    #[test]
    fn test_device_count() {
        let db = make_db();
        assert_eq!(db.count_devices().unwrap(), 0);
        db.upsert_device(&DeviceRecord::new(1, 1, "Dev1", "Mfr", DeviceType::Generic, 8, 2)).unwrap();
        db.upsert_device(&DeviceRecord::new(2, 2, "Dev2", "Mfr", DeviceType::Generic, 8, 2)).unwrap();
        assert_eq!(db.count_devices().unwrap(), 2);
    }
}
