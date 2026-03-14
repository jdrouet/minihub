ALTER TABLE devices ADD COLUMN integration TEXT NOT NULL DEFAULT '';
ALTER TABLE devices ADD COLUMN unique_id TEXT NOT NULL DEFAULT '';

CREATE UNIQUE INDEX idx_devices_integration_unique_id ON devices(integration, unique_id);
