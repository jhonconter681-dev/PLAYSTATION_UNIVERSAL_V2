-- =============================================================================
-- Migration 002 — Profiles v2: Export / Import support
-- PUCE (PlayStation Universal Controller Emulator)
-- =============================================================================

-- Guard: skip if already applied
INSERT OR IGNORE INTO schema_migrations (version) VALUES ('002');

-- ---------------------------------------------------------------------------
-- Add export/import columns to mapping_profiles
-- ---------------------------------------------------------------------------

ALTER TABLE mapping_profiles ADD COLUMN export_version TEXT DEFAULT NULL;
ALTER TABLE mapping_profiles ADD COLUMN export_hash    TEXT DEFAULT NULL;
ALTER TABLE mapping_profiles ADD COLUMN source_url     TEXT DEFAULT NULL;

-- ---------------------------------------------------------------------------
-- New table: profile_tags — user-defined tags per profile
-- ---------------------------------------------------------------------------

CREATE TABLE IF NOT EXISTS profile_tags (
    profile_id  TEXT    NOT NULL REFERENCES mapping_profiles (id) ON DELETE CASCADE,
    tag         TEXT    NOT NULL,
    PRIMARY KEY (profile_id, tag)
);

CREATE INDEX IF NOT EXISTS idx_profile_tags_tag ON profile_tags (tag);

-- ---------------------------------------------------------------------------
-- New table: profile_shares — community sharing metadata
-- ---------------------------------------------------------------------------

CREATE TABLE IF NOT EXISTS profile_shares (
    id             INTEGER PRIMARY KEY AUTOINCREMENT,
    profile_id     TEXT    NOT NULL REFERENCES mapping_profiles (id) ON DELETE CASCADE,
    share_token    TEXT    NOT NULL UNIQUE,          -- short unique share code
    share_url      TEXT             DEFAULT NULL,
    downloads      INTEGER NOT NULL DEFAULT 0,
    rating         REAL             DEFAULT NULL,    -- 0.0 – 5.0
    shared_at      TEXT    NOT NULL DEFAULT (datetime('now')),
    expires_at     TEXT             DEFAULT NULL
);

CREATE INDEX IF NOT EXISTS idx_profile_shares_profile_id  ON profile_shares (profile_id);
CREATE INDEX IF NOT EXISTS idx_profile_shares_share_token ON profile_shares (share_token);

-- ---------------------------------------------------------------------------
-- New table: profile_history — undo/redo audit log
-- ---------------------------------------------------------------------------

CREATE TABLE IF NOT EXISTS profile_history (
    id              INTEGER PRIMARY KEY AUTOINCREMENT,
    profile_id      TEXT    NOT NULL REFERENCES mapping_profiles (id) ON DELETE CASCADE,
    changed_at      TEXT    NOT NULL DEFAULT (datetime('now')),
    change_type     TEXT    NOT NULL,                -- 'create','update','delete'
    previous_state  TEXT             DEFAULT NULL,   -- JSON snapshot before change
    new_state       TEXT             DEFAULT NULL    -- JSON snapshot after change
);

CREATE INDEX IF NOT EXISTS idx_profile_history_profile_id ON profile_history (profile_id);
CREATE INDEX IF NOT EXISTS idx_profile_history_changed_at ON profile_history (changed_at);

-- ---------------------------------------------------------------------------
-- New table: device_aliases — alternative VID/PID combos for same physical device
-- (e.g. DS3 shows up with different PID when connected via USB vs Bluetooth)
-- ---------------------------------------------------------------------------

CREATE TABLE IF NOT EXISTS device_aliases (
    canonical_vid   INTEGER NOT NULL,
    canonical_pid   INTEGER NOT NULL,
    alias_vid       INTEGER NOT NULL,
    alias_pid       INTEGER NOT NULL,
    note            TEXT    DEFAULT NULL,
    PRIMARY KEY (alias_vid, alias_pid),
    FOREIGN KEY (canonical_vid, canonical_pid) REFERENCES devices (vid, pid)
        ON UPDATE CASCADE ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_device_aliases_canonical ON device_aliases (canonical_vid, canonical_pid);

-- ---------------------------------------------------------------------------
-- Extend calibration_data with per-axis entries
-- ---------------------------------------------------------------------------

ALTER TABLE calibration_data ADD COLUMN right_center_x REAL NOT NULL DEFAULT 128.0;
ALTER TABLE calibration_data ADD COLUMN right_center_y REAL NOT NULL DEFAULT 128.0;
ALTER TABLE calibration_data ADD COLUMN right_dead_zone REAL NOT NULL DEFAULT 10.0;
ALTER TABLE calibration_data ADD COLUMN right_max_radius REAL NOT NULL DEFAULT 110.0;

-- ---------------------------------------------------------------------------
-- View: enriched profile listing with tags
-- ---------------------------------------------------------------------------

CREATE VIEW IF NOT EXISTS v_profiles_tagged AS
SELECT
    mp.id,
    mp.name,
    mp.ps_mode,
    mp.is_default,
    mp.export_version,
    mp.export_hash,
    mp.source_url,
    GROUP_CONCAT(pt.tag, ',') AS tags,
    d.name AS device_name,
    d.vid,
    d.pid
FROM mapping_profiles mp
LEFT JOIN profile_tags pt ON pt.profile_id = mp.id
LEFT JOIN devices d ON mp.device_vid = d.vid AND mp.device_pid = d.pid
GROUP BY mp.id;
