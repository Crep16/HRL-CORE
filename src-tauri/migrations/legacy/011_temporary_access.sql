CREATE TABLE temporary_access_grants (
 id TEXT PRIMARY KEY NOT NULL,
 recipient_id TEXT NOT NULL REFERENCES users(id) ON DELETE RESTRICT,
 recipient_name TEXT NOT NULL,
 approver_id TEXT NOT NULL REFERENCES users(id) ON DELETE RESTRICT,
 approver_name TEXT NOT NULL,
 scope TEXT NOT NULL CHECK(scope IN ('clients','repairs','logbook','company','users')),
 record_id TEXT,
 operations_json TEXT NOT NULL CHECK(json_valid(operations_json)),
 reason TEXT NOT NULL CHECK(length(trim(reason))>0),
 created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
 ended_at TEXT,
 end_reason TEXT
);
CREATE TRIGGER temporary_access_start AFTER INSERT ON temporary_access_grants
BEGIN
 INSERT INTO system_logbook_events(record_id,author_id,author_name,operation,details_json,reason)
 VALUES (coalesce(NEW.record_id,NEW.scope),NEW.recipient_id,NEW.recipient_name,'access.temporary.start',
 json_array(json_object('field','temporaryGrant','oldValue',NULL,'newValue',NEW.id),json_object('field','authorizedBy','oldValue',NULL,'newValue',NEW.approver_name),json_object('field','performedBy','oldValue',NULL,'newValue',NEW.recipient_name),json_object('field','temporaryScope','oldValue',NULL,'newValue',NEW.scope),json_object('field','temporaryOperations','oldValue',NULL,'newValue',json(NEW.operations_json))),NEW.reason);
END;
CREATE TRIGGER temporary_access_end AFTER UPDATE ON temporary_access_grants
BEGIN
 INSERT INTO system_logbook_events(record_id,author_id,author_name,operation,details_json,reason)
 VALUES (coalesce(NEW.record_id,NEW.scope),NEW.recipient_id,NEW.recipient_name,'access.temporary.end',
 json_array(json_object('field','temporaryGrant','oldValue',NULL,'newValue',NEW.id),json_object('field','authorizedBy','oldValue',NULL,'newValue',NEW.approver_name),json_object('field','temporaryScope','oldValue',NULL,'newValue',NEW.scope)),NEW.end_reason);
END;
CREATE TRIGGER temporary_access_restrict_update BEFORE UPDATE ON temporary_access_grants
WHEN OLD.ended_at IS NOT NULL OR NEW.ended_at IS NULL OR NEW.id IS NOT OLD.id
 OR NEW.recipient_id IS NOT OLD.recipient_id OR NEW.recipient_name IS NOT OLD.recipient_name
 OR NEW.approver_id IS NOT OLD.approver_id OR NEW.approver_name IS NOT OLD.approver_name
 OR NEW.scope IS NOT OLD.scope OR NEW.record_id IS NOT OLD.record_id
 OR NEW.operations_json IS NOT OLD.operations_json OR NEW.reason IS NOT OLD.reason OR NEW.created_at IS NOT OLD.created_at
BEGIN SELECT RAISE(ABORT,'Temporary grants can only be ended once'); END;
CREATE TRIGGER temporary_access_no_delete BEFORE DELETE ON temporary_access_grants
BEGIN SELECT RAISE(ABORT,'Temporary access history cannot be deleted'); END;
