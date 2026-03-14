-- Migrate UUID columns from TEXT to BLOB for proper sqlx Uuid encoding.
-- SQLite requires table recreation to change column types.

-- ── areas ────────────────────────────────────────────────────────────

CREATE TABLE areas_new (
    id        BLOB PRIMARY KEY NOT NULL,
    name      TEXT NOT NULL,
    parent_id BLOB,
    FOREIGN KEY (parent_id) REFERENCES areas_new(id) ON DELETE SET NULL
);

INSERT INTO areas_new (id, name, parent_id)
SELECT unhex(replace(id, '-', '')), name, unhex(replace(parent_id, '-', '')) FROM areas;

DROP TABLE areas;
ALTER TABLE areas_new RENAME TO areas;

-- ── devices ──────────────────────────────────────────────────────────

CREATE TABLE devices_new (
    id           BLOB PRIMARY KEY NOT NULL,
    name         TEXT NOT NULL,
    manufacturer TEXT,
    model        TEXT,
    area_id      BLOB,
    integration  TEXT NOT NULL DEFAULT '',
    unique_id    TEXT NOT NULL DEFAULT '',
    FOREIGN KEY (area_id) REFERENCES areas(id) ON DELETE SET NULL
);

INSERT INTO devices_new (id, name, manufacturer, model, area_id, integration, unique_id)
SELECT unhex(replace(id, '-', '')), name, manufacturer, model, unhex(replace(area_id, '-', '')), integration, unique_id FROM devices;

DROP TABLE devices;
ALTER TABLE devices_new RENAME TO devices;

CREATE UNIQUE INDEX idx_devices_integration_unique_id ON devices(integration, unique_id);

-- ── entities ─────────────────────────────────────────────────────────

CREATE TABLE entities_new (
    id            BLOB PRIMARY KEY NOT NULL,
    device_id     BLOB NOT NULL,
    entity_id     TEXT NOT NULL UNIQUE,
    friendly_name TEXT NOT NULL,
    state         TEXT NOT NULL,
    attributes    TEXT NOT NULL DEFAULT '{}',
    last_changed  TEXT NOT NULL,
    last_updated  TEXT NOT NULL,
    FOREIGN KEY (device_id) REFERENCES devices(id) ON DELETE CASCADE
);

INSERT INTO entities_new (id, device_id, entity_id, friendly_name, state, attributes, last_changed, last_updated)
SELECT unhex(replace(id, '-', '')), unhex(replace(device_id, '-', '')), entity_id, friendly_name, state, attributes, last_changed, last_updated FROM entities;

DROP TABLE entities;
ALTER TABLE entities_new RENAME TO entities;

CREATE INDEX idx_entities_device_id ON entities(device_id);
CREATE INDEX idx_entities_entity_id ON entities(entity_id);

-- ── events ───────────────────────────────────────────────────────────

CREATE TABLE events_new (
    id         BLOB PRIMARY KEY NOT NULL,
    event_type TEXT NOT NULL,
    entity_id  BLOB,
    timestamp  TEXT NOT NULL,
    data       JSON NOT NULL DEFAULT '{}',
    FOREIGN KEY (entity_id) REFERENCES entities(id) ON DELETE SET NULL
);

INSERT INTO events_new (id, event_type, entity_id, timestamp, data)
SELECT unhex(replace(id, '-', '')), event_type, unhex(replace(entity_id, '-', '')), timestamp, data FROM events;

DROP TABLE events;
ALTER TABLE events_new RENAME TO events;

CREATE INDEX idx_events_timestamp ON events(timestamp DESC);
CREATE INDEX idx_events_entity_id ON events(entity_id, timestamp DESC);

-- ── automations ──────────────────────────────────────────────────────

CREATE TABLE automations_new (
    id              BLOB    PRIMARY KEY NOT NULL,
    name            TEXT    NOT NULL,
    enabled         INTEGER NOT NULL DEFAULT 1,
    trigger_data    JSON    NOT NULL DEFAULT '{}',
    conditions      JSON    NOT NULL DEFAULT '[]',
    actions         JSON    NOT NULL DEFAULT '[]',
    last_triggered  TEXT
);

INSERT INTO automations_new (id, name, enabled, trigger_data, conditions, actions, last_triggered)
SELECT unhex(replace(id, '-', '')), name, enabled, trigger_data, conditions, actions, last_triggered FROM automations;

DROP TABLE automations;
ALTER TABLE automations_new RENAME TO automations;
