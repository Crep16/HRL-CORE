ALTER TABLE repair_changes ADD COLUMN details_json TEXT CHECK(details_json IS NULL OR json_valid(details_json));

CREATE TABLE logbook_events (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    source TEXT NOT NULL,
    source_id TEXT NOT NULL,
    order_id TEXT NOT NULL REFERENCES orders(id) ON DELETE RESTRICT,
    author_id TEXT NOT NULL REFERENCES users(id) ON DELETE RESTRICT,
    author_name TEXT NOT NULL,
    operation TEXT NOT NULL,
    field TEXT,
    old_value TEXT,
    new_value TEXT,
    details_json TEXT CHECK(details_json IS NULL OR json_valid(details_json)),
    occurred_at TEXT,
    event_date TEXT,
    UNIQUE(source, source_id)
);
CREATE INDEX logbook_date ON logbook_events(occurred_at DESC, id DESC);
CREATE INDEX logbook_author ON logbook_events(author_id, id DESC);
CREATE INDEX logbook_order ON logbook_events(order_id, id DESC);

INSERT INTO logbook_events(source,source_id,order_id,author_id,author_name,operation,field,old_value,new_value,occurred_at)
SELECT 'order',a.id,a.order_id,a.author_id,u.name,CASE WHEN a.field='created' THEN 'orders.create' ELSE 'orders.edit' END,a.field,a.old_value,a.new_value,a.created_at
FROM audit_events a JOIN users u ON u.id=a.author_id;
INSERT INTO logbook_events(source,source_id,order_id,author_id,author_name,operation,new_value,event_date)
SELECT 'payment',p.id,p.order_id,p.author_id,u.name,'payments.add',CAST(p.amount_minor AS TEXT),nullif(p.occurred_at,'')
FROM payments p JOIN users u ON u.id=p.author_id;
INSERT INTO logbook_events(source,source_id,order_id,author_id,author_name,operation,details_json,occurred_at)
SELECT 'repair',CAST(r.id AS TEXT),r.order_id,r.author_id,u.name,r.operation,r.details_json,r.created_at
FROM repair_changes r JOIN users u ON u.id=r.author_id WHERE r.operation<>'payments.add';
INSERT INTO logbook_events(source,source_id,order_id,author_id,author_name,operation,occurred_at)
SELECT 'visibility',CAST(v.sequence AS TEXT),v.order_id,v.author_id,u.name,CASE WHEN v.action='deleted' THEN 'orders.delete' ELSE 'orders.restore' END,v.created_at
FROM order_visibility_events v JOIN users u ON u.id=v.author_id;

CREATE TRIGGER logbook_order_insert AFTER INSERT ON audit_events
BEGIN
    INSERT INTO logbook_events(source,source_id,order_id,author_id,author_name,operation,field,old_value,new_value,occurred_at)
    VALUES ('order',NEW.id,NEW.order_id,NEW.author_id,(SELECT name FROM users WHERE id=NEW.author_id),CASE WHEN NEW.field='created' THEN 'orders.create' ELSE 'orders.edit' END,NEW.field,NEW.old_value,NEW.new_value,NEW.created_at);
END;
CREATE TRIGGER logbook_payment_insert AFTER INSERT ON payments
BEGIN
    INSERT INTO logbook_events(source,source_id,order_id,author_id,author_name,operation,new_value,occurred_at,event_date)
    VALUES ('payment',NEW.id,NEW.order_id,NEW.author_id,(SELECT name FROM users WHERE id=NEW.author_id),'payments.add',CAST(NEW.amount_minor AS TEXT),strftime('%Y-%m-%dT%H:%M:%fZ','now'),nullif(NEW.occurred_at,''));
END;
CREATE TRIGGER logbook_repair_insert AFTER INSERT ON repair_changes
BEGIN
    INSERT INTO logbook_events(source,source_id,order_id,author_id,author_name,operation,details_json,occurred_at)
    SELECT 'repair',CAST(NEW.id AS TEXT),NEW.order_id,NEW.author_id,u.name,NEW.operation,NEW.details_json,NEW.created_at
    FROM users u WHERE u.id=NEW.author_id AND NEW.operation<>'payments.add';
END;
CREATE TRIGGER logbook_visibility_insert AFTER INSERT ON order_visibility_events
BEGIN
    INSERT INTO logbook_events(source,source_id,order_id,author_id,author_name,operation,occurred_at)
    VALUES ('visibility',CAST(NEW.sequence AS TEXT),NEW.order_id,NEW.author_id,(SELECT name FROM users WHERE id=NEW.author_id),CASE WHEN NEW.action='deleted' THEN 'orders.delete' ELSE 'orders.restore' END,NEW.created_at);
END;
CREATE TRIGGER logbook_no_update BEFORE UPDATE ON logbook_events
BEGIN SELECT RAISE(ABORT,'Logbook events cannot be edited'); END;
CREATE TRIGGER logbook_no_delete BEFORE DELETE ON logbook_events
BEGIN SELECT RAISE(ABORT,'Logbook events cannot be deleted'); END;
