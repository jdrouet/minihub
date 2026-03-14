-- Add optional MAC address column to entities (nullable, no default).
ALTER TABLE entities ADD COLUMN mac_address TEXT;
