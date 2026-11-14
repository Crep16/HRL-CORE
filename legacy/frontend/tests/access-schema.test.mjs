import assert from 'node:assert/strict';
import { test } from 'node:test';
import { DatabaseSync } from 'node:sqlite';
import { readFileSync, mkdtempSync, rmSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join } from 'node:path';

const schema = readFileSync(new URL('../src-tauri/migrations/001_access.sql', import.meta.url), 'utf8');
test('first account and last administrator are protected in SQLite', () => {
  const db = new DatabaseSync(':memory:');
  try {
    db.exec('PRAGMA foreign_keys=ON');
    db.exec(schema);
    const add = db.prepare('INSERT INTO users(id,name,password_hash,access_level) VALUES (?,?,?,?)');
    assert.throws(() => add.run('l', 'Técnico', 'test-hash', 'limited'));
    add.run('a', 'Administrador', 'test-hash', 'complete');
    assert.throws(() => db.exec("DELETE FROM users WHERE id='a'"));
    assert.throws(() => db.exec("UPDATE users SET active=0 WHERE id='a'"));
    assert.throws(() => db.exec("UPDATE users SET access_level='custom' WHERE id='a'"));
    add.run('b', 'Segundo administrador', 'test-hash', 'complete');
    db.exec("UPDATE users SET active=0 WHERE id='a'");
    assert.throws(() => db.exec("DELETE FROM users WHERE id='b'"));
    const grant = db.prepare('INSERT INTO user_permissions VALUES (?,?,?)');
    grant.run('a', 'clients', 'view');
    assert.throws(() => grant.run('missing', 'clients', 'view'));
    assert.throws(() => grant.run('a', 'company', 'unknown'));
    assert.throws(() => grant.run('a', 'clients', 'view'));
  } finally { db.close(); }
});

test('accounts and individual permissions survive reopening the database', () => {
  const dir = mkdtempSync(join(tmpdir(), 'hrl-access-'));
  const file = join(dir, 'test.db');
  let db;
  try {
    db = new DatabaseSync(file);
    db.exec(schema);
    db.prepare('INSERT INTO users(id,name,password_hash,access_level) VALUES (?,?,?,?)').run('a', 'Administrador', 'test-hash', 'complete');
    db.exec("INSERT INTO user_permissions VALUES ('a','repairs','view')");
    db.close(); db = new DatabaseSync(file);
    assert.equal(db.prepare('SELECT access_level FROM users').get().access_level, 'complete');
    assert.equal(db.prepare('SELECT action FROM user_permissions').get().action, 'view');
  } finally { db?.close(); rmSync(dir, {recursive:true, force:true}); }
});
