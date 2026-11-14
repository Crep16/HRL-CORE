ALTER TABLE orders ADD COLUMN record_json TEXT CHECK(record_json IS NULL OR json_valid(record_json));
CREATE TABLE legacy_imports (
    source TEXT PRIMARY KEY NOT NULL,
    imported_by TEXT NOT NULL REFERENCES users(id) ON DELETE RESTRICT,
    imported_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
);
