CREATE TABLE order_visibility_events (
    sequence INTEGER PRIMARY KEY AUTOINCREMENT,
    order_id TEXT NOT NULL REFERENCES orders(id) ON DELETE RESTRICT,
    author_id TEXT NOT NULL REFERENCES users(id) ON DELETE RESTRICT,
    action TEXT NOT NULL CHECK(action IN ('deleted', 'restored')),
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
);
CREATE INDEX order_visibility_order ON order_visibility_events(order_id, sequence);
CREATE TRIGGER order_visibility_no_update BEFORE UPDATE ON order_visibility_events
BEGIN SELECT RAISE(ABORT, 'Order visibility history cannot be edited'); END;
CREATE TRIGGER order_visibility_no_delete BEFORE DELETE ON order_visibility_events
BEGIN SELECT RAISE(ABORT, 'Order visibility history cannot be deleted'); END;
