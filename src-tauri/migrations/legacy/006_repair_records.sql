CREATE TABLE repair_records (
    order_id TEXT PRIMARY KEY NOT NULL REFERENCES orders(id) ON DELETE RESTRICT,
    record_json TEXT NOT NULL CHECK(json_valid(record_json)),
    revision INTEGER NOT NULL CHECK(revision > 0)
);
CREATE TABLE repair_changes (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    order_id TEXT NOT NULL REFERENCES orders(id) ON DELETE RESTRICT,
    author_id TEXT NOT NULL REFERENCES users(id) ON DELETE RESTRICT,
    operation TEXT NOT NULL,
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
);
CREATE INDEX repair_changes_order ON repair_changes(order_id, id);
CREATE TRIGGER repair_changes_no_update BEFORE UPDATE ON repair_changes
BEGIN SELECT RAISE(ABORT, 'Repair history cannot be edited'); END;
CREATE TRIGGER repair_changes_no_delete BEFORE DELETE ON repair_changes
BEGIN SELECT RAISE(ABORT, 'Repair history cannot be deleted'); END;
