import assert from 'node:assert/strict';
import { test } from 'node:test';
import { DatabaseSync } from 'node:sqlite';
import { readFileSync } from 'node:fs';

test('money migration preserves historical receipts and grants previously authorized collection without granting refunds',()=>{
 const db=new DatabaseSync(':memory:');
 try{
  db.exec('PRAGMA foreign_keys=ON');
  const files=[...readFileSync(new URL('../src-tauri/src/lib.rs',import.meta.url),'utf8').matchAll(/include_str!\("\.\.\/migrations\/([^"\n]+\.sql)"\)/g)].map(match=>match[1]);
  for(const file of files.filter(file=>!file.startsWith('010_')))db.exec(readFileSync(new URL(`../src-tauri/migrations/${file}`,import.meta.url),'utf8'));
  db.exec("INSERT INTO users(id,name,password_hash,access_level,login_name) VALUES ('admin','Admin','test-hash','complete','admin'),('limited','Limited','test-hash','limited','limited'),('custom','Custom','test-hash','custom','custom'); INSERT INTO user_permissions VALUES ('custom','repairs','edit'); INSERT INTO clients(id,name) VALUES ('c','Prueba'); INSERT INTO equipment(id,client_id,item) VALUES ('e','c','Laptop'); INSERT INTO orders(id,equipment_id,created_by,received,status,price_minor) VALUES ('o','e','admin','2026-10-28','Recibido',75000); INSERT INTO payments VALUES ('p','o','admin',75000,'');");
  db.exec(readFileSync(new URL('../src-tauri/migrations/010_payment_adjustments.sql',import.meta.url),'utf8'));
  const grants=db.prepare("SELECT section,action FROM user_permissions WHERE user_id='custom' ORDER BY section,action").all();
  assert.deepEqual(grants.map(g=>`${g.section}:${g.action}`),['payments:add','repairs:edit']);
  assert.equal(db.prepare("SELECT count(*) AS n FROM user_permissions WHERE action='refund'").get().n,0);
  const original=db.prepare('SELECT * FROM payments').get();assert.equal(original.amount_minor,75000);assert.equal(original.method,null);assert.equal(original.occurred_at,'');
  const totals=db.prepare('SELECT * FROM order_payment_totals').get();assert.equal(totals.net_minor,75000);assert.equal(totals.adjusted_minor,0);assert.equal(totals.refunded_minor,0);
  assert.throws(()=>db.exec("UPDATE payments SET amount_minor=50000"));assert.throws(()=>db.exec('DELETE FROM payments'));
  assert.deepEqual(db.prepare('PRAGMA foreign_key_check').all(),[]);
 }finally{db.close();}
});
