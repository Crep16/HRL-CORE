CREATE TABLE clients (
    id TEXT PRIMARY KEY NOT NULL,
    name TEXT NOT NULL CHECK(length(trim(name)) > 0),
    phone TEXT NOT NULL DEFAULT '',
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
);

CREATE TABLE equipment (
    id TEXT PRIMARY KEY NOT NULL,
    client_id TEXT NOT NULL REFERENCES clients(id) ON DELETE RESTRICT,
    item TEXT NOT NULL CHECK(length(trim(item)) > 0),
    brand TEXT NOT NULL DEFAULT '',
    model TEXT NOT NULL DEFAULT '',
    serial TEXT NOT NULL DEFAULT ''
);
CREATE INDEX equipment_client ON equipment(client_id);
CREATE INDEX equipment_serial ON equipment(serial);

CREATE TABLE orders (
    id TEXT PRIMARY KEY NOT NULL CHECK(length(trim(id)) > 0),
    equipment_id TEXT NOT NULL REFERENCES equipment(id) ON DELETE RESTRICT,
    created_by TEXT NOT NULL REFERENCES users(id) ON DELETE RESTRICT,
    received TEXT NOT NULL,
    status TEXT NOT NULL CHECK(length(trim(status)) > 0),
    failure TEXT NOT NULL DEFAULT '',
    work TEXT NOT NULL DEFAULT '',
    price_minor INTEGER NOT NULL DEFAULT 0 CHECK(typeof(price_minor) = 'integer' AND price_minor >= 0),
    currency TEXT NOT NULL DEFAULT 'MXN' CHECK(length(currency) = 3),
    other_shop INTEGER NOT NULL DEFAULT 0 CHECK(other_shop IN (0, 1)),
    other_shop_reason TEXT NOT NULL DEFAULT '',
    dropped INTEGER NOT NULL DEFAULT 0 CHECK(dropped IN (0, 1)),
    drop_reason TEXT NOT NULL DEFAULT '',
    delivered_at TEXT,
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
);
CREATE INDEX orders_equipment ON orders(equipment_id);

CREATE TABLE repair_events (
    id TEXT PRIMARY KEY NOT NULL,
    order_id TEXT NOT NULL REFERENCES orders(id) ON DELETE RESTRICT,
    author_id TEXT NOT NULL REFERENCES users(id) ON DELETE RESTRICT,
    stage TEXT NOT NULL,
    title TEXT NOT NULL CHECK(length(trim(title)) > 0),
    detail TEXT NOT NULL DEFAULT '',
    public INTEGER NOT NULL DEFAULT 0 CHECK(public IN (0, 1)),
    occurred_at TEXT NOT NULL
);
CREATE INDEX repair_events_order ON repair_events(order_id, occurred_at);

CREATE TABLE repair_notes (
    id TEXT PRIMARY KEY NOT NULL,
    order_id TEXT NOT NULL REFERENCES orders(id) ON DELETE RESTRICT,
    author_id TEXT NOT NULL REFERENCES users(id) ON DELETE RESTRICT,
    body TEXT NOT NULL CHECK(length(trim(body)) > 0),
    public INTEGER NOT NULL DEFAULT 0 CHECK(public IN (0, 1)),
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
);
CREATE INDEX repair_notes_order ON repair_notes(order_id);

CREATE TABLE order_parts (
    id TEXT PRIMARY KEY NOT NULL,
    order_id TEXT NOT NULL REFERENCES orders(id) ON DELETE RESTRICT,
    author_id TEXT NOT NULL REFERENCES users(id) ON DELETE RESTRICT,
    name TEXT NOT NULL CHECK(length(trim(name)) > 0),
    price_minor INTEGER NOT NULL CHECK(typeof(price_minor) = 'integer' AND price_minor >= 0)
);
CREATE INDEX order_parts_order ON order_parts(order_id);

CREATE TABLE warranties (
    id TEXT PRIMARY KEY NOT NULL,
    order_id TEXT NOT NULL REFERENCES orders(id) ON DELETE RESTRICT,
    author_id TEXT NOT NULL REFERENCES users(id) ON DELETE RESTRICT,
    coverage TEXT NOT NULL CHECK(length(trim(coverage)) > 0),
    days INTEGER NOT NULL CHECK(typeof(days) = 'integer' AND days >= 0)
);
CREATE INDEX warranties_order ON warranties(order_id);

CREATE TABLE payments (
    id TEXT PRIMARY KEY NOT NULL,
    order_id TEXT NOT NULL REFERENCES orders(id) ON DELETE RESTRICT,
    author_id TEXT NOT NULL REFERENCES users(id) ON DELETE RESTRICT,
    amount_minor INTEGER NOT NULL CHECK(typeof(amount_minor) = 'integer' AND amount_minor > 0),
    occurred_at TEXT NOT NULL
);
CREATE INDEX payments_order ON payments(order_id, occurred_at);

CREATE TABLE audit_events (
    id TEXT PRIMARY KEY NOT NULL,
    order_id TEXT NOT NULL REFERENCES orders(id) ON DELETE RESTRICT,
    author_id TEXT NOT NULL REFERENCES users(id) ON DELETE RESTRICT,
    field TEXT NOT NULL CHECK(field IN ('created', 'client', 'phone', 'item', 'brand', 'model', 'serial', 'received', 'status', 'failure', 'work', 'price_minor', 'other_shop', 'other_shop_reason', 'dropped', 'drop_reason', 'delivered_at')),
    old_value TEXT,
    new_value TEXT,
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
);
CREATE INDEX audit_events_order ON audit_events(order_id, created_at);
CREATE INDEX audit_events_author ON audit_events(author_id, created_at);
CREATE TRIGGER audit_events_no_update BEFORE UPDATE ON audit_events
BEGIN SELECT RAISE(ABORT, 'Audit events cannot be edited'); END;
CREATE TRIGGER audit_events_no_delete BEFORE DELETE ON audit_events
BEGIN SELECT RAISE(ABORT, 'Audit events cannot be deleted'); END;
