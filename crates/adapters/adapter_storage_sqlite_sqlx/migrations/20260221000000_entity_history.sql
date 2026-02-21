CREATE TABLE IF NOT EXISTS entity_history (
    id          TEXT PRIMARY KEY NOT NULL,
    entity_id   TEXT NOT NULL,
    state       TEXT NOT NULL,
    attributes  JSON NOT NULL DEFAULT '{}',
    recorded_at TEXT NOT NULL,
    FOREIGN KEY (entity_id) REFERENCES entities(id) ON DELETE CASCADE
);

CREATE INDEX idx_entity_history_entity_recorded ON entity_history(entity_id, recorded_at DESC);
