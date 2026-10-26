import assert from 'node:assert/strict';
import { after, test } from 'node:test';
import { companyErrors, defaultCompany, formatFolio, nextAvailableNumber, useCompany } from '../src/composables/useCompany.ts';
import { companyDateTime } from '../src/utils/companyDate.ts';
import { orderAmount } from '../src/utils/orderAmount.ts';
import { printLayout } from '../src/utils/printLayout.ts';

const previousStorage = Object.getOwnPropertyDescriptor(globalThis, 'localStorage');
const stored = new Map();
let failWrites = false;
Object.defineProperty(globalThis, 'localStorage', {configurable:true, value:{
  getItem(key) {return stored.get(key) ?? null;},
  setItem(key,value) {if(failWrites) throw new Error('Storage full');stored.set(key,value);},
}});
after(() => {
  if(previousStorage) Object.defineProperty(globalThis,'localStorage',previousStorage);
  else delete globalThis.localStorage;
});

test('folios use the configured prefix and skip occupied numbers', () => {
  const settings=defaultCompany();settings.prefix='HRL-';settings.nextNumber=7;
  const number=nextAvailableNumber(settings,[{id:'HRL-007'},{id:'HRL-008'},{id:'ORD-009'}]);
  assert.equal(number,9);
  assert.equal(formatFolio(settings.prefix,number),'HRL-009');
  assert.equal(formatFolio(settings.prefix,1200),'HRL-1200');
  assert.equal(settings.nextNumber,7,'Preview must not reserve a number');
});

test('letter copies are paired on the same physical sheet', () => {
  for(const paper of ['letter']) {
    for(const orientation of ['portrait','landscape']) {
      const settings={...defaultCompany(),paper,orientation,copies:2};
      const layout=printLayout(settings);
      assert.deepEqual(layout.groups,[2]);
      assert.equal(orientation==='portrait',layout.height>layout.width);
      assert.equal(layout.contentWidth,layout.width-16);
      assert.equal(layout.contentHeight,layout.height-16);
    }
  }
  assert.deepEqual(printLayout({...defaultCompany(),copies:5}).groups,[2,2,1]);
  assert.deepEqual(printLayout({...defaultCompany(),paper:'ticket',copies:2}).groups,[1,1]);
});

test('80 and 60 mm tickets retain their physical width and usable margins', () => {
  for(const [paper,width] of [['ticket',80],['ticket60',60]]) {
    const layout=printLayout({...defaultCompany(),paper,copies:2,orientation:'landscape'});
    assert.equal(layout.width,width);
    assert.equal(layout.contentWidth,width-6);
    assert.equal(layout.margin,3);
    assert.deepEqual(layout.groups,[1,1]);
  }
});

test('invalid numbering, copy counts, tax rates and empty clauses cannot be saved', () => {
  for(const [property,value] of [['nextNumber',0],['nextNumber',1.5],['copies',0],['copies',6],['taxRate',-1],['taxRate',101],['laborDays',-1],['partsDays',Infinity],['prefix','invalid prefix'],['email','invalid']]) {
    const settings=defaultCompany();settings[property]=value;
    assert(companyErrors(settings).length>0,`${property}=${value} must be rejected`);
  }
  const settings=defaultCompany();settings.clauses[0].text=' ';
  assert(companyErrors(settings).some(error=>error.includes('cláusula')));
});

test('money parsing preserves Mexican and European decimal separators', () => {
  for(const [value,expected] of [['$1,200.00',1200],['1.200,00',1200],['1200,50',1200.5],['1,200',1200],['€ 2.500,75',2500.75],['$0.00',0],['12.50',12.5]]) assert.equal(orderAmount(value),expected,value);
});

test('repair timestamps use the configured timezone across a date boundary', () => {
  const date=new Date('2026-10-26T04:30:00Z');
  assert.equal(companyDateTime('America/Cancun',date),'2026-10-25T23:30');
  assert.equal(companyDateTime('UTC',date),'2026-10-26T04:30');
});

test('failed or invalid saves keep the last committed configuration', () => {
  const {company,saveCompany,companyStorageError}=useCompany();
  const initial=defaultCompany();initial.name='Empresa de prueba';
  assert.equal(saveCompany(initial),true);
  initial.name='Mutación fuera del formulario';
  assert.equal(company.value.name,'Empresa de prueba','Committed data must be an independent copy');
  const previous=stored.get('hrl.company.v1');
  const pending=defaultCompany();pending.name='Cambio pendiente';
  failWrites=true;
  assert.equal(saveCompany(pending),false);
  assert.equal(company.value.name,'Empresa de prueba');
  assert.equal(stored.get('hrl.company.v1'),previous);
  assert(companyStorageError.value.includes('No se pudo guardar'));
  failWrites=false;pending.nextNumber=0;
  assert.equal(saveCompany(pending),false);
  assert.equal(stored.get('hrl.company.v1'),previous);
});

test('corrupt stored configuration falls back with a visible error', async () => {
  stored.set('hrl.company.v1','{broken');
  const module=await import('../src/composables/useCompany.ts?corruption-test');
  const {company,companyStorageError}=module.useCompany();
  assert.equal(company.value.name,defaultCompany().name);
  assert(companyStorageError.value.includes('No se pudo recuperar'));
});

test('old half-letter preferences migrate to letter without losing company data', async () => {
  stored.set('hrl.company.v1',JSON.stringify({...defaultCompany(),paper:'half',name:'Empresa conservada',phone:'5551234567'}));
  const module=await import('../src/composables/useCompany.ts?paper-migration-test');
  const {company,companyStorageError}=module.useCompany();
  assert.equal(company.value.paper,'letter');
  assert.equal(company.value.name,'Empresa conservada');
  assert.equal(company.value.phone,'5551234567');
  assert.equal(companyStorageError.value,'');
});
