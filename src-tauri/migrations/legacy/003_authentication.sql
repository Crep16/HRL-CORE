ALTER TABLE users ADD COLUMN login_name TEXT;
UPDATE users SET login_name = lower(trim(name));
CREATE UNIQUE INDEX users_login_name ON users(login_name);

CREATE TABLE access_settings (
    id INTEGER PRIMARY KEY CHECK(id = 1),
    master_hash TEXT NOT NULL CHECK(length(master_hash) > 0)
);
