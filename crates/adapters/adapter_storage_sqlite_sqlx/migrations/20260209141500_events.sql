CREATE TABLE IF NOT EXISTS events (
    id         TEXT    PRIMARY KEY NOT NULL,
    event_type TEXT    NOT NULL,
    entity_id  TEXT,
    timestamp  TEXT    NOT NULL,
    data       JSON    NOT NULL DEFAULT '{}',
    FOREIGN KEY (entity_id) REFERENCES entities(id) ON DELETE SET NULL
);

CREATE INDEX idx_events_timestamp ON events(timestamp DESC);
CREATE INDEX idx_events_entity_id ON events(entity_id, timestamp DESC);
