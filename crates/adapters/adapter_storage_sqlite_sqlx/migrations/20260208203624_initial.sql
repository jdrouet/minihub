CREATE TABLE IF NOT EXISTS areas (
    id   TEXT PRIMARY KEY NOT NULL,
    name TEXT NOT NULL,
    parent_id TEXT,
    FOREIGN KEY (parent_id) REFERENCES areas(id) ON DELETE SET NULL
);

CREATE TABLE IF NOT EXISTS devices (
    id           TEXT PRIMARY KEY NOT NULL,
    name         TEXT NOT NULL,
    manufacturer TEXT,
    model        TEXT,
    area_id      TEXT,
    FOREIGN KEY (area_id) REFERENCES areas(id) ON DELETE SET NULL
);

CREATE TABLE IF NOT EXISTS entities (
    id            TEXT PRIMARY KEY NOT NULL,
    device_id     TEXT NOT NULL,
    entity_id     TEXT NOT NULL UNIQUE,
    friendly_name TEXT NOT NULL,
    state         TEXT NOT NULL,
    attributes    TEXT NOT NULL DEFAULT '{}',
    last_changed  TEXT NOT NULL,
    last_updated  TEXT NOT NULL,
    FOREIGN KEY (device_id) REFERENCES devices(id) ON DELETE CASCADE
);

CREATE INDEX idx_entities_device_id ON entities(device_id);
CREATE INDEX idx_entities_entity_id ON entities(entity_id);
