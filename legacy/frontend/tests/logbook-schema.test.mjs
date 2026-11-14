import assert from 'node:assert/strict';
import { test } from 'node:test';
import { DatabaseSync } from 'node:sqlite';
import { readFileSync, mkdtempSync, rmSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join } from 'node:path';

test('logbook migration backfills real history and preserves authors and events on database reopen', () => {
  const dir=mkdtempSync(join(tmpdir(),'hrl-logbook-'));
  let db;
  try {
    const file=join(dir,'logbook.db'); db=new DatabaseSync(file);
    for(const name of ['001_access','002_workshop','003_authentication','004_order_records','005_order_visibility','006_repair_records']) db.exec(readFileSync(new URL(`../src-tauri/migrations/${name}.sql`,import.meta.url),'utf8'));
    db.exec("INSERT INTO users(id,name,password_hash,access_level) VALUES ('a','Admin','test-only','complete'); INSERT INTO clients(id,name) VALUES ('c','Cliente'); INSERT INTO equipment(id,client_id,item) VALUES ('e','c','Laptop'); INSERT INTO orders(id,equipment_id,created_by,received,status) VALUES ('ORD-1','e','a','2026-10-28','Recibido'); INSERT INTO audit_events(id,order_id,author_id,field,new_value) VALUES ('created','ORD-1','a','created','ORD-1'); INSERT INTO payments VALUES ('p','ORD-1','a',10000,''); INSERT INTO repair_changes(order_id,author_id,operation) VALUES ('ORD-1','a','payments.add');");
    db.exec(readFileSync(new URL('../src-tauri/migrations/007_logbook.sql',import.meta.url),'utf8'));
    assert.equal(db.prepare('SELECT count(*) AS n FROM logbook_events').get().n,2);
    assert.equal(db.prepare("SELECT occurred_at FROM logbook_events WHERE source='payment'").get().occurred_at,null);
    db.exec("INSERT INTO payments VALUES ('p2','ORD-1','a',5000,'2026-10-28T12:00'); UPDATE users SET name='Nuevo' WHERE id='a'");
    assert.equal(db.prepare("SELECT author_name FROM logbook_events WHERE source_id='p2'").get().author_name,'Admin');
    assert.throws(()=>db.exec("DELETE FROM logbook_events"));
    db.close(); db=new DatabaseSync(file);
    assert.equal(db.prepare('SELECT count(*) AS n FROM logbook_events').get().n,3);
    assert.match(db.prepare("SELECT occurred_at FROM logbook_events WHERE source_id='p2'").get().occurred_at,/Z$/);
  } finally { db?.close(); rmSync(dir,{recursive:true,force:true}); }
});
