CREATE TABLE users (
    id TEXT PRIMARY KEY NOT NULL,
    name TEXT NOT NULL CHECK(length(trim(name)) > 0),
    password_hash TEXT NOT NULL CHECK(length(password_hash) > 0),
    access_level TEXT NOT NULL CHECK(access_level IN ('complete', 'limited', 'custom')),
    active INTEGER NOT NULL DEFAULT 1 CHECK(active IN (0, 1)),
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
);

CREATE TABLE user_permissions (
    user_id TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    section TEXT NOT NULL CHECK(section IN ('clients', 'repairs', 'logbook', 'company')),
    action TEXT NOT NULL CHECK(action IN ('view', 'create', 'edit', 'delete')),
    PRIMARY KEY(user_id, section, action)
);

CREATE TRIGGER first_user_must_be_complete
BEFORE INSERT ON users
WHEN NOT EXISTS (SELECT 1 FROM users) AND (NEW.access_level <> 'complete' OR NEW.active <> 1)
BEGIN
    SELECT RAISE(ABORT, 'The first user must have complete access');
END;

CREATE TRIGGER preserve_last_complete_user_update
BEFORE UPDATE ON users
WHEN OLD.access_level = 'complete' AND OLD.active = 1
 AND (NEW.access_level <> 'complete' OR NEW.active <> 1)
 AND NOT EXISTS (SELECT 1 FROM users WHERE id <> OLD.id AND access_level = 'complete' AND active = 1)
BEGIN
    SELECT RAISE(ABORT, 'At least one active complete user is required');
END;

CREATE TRIGGER preserve_last_complete_user_delete
BEFORE DELETE ON users
WHEN OLD.access_level = 'complete' AND OLD.active = 1
 AND NOT EXISTS (SELECT 1 FROM users WHERE id <> OLD.id AND access_level = 'complete' AND active = 1)
BEGIN
    SELECT RAISE(ABORT, 'At least one active complete user is required');
END;
