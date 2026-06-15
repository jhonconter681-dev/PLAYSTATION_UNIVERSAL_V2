-- =============================================================================
-- PUCE (PlayStation Universal Controller Emulator)
-- Database Schema — SQLite
-- Version: 2.0
-- =============================================================================

PRAGMA journal_mode = WAL;
PRAGMA foreign_keys = ON;
PRAGMA encoding = 'UTF-8';

-- ---------------------------------------------------------------------------
-- Schema Migrations tracker (always first)
-- ---------------------------------------------------------------------------
CREATE TABLE IF NOT EXISTS schema_migrations (
    version      TEXT    NOT NULL PRIMARY KEY,  -- e.g. '001', '002'
    applied_at   TEXT    NOT NULL DEFAULT (datetime('now'))
);

-- ---------------------------------------------------------------------------
-- Devices
-- Represents a physical HID device that PUCE has seen or knows about.
-- ---------------------------------------------------------------------------
CREATE TABLE IF NOT EXISTS devices (
    id              INTEGER PRIMARY KEY AUTOINCREMENT,
    vid             INTEGER NOT NULL,                  -- USB Vendor ID  (hex stored as int)
    pid             INTEGER NOT NULL,                  -- USB Product ID (hex stored as int)
    name            TEXT    NOT NULL,                  -- Human-readable product name
    manufacturer    TEXT    NOT NULL DEFAULT '',       -- Manufacturer string
    device_type     TEXT    NOT NULL DEFAULT 'gamepad',-- 'gamepad','keyboard','mouse','flight_stick','wheel','touchpad','other'
    button_count    INTEGER NOT NULL DEFAULT 0,
    axis_count      INTEGER NOT NULL DEFAULT 0,
    -- JSON arrays / objects for rich capability description
    sensors         TEXT    NOT NULL DEFAULT '[]',     -- JSON: [{"type":"accelerometer","axes":3}, ...]
    capabilities    TEXT    NOT NULL DEFAULT '{}',     -- JSON: {"rumble":true,"leds":true,"touchpad":true,...}
    firmware        TEXT             DEFAULT NULL,     -- Firmware version string if known
    bt_version      TEXT             DEFAULT NULL,     -- Bluetooth profile version e.g. "4.0"
    usb_version     TEXT             DEFAULT NULL,     -- USB spec version e.g. "2.0"
    created_at      TEXT    NOT NULL DEFAULT (datetime('now')),
    updated_at      TEXT    NOT NULL DEFAULT (datetime('now')),

    UNIQUE (vid, pid)
);

CREATE INDEX IF NOT EXISTS idx_devices_vid_pid ON devices (vid, pid);
CREATE INDEX IF NOT EXISTS idx_devices_device_type ON devices (device_type);

-- ---------------------------------------------------------------------------
-- Mapping Profiles
-- A named set of button/axis mappings for a specific device -> PS mode.
-- ---------------------------------------------------------------------------
CREATE TABLE IF NOT EXISTS mapping_profiles (
    id              TEXT    PRIMARY KEY,               -- UUID v4
    name            TEXT    NOT NULL,
    device_vid      INTEGER          DEFAULT NULL,     -- NULL = generic/any device
    device_pid      INTEGER          DEFAULT NULL,
    ps_mode         TEXT    NOT NULL DEFAULT 'ds4',    -- 'ps1','ps2','ps3','ds4','ds5'
    button_mappings TEXT    NOT NULL DEFAULT '{}',     -- JSON object: {"CROSS":"BTN_SOUTH",...}
    axis_mappings   TEXT    NOT NULL DEFAULT '{}',     -- JSON object: {"LEFT_X":"ABS_X",...}
    virtual_buttons TEXT    NOT NULL DEFAULT '[]',     -- JSON array of virtual button definitions
    is_default      INTEGER NOT NULL DEFAULT 0,        -- 1 = default profile for this device
    -- Export/import metadata (added in migration 002)
    export_version  TEXT             DEFAULT NULL,
    export_hash     TEXT             DEFAULT NULL,
    source_url      TEXT             DEFAULT NULL,
    created_at      TEXT    NOT NULL DEFAULT (datetime('now')),
    updated_at      TEXT    NOT NULL DEFAULT (datetime('now')),

    FOREIGN KEY (device_vid, device_pid) REFERENCES devices (vid, pid)
        ON UPDATE CASCADE ON DELETE SET NULL
);

