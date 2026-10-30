ALTER TABLE logbook_events ADD COLUMN reason TEXT;
ALTER TABLE logbook_events ADD COLUMN result TEXT NOT NULL DEFAULT 'success';
CREATE TABLE system_logbook_events (
 id INTEGER PRIMARY KEY AUTOINCREMENT,
 record_id TEXT NOT NULL,
 author_id TEXT NOT NULL REFERENCES users(id) ON DELETE RESTRICT,
 author_name TEXT NOT NULL,
 operation TEXT NOT NULL,
 details_json TEXT NOT NULL CHECK(json_valid(details_json)),
 reason TEXT,
 result TEXT NOT NULL DEFAULT 'success',
 occurred_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now'))
);
CREATE TRIGGER system_logbook_no_update BEFORE UPDATE ON system_logbook_events
BEGIN SELECT RAISE(ABORT,'System history cannot be edited'); END;
CREATE TRIGGER system_logbook_no_delete BEFORE DELETE ON system_logbook_events
BEGIN SELECT RAISE(ABORT,'System history cannot be deleted'); END;

CREATE TABLE direct_expenses (
 id TEXT PRIMARY KEY NOT NULL,
 order_id TEXT NOT NULL REFERENCES orders(id) ON DELETE RESTRICT,
 author_id TEXT NOT NULL REFERENCES users(id) ON DELETE RESTRICT,
 description TEXT NOT NULL CHECK(length(trim(description))>0),
 amount_minor INTEGER NOT NULL CHECK(typeof(amount_minor)='integer' AND amount_minor>0),
 reason TEXT NOT NULL CHECK(length(trim(reason))>0),
 occurred_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now'))
);
CREATE TRIGGER direct_expense_audit AFTER INSERT ON direct_expenses
BEGIN
 INSERT INTO logbook_events(source,source_id,order_id,author_id,author_name,operation,field,new_value,occurred_at,reason,details_json)
 VALUES ('expense',NEW.id,NEW.order_id,NEW.author_id,(SELECT name FROM users WHERE id=NEW.author_id),'costs.add','direct_cost_minor',CAST(NEW.amount_minor AS TEXT),NEW.occurred_at,NEW.reason,json_array(json_object('field','expenseDescription','oldValue',NULL,'newValue',NEW.description),json_object('field','direct_cost_minor','oldValue',NULL,'newValue',NEW.amount_minor)));
END;
CREATE TRIGGER direct_expenses_no_update BEFORE UPDATE ON direct_expenses
BEGIN SELECT RAISE(ABORT,'Direct expenses cannot be edited'); END;
CREATE TRIGGER direct_expenses_no_delete BEFORE DELETE ON direct_expenses
BEGIN SELECT RAISE(ABORT,'Direct expenses cannot be deleted'); END;

CREATE VIEW logbook_all AS
 SELECT id,order_id,author_id,author_name,operation,field,old_value,new_value,details_json,occurred_at,event_date,reason,result FROM logbook_events
 UNION ALL
 SELECT -id,record_id,author_id,author_name,operation,NULL,NULL,NULL,details_json,occurred_at,NULL,reason,result FROM system_logbook_events;
