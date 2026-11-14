CREATE TABLE order_evidence (
    id TEXT PRIMARY KEY NOT NULL CHECK(length(id)=32),
    order_id TEXT NOT NULL REFERENCES orders(id) ON DELETE RESTRICT,
    original_name TEXT NOT NULL,
    mime TEXT NOT NULL CHECK(mime IN ('image/jpeg','image/png','image/webp')),
    size_bytes INTEGER NOT NULL CHECK(size_bytes>0 AND size_bytes<=20971520),
    source TEXT NOT NULL CHECK(source IN ('camera','file')),
    note TEXT NOT NULL DEFAULT '',
    initial_note TEXT NOT NULL DEFAULT '',
    author_id TEXT NOT NULL REFERENCES users(id) ON DELETE RESTRICT,
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
    deleted INTEGER NOT NULL DEFAULT 0 CHECK(deleted IN (0,1)),
    revision INTEGER NOT NULL DEFAULT 1 CHECK(revision>0)
);
CREATE INDEX order_evidence_order ON order_evidence(order_id,deleted,created_at);
CREATE TRIGGER evidence_no_delete BEFORE DELETE ON order_evidence
BEGIN SELECT RAISE(ABORT,'Evidence must be retained'); END;
CREATE TRIGGER evidence_preserve_original BEFORE UPDATE ON order_evidence
WHEN NEW.id IS NOT OLD.id OR NEW.order_id IS NOT OLD.order_id
 OR NEW.original_name IS NOT OLD.original_name OR NEW.mime IS NOT OLD.mime
 OR NEW.size_bytes IS NOT OLD.size_bytes OR NEW.source IS NOT OLD.source
 OR NEW.initial_note IS NOT OLD.initial_note OR NEW.author_id IS NOT OLD.author_id
 OR NEW.created_at IS NOT OLD.created_at
BEGIN SELECT RAISE(ABORT,'Original evidence cannot be replaced'); END;
