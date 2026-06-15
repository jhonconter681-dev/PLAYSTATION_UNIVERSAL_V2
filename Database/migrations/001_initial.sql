-- =============================================================================
-- Migration 001 — Initial Schema
-- PUCE (PlayStation Universal Controller Emulator)
-- Applied once on fresh database creation.
-- =============================================================================

-- Record this migration
INSERT OR IGNORE INTO schema_migrations (version) VALUES ('001');

-- ---------------------------------------------------------------------------
-- Create all core tables (identical to schema.sql bootstrap)
-- These statements are idempotent via IF NOT EXISTS.
-- ---------------------------------------------------------------------------

CREATE TABLE IF NOT EXISTS schema_migrations (
    version    TEXT NOT NULL PRIMARY KEY,
    applied_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS devices (
    id           INTEGER PRIMARY KEY AUTOINCREMENT,
    vid          INTEGER NOT NULL,
    pid          INTEGER NOT NULL,
    name         TEXT    NOT NULL,
    manufacturer TEXT    NOT NULL DEFAULT '',
    device_type  TEXT    NOT NULL DEFAULT 'gamepad',
    button_count INTEGER NOT NULL DEFAULT 0,
    axis_count   INTEGER NOT NULL DEFAULT 0,
    sensors      TEXT    NOT NULL DEFAULT '[]',
    capabilities TEXT    NOT NULL DEFAULT '{}',
    firmware     TEXT             DEFAULT NULL,
    bt_version   TEXT             DEFAULT NULL,
    usb_version  TEXT             DEFAULT NULL,
    created_at   TEXT    NOT NULL DEFAULT (datetime('now')),
    updated_at   TEXT    NOT NULL DEFAULT (datetime('now')),
    UNIQUE (vid, pid)
);

CREATE INDEX IF NOT EXISTS idx_devices_vid_pid    ON devices (vid, pid);
CREATE INDEX IF NOT EXISTS idx_devices_device_type ON devices (device_type);

CREATE TABLE IF NOT EXISTS mapping_profiles (
    id              TEXT    PRIMARY KEY,
    name            TEXT    NOT NULL,
    device_vid      INTEGER          DEFAULT NULL,
    device_pid      INTEGER          DEFAULT NULL,
    ps_mode         TEXT    NOT NULL DEFAULT 'ds4',
    button_mappings TEXT    NOT NULL DEFAULT '{}',
    axis_mappings   TEXT    NOT NULL DEFAULT '{}',
    virtual_buttons TEXT    NOT NULL DEFAULT '[]',
    is_default      INTEGER NOT NULL DEFAULT 0,
    created_at      TEXT    NOT NULL DEFAULT (datetime('now')),
    updated_at      TEXT    NOT NULL DEFAULT (datetime('now')),
    FOREIGN KEY (device_vid, device_pid) REFERENCES devices (vid, pid)
        ON UPDATE CASCADE ON DELETE SET NULL
);

CREATE INDEX IF NOT EXISTS idx_mapping_profiles_device     ON mapping_profiles (device_vid, device_pid);
CREATE INDEX IF NOT EXISTS idx_mapping_profiles_ps_mode    ON mapping_profiles (ps_mode);
CREATE INDEX IF NOT EXISTS idx_mapping_profiles_is_default ON mapping_profiles (is_default);

CREATE TABLE IF NOT EXISTS user_settings (
    key        TEXT PRIMARY KEY,
    value      TEXT NOT NULL DEFAULT '',
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS plugin_registry (
    id             INTEGER PRIMARY KEY AUTOINCREMENT,
    name           TEXT    NOT NULL UNIQUE,
    version        TEXT    NOT NULL,
    author         TEXT    NOT NULL DEFAULT '',
    path           TEXT    NOT NULL,
    signature_hash TEXT    NOT NULL,
    enabled        INTEGER NOT NULL DEFAULT 1,
    installed_at   TEXT    NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX IF NOT EXISTS idx_plugin_registry_enabled ON plugin_registry (enabled);
CREATE INDEX IF NOT EXISTS idx_plugin_registry_name    ON plugin_registry (name);

CREATE TABLE IF NOT EXISTS calibration_data (
    device_id     INTEGER NOT NULL,
    center_x      REAL    NOT NULL DEFAULT 128.0,
    center_y      REAL    NOT NULL DEFAULT 128.0,
    dead_zone     REAL    NOT NULL DEFAULT 10.0,
    max_radius    REAL    NOT NULL DEFAULT 110.0,
    calibrated_at TEXT    NOT NULL DEFAULT (datetime('now')),
    PRIMARY KEY (device_id),
    FOREIGN KEY (device_id) REFERENCES devices (id) ON UPDATE CASCADE ON DELETE CASCADE
);

-- ---------------------------------------------------------------------------
-- Triggers
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
-- Default user settings inserted on first run
-- ---------------------------------------------------------------------------

INSERT OR IGNORE INTO user_settings (key, value) VALUES
    ('theme',                   'dark'),
    ('language',                'en'),
    ('auto_detect',             '1'),
    ('emulation_mode',          'ds4'),
    ('start_minimized',         '0'),
    ('show_notifications',      '1'),
    ('check_updates',           '1'),
    ('deadzone_default',        '10'),
    ('plugin_verify_signature', '1'),
    ('log_level',               'info');
