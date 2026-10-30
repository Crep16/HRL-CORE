import assert from 'node:assert/strict';
import { test } from 'node:test';
import { entryChanges, logbookValue } from '../src/utils/logbook.ts';

test('logbook reads legacy values and formats money without leaking stored JSON syntax', () => {
  const entry = {operation:'orders.edit',field:'price_minor',oldValue:'150000',newValue:'170000',details:null};
  assert.deepEqual(entryChanges(entry),[{field:'price_minor',oldValue:150000,newValue:170000}]);
  assert.equal(logbookValue('price_minor',170000,'MXN','es-MX'),'$1,700.00');
  assert.equal(logbookValue('status','En diagnóstico','MXN','es-MX'),'En diagnóstico');
  assert.equal(logbookValue('other_shop',false,'MXN','es-MX'),'No');
  assert.deepEqual(entryChanges({...entry,field:'client',oldValue:'"Anterior"',newValue:'"Actual"'}),[{field:'client',oldValue:'Anterior',newValue:'Actual'}]);
  assert.deepEqual(entryChanges({...entry,field:'notes',oldValue:'"Texto literal"',newValue:'123'}),[{field:'notes',oldValue:'"Texto literal"',newValue:'123'}]);
  assert.deepEqual(entryChanges({...entry,operation:'payments.add',field:null,newValue:'10000'}),[{field:'amount_minor',oldValue:null,newValue:10000}]);
});

test('new repair details display meaningful labels and preserve before/after notes', () => {
  const details=[{field:'publicNote',oldValue:'Pendiente',newValue:'Terminado'}];
  assert.deepEqual(entryChanges({details}),details);
  assert.equal(logbookValue('warranties',[{name:'Pantalla',days:30}],'MXN','es-MX'),'Pantalla: 30 días');
  assert.equal(logbookValue('parts',[{name:'Pantalla',price:500}],'USD','en-US'),'Pantalla: $500.00');
  assert.equal(logbookValue('events',[{stage:'Diagnóstico',title:'Revisión',date:'2026-10-28T12:00',public:false,detail:'Sin encendido'}],'MXN','es-MX'),'Diagnóstico: Revisión\n2026-10-28 12:00 · Solo interno\nSin encendido');
});
