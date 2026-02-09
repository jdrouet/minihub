CREATE TABLE IF NOT EXISTS automations (
    id              TEXT    PRIMARY KEY NOT NULL,
    name            TEXT    NOT NULL,
    enabled         INTEGER NOT NULL DEFAULT 1,
    trigger_data    JSON    NOT NULL DEFAULT '{}',
    conditions      JSON    NOT NULL DEFAULT '[]',
    actions         JSON    NOT NULL DEFAULT '[]',
    last_triggered  TEXT
);
