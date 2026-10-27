import assert from 'node:assert/strict';
import { test } from 'node:test';
import { DatabaseSync } from 'node:sqlite';
import { readFileSync, mkdtempSync, rmSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join } from 'node:path';

const migrations = ['001_access.sql', '002_workshop.sql'].map(name => readFileSync(new URL(`../src-tauri/migrations/${name}`, import.meta.url), 'utf8'));
function seed(db) {
  db.exec("INSERT INTO users(id,name,password_hash,access_level) VALUES ('a','Admin','test-only','complete')");
  db.exec("INSERT INTO clients(id,name,phone) VALUES ('c','Cliente','9980000000')");
  db.exec("INSERT INTO equipment(id,client_id,item,serial) VALUES ('e','c','Laptop','SERIE')");
  db.exec("INSERT INTO orders(id,equipment_id,created_by,received,status,price_minor) VALUES ('ORD-001','e','a','2026-10-27','Diagnóstico',150000)");
}

test('workshop migration upgrades existing access data and keeps an order on reopen', () => {
  const dir = mkdtempSync(join(tmpdir(), 'hrl-workshop-'));
  const file = join(dir, 'test.db');
  let db;
  try {
    db = new DatabaseSync(file);
    db.exec('PRAGMA foreign_keys=ON');
    db.exec(migrations[0]);
    db.exec("INSERT INTO users(id,name,password_hash,access_level) VALUES ('old','Existing','test-only','complete')");
    db.exec(migrations[1]);
    seed(db);
    db.exec("INSERT INTO payments VALUES ('p','ORD-001','a',50000,'2026-10-27T10:00:00Z')");
    db.exec("INSERT INTO audit_events(id,order_id,author_id,field,new_value) VALUES ('h','ORD-001','a','created','ORD-001')");
    db.close(); db = new DatabaseSync(file);
    assert.equal(db.prepare("SELECT name FROM users WHERE id='old'").get().name, 'Existing');
    const order = db.prepare('SELECT orders.id,clients.name,equipment.serial,price_minor FROM orders JOIN equipment ON equipment.id=equipment_id JOIN clients ON clients.id=client_id').get();
    assert.deepEqual({ ...order }, { id: 'ORD-001', name: 'Cliente', serial: 'SERIE', price_minor: 150000 });
    assert.equal(db.prepare('SELECT amount_minor FROM payments').get().amount_minor, 50000);
    assert.equal(db.prepare('SELECT author_id FROM audit_events').get().author_id, 'a');
  } finally { db?.close(); rmSync(dir, { recursive: true, force: true }); }
});

test('workshop constraints preserve references, exact money and immutable history', () => {
  const db = new DatabaseSync(':memory:');
  try {
    db.exec('PRAGMA foreign_keys=ON'); migrations.forEach(sql => db.exec(sql)); seed(db);
    assert.throws(() => db.exec("INSERT INTO equipment(id,client_id,item) VALUES ('bad','missing','Laptop')"));
    assert.throws(() => db.exec("UPDATE orders SET price_minor=1.5"));
    assert.throws(() => db.exec("UPDATE orders SET price_minor=-1"));
    assert.throws(() => db.exec("INSERT INTO payments VALUES ('p','missing','a',1,'today')"));
    assert.throws(() => db.exec("INSERT INTO payments VALUES ('p','ORD-001','a',0,'today')"));
    db.exec("INSERT INTO audit_events(id,order_id,author_id,field,new_value) VALUES ('h','ORD-001','a','work','Diagnóstico')");
    assert.throws(() => db.exec("INSERT INTO audit_events(id,order_id,author_id,field) VALUES ('secret','ORD-001','a','password')"));
    assert.throws(() => db.exec("UPDATE audit_events SET new_value='changed'"));
    assert.throws(() => db.exec('DELETE FROM audit_events'));
    db.exec("INSERT INTO users(id,name,password_hash,access_level) VALUES ('b','Admin 2','test-only','complete')");
    db.exec("UPDATE users SET active=0 WHERE id='a'");
    assert.equal(db.prepare('SELECT author_id FROM audit_events').get().author_id, 'a');
    assert.throws(() => db.exec("DELETE FROM users WHERE id='a'"));
    assert.throws(() => db.exec('DELETE FROM orders'));
  } finally { db.close(); }
});

test('a failed audit rolls back the associated order transaction', () => {
  const db = new DatabaseSync(':memory:');
  try {
    db.exec('PRAGMA foreign_keys=ON'); migrations.forEach(sql => db.exec(sql)); seed(db);
    db.exec('BEGIN');
    db.exec("UPDATE orders SET work='Changed' WHERE id='ORD-001'");
    assert.throws(() => db.exec("INSERT INTO audit_events(id,order_id,author_id,field) VALUES ('h','ORD-001','missing','work')"));
    db.exec('ROLLBACK');
    assert.equal(db.prepare('SELECT work FROM orders').get().work, '');
  } finally { db.close(); }
});
