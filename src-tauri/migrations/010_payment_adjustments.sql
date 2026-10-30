ALTER TABLE user_permissions RENAME TO user_permissions_before_money;
CREATE TABLE user_permissions (
 user_id TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
 section TEXT NOT NULL CHECK(section IN ('clients','repairs','logbook','company','payments')),
 action TEXT NOT NULL,
 CHECK((section='payments' AND action IN ('add','refund')) OR (section<>'payments' AND action IN ('view','create','edit','delete'))),
 PRIMARY KEY(user_id,section,action)
);
INSERT INTO user_permissions SELECT * FROM user_permissions_before_money;
INSERT OR IGNORE INTO user_permissions
 SELECT p.user_id,'payments','add' FROM user_permissions_before_money p JOIN users u ON u.id=p.user_id
 WHERE p.section='repairs' AND p.action='edit' AND u.access_level='custom';
DROP TABLE user_permissions_before_money;

ALTER TABLE payments ADD COLUMN method TEXT;
ALTER TABLE payments ADD COLUMN reason TEXT;
DROP TRIGGER logbook_payment_insert;
CREATE TRIGGER logbook_payment_insert AFTER INSERT ON payments
BEGIN
 INSERT INTO logbook_events(source,source_id,order_id,author_id,author_name,operation,new_value,occurred_at,event_date,reason,details_json)
 VALUES ('payment',NEW.id,NEW.order_id,NEW.author_id,(SELECT name FROM users WHERE id=NEW.author_id),'payments.add',CAST(NEW.amount_minor AS TEXT),strftime('%Y-%m-%dT%H:%M:%fZ','now'),nullif(NEW.occurred_at,''),NEW.reason,
 json_array(json_object('field','amount_minor','oldValue',NULL,'newValue',NEW.amount_minor),json_object('field','paymentMethod','oldValue',NULL,'newValue',NEW.method)));
END;
CREATE TABLE payment_requests (
 id TEXT PRIMARY KEY NOT NULL,
 author_id TEXT NOT NULL REFERENCES users(id) ON DELETE RESTRICT,
 request_json TEXT NOT NULL CHECK(json_valid(request_json))
);
CREATE TRIGGER payment_requests_no_update BEFORE UPDATE ON payment_requests
BEGIN SELECT RAISE(ABORT,'Payment requests cannot be edited'); END;
CREATE TRIGGER payment_requests_no_delete BEFORE DELETE ON payment_requests
BEGIN SELECT RAISE(ABORT,'Payment requests cannot be deleted'); END;
CREATE TABLE payment_operations (
 id TEXT PRIMARY KEY NOT NULL,
 order_id TEXT NOT NULL REFERENCES orders(id) ON DELETE RESTRICT,
 author_id TEXT NOT NULL REFERENCES users(id) ON DELETE RESTRICT,
 kind TEXT NOT NULL CHECK(kind IN ('adjustment','refund')),
 payment_id TEXT REFERENCES payments(id) ON DELETE RESTRICT,
 amount_minor INTEGER NOT NULL CHECK(typeof(amount_minor)='integer' AND amount_minor<>0 AND abs(amount_minor)<=9000000000000),
 expected_minor INTEGER NOT NULL,
 method TEXT,
 reason TEXT NOT NULL CHECK(length(trim(reason))>0),
 occurred_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
 CHECK((kind='adjustment' AND payment_id IS NOT NULL AND method IS NULL) OR (kind='refund' AND payment_id IS NULL AND amount_minor>0 AND length(trim(method))>0))
);
CREATE INDEX payment_operations_order ON payment_operations(order_id,occurred_at,id);
CREATE INDEX payment_operations_payment ON payment_operations(payment_id);
CREATE VIEW order_payment_totals AS
 SELECT o.id,
 coalesce((SELECT sum(amount_minor) FROM payments WHERE order_id=o.id),0) AS received_minor,
 coalesce((SELECT sum(amount_minor) FROM payment_operations WHERE order_id=o.id AND kind='adjustment'),0) AS adjusted_minor,
 coalesce((SELECT sum(amount_minor) FROM payment_operations WHERE order_id=o.id AND kind='refund'),0) AS refunded_minor,
 coalesce((SELECT sum(amount_minor) FROM payments WHERE order_id=o.id),0)
 +coalesce((SELECT sum(amount_minor) FROM payment_operations WHERE order_id=o.id AND kind='adjustment'),0)
 -coalesce((SELECT sum(amount_minor) FROM payment_operations WHERE order_id=o.id AND kind='refund'),0) AS net_minor
 FROM orders o;
CREATE TRIGGER payment_operations_audit AFTER INSERT ON payment_operations
BEGIN
 INSERT INTO logbook_events(source,source_id,order_id,author_id,author_name,operation,field,new_value,occurred_at,reason,details_json)
 VALUES ('money',NEW.id,NEW.order_id,NEW.author_id,(SELECT name FROM users WHERE id=NEW.author_id),
 CASE NEW.kind WHEN 'refund' THEN 'payments.refund' ELSE 'payments.adjust' END,'amount_minor',CAST(NEW.amount_minor AS TEXT),NEW.occurred_at,NEW.reason,
 json_array(json_object('field','amount_minor','oldValue',NULL,'newValue',NEW.amount_minor),json_object('field','paymentReference','oldValue',NULL,'newValue',NEW.payment_id),json_object('field','paymentMethod','oldValue',NULL,'newValue',NEW.method)));
END;
CREATE TRIGGER payment_operations_no_update BEFORE UPDATE ON payment_operations
BEGIN SELECT RAISE(ABORT,'Payment operations cannot be edited'); END;
CREATE TRIGGER payment_operations_no_delete BEFORE DELETE ON payment_operations
BEGIN SELECT RAISE(ABORT,'Payment operations cannot be deleted'); END;
CREATE TRIGGER payments_no_update BEFORE UPDATE ON payments
BEGIN SELECT RAISE(ABORT,'Payments cannot be edited'); END;
CREATE TRIGGER payments_no_delete BEFORE DELETE ON payments
BEGIN SELECT RAISE(ABORT,'Payments cannot be deleted'); END;