CREATE INDEX IF NOT EXISTS idx_mapping_profiles_device ON mapping_profiles (device_vid, device_pid);
CREATE INDEX IF NOT EXISTS idx_mapping_profiles_ps_mode ON mapping_profiles (ps_mode);
CREATE INDEX IF NOT EXISTS idx_mapping_profiles_is_default ON mapping_profiles (is_default);

-- ---------------------------------------------------------------------------
-- User Settings
-- Simple key-value store for application preferences.
-- ---------------------------------------------------------------------------
CREATE TABLE IF NOT EXISTS user_settings (
    key         TEXT PRIMARY KEY,
    value       TEXT NOT NULL DEFAULT '',
    updated_at  TEXT NOT NULL DEFAULT (datetime('now'))
);

-- ---------------------------------------------------------------------------
-- Plugin Registry
-- Tracks installed plugins (external device handlers, extra mappings, etc.)
-- ---------------------------------------------------------------------------
CREATE TABLE IF NOT EXISTS plugin_registry (
    id              INTEGER PRIMARY KEY AUTOINCREMENT,
    name            TEXT    NOT NULL UNIQUE,
    version         TEXT    NOT NULL,                  -- semver string
    author          TEXT    NOT NULL DEFAULT '',
    path            TEXT    NOT NULL,                  -- Absolute path to the shared library
    signature_hash  TEXT    NOT NULL,                  -- SHA-256 of the plugin binary
    enabled         INTEGER NOT NULL DEFAULT 1,        -- 1 = active, 0 = disabled
    installed_at    TEXT    NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX IF NOT EXISTS idx_plugin_registry_enabled ON plugin_registry (enabled);
CREATE INDEX IF NOT EXISTS idx_plugin_registry_name    ON plugin_registry (name);

-- ---------------------------------------------------------------------------
-- Calibration Data
-- Per-device analog stick calibration values.
-- ---------------------------------------------------------------------------
CREATE TABLE IF NOT EXISTS calibration_data (
    device_id       INTEGER NOT NULL,                  -- FK -> devices.id
    center_x        REAL    NOT NULL DEFAULT 128.0,    -- Neutral X position (0-255 range)
    center_y        REAL    NOT NULL DEFAULT 128.0,    -- Neutral Y position
    dead_zone       REAL    NOT NULL DEFAULT 10.0,     -- Deadzone radius (same units)
    max_radius      REAL    NOT NULL DEFAULT 110.0,    -- Maximum effective radius
    calibrated_at   TEXT    NOT NULL DEFAULT (datetime('now')),

    PRIMARY KEY (device_id),
    FOREIGN KEY (device_id) REFERENCES devices (id)
        ON UPDATE CASCADE ON DELETE CASCADE
);

-- ---------------------------------------------------------------------------
-- Triggers: auto-update updated_at timestamps
-- ---------------------------------------------------------------------------
CREATE TRIGGER IF NOT EXISTS trg_devices_updated_at
AFTER UPDATE ON devices
BEGIN
    UPDATE devices SET updated_at = datetime('now') WHERE id = NEW.id;
END;

CREATE TRIGGER IF NOT EXISTS trg_mapping_profiles_updated_at
AFTER UPDATE ON mapping_profiles
BEGIN
    UPDATE mapping_profiles SET updated_at = datetime('now') WHERE id = NEW.id;
END;

CREATE TRIGGER IF NOT EXISTS trg_user_settings_updated_at
AFTER UPDATE ON user_settings
BEGIN
    UPDATE user_settings SET updated_at = datetime('now') WHERE key = NEW.key;
END;

-- ---------------------------------------------------------------------------
-- Views: convenience helpers
-- ---------------------------------------------------------------------------

-- All profiles joined with their device info
CREATE VIEW IF NOT EXISTS v_profiles_full AS
SELECT
    mp.id,
    mp.name            AS profile_name,
    mp.ps_mode,
    mp.is_default,
    d.name             AS device_name,
    d.manufacturer,
    d.device_type,
    d.vid,
    d.pid,
    mp.created_at,
    mp.updated_at
FROM mapping_profiles mp
LEFT JOIN devices d ON mp.device_vid = d.vid AND mp.device_pid = d.pid;

-- Enabled plugins summary
CREATE VIEW IF NOT EXISTS v_plugins_enabled AS
SELECT id, name, version, author, path
FROM plugin_registry
WHERE enabled = 1;
