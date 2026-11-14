import assert from 'node:assert/strict';
import { test } from 'node:test';
import { filterFinancialOrders,financialTotals,calendarDate,utcDayBoundary } from '../src/utils/finances.ts';
const filters={query:'',classification:'all',dateBasis:'received',from:'',to:'',scope:'active'};
const make=(values={})=>({id:'ORD-1',client:'Héctor',currency:'MXN',classification:'pending',excluded:false,totalMinor:150000,collectedMinor:50000,balanceMinor:100000,costsMinor:60000,profitMinor:90000,received:'28/10/2026',deliveredAt:null,payments:[{amountMinor:50000,occurredAt:'2026-10-27T12:00'}],...values});

test('refunds and corrections have separate period totals while credit remains cumulative',()=>{
 const order=make({receivedMinor:75000,adjustedMinor:-10000,refundedMinor:5000,collectedMinor:60000,totalMinor:55000,creditMinor:5000,balanceMinor:0,payments:[
  {kind:'receipt',amountMinor:75000,occurredAt:'2026-10-27T12:00'},
  {kind:'adjustment',amountMinor:-10000,occurredAt:'2026-10-28T12:00'},
  {kind:'refund',amountMinor:-5000,occurredAt:'2026-10-28T13:00'},
 ]});
 const all=financialTotals([order],filters,'America/Cancun')[0];
 assert.equal(all.received,75000);assert.equal(all.adjusted,-10000);assert.equal(all.refunded,5000);assert.equal(all.collected,60000);assert.equal(all.credit,5000);
 const period=financialTotals([order],{...filters,dateBasis:'payment',from:'2026-10-28',to:'2026-10-28'},'America/Cancun')[0];
 assert.equal(period.received,0);assert.equal(period.adjusted,-10000);assert.equal(period.refunded,5000);assert.equal(period.collected,-15000);assert.equal(period.credit,5000);
});
test('class filters, exclusions and currency groups preserve independent financial totals',()=>{
 const orders=['completed','paid_uncollected','delivered_unpaid','pending'].map((classification,index)=>make({id:`ORD-${index}`,classification}));
 assert.equal(filterFinancialOrders(orders,{...filters,classification:'incomplete'},'America/Cancun').length,3);
 assert.equal(filterFinancialOrders(orders,{...filters,classification:'completed'},'America/Cancun').length,1);
 assert.equal(filterFinancialOrders(orders,{...filters,query:'hector'},'America/Cancun').length,4);
 const totals=financialTotals([...orders,make({currency:'USD'}),make({excluded:true})],filters,'America/Cancun');
 assert.equal(totals[0].total,600000);assert.equal(totals[0].profit,360000);assert.equal(totals[0].completedProfit,90000);assert.equal(totals[0].excludedCollected,50000);assert.equal(totals[1].currency,'USD');
});
test('payment date filters attribute only payments inside the period and never unknown dates',()=>{
 const order=make({collectedMinor:150000,balanceMinor:0,payments:[{amountMinor:50000,occurredAt:'2026-10-27T12:00'},{amountMinor:60000,occurredAt:'2026-10-28T12:00'},{amountMinor:40000,occurredAt:null}]});
 const period={...filters,dateBasis:'payment',from:'2026-10-28',to:'2026-10-28'};
 const selected=filterFinancialOrders([order],period,'America/Cancun');assert.equal(selected.length,1);assert.equal(financialTotals(selected,period,'America/Cancun')[0].collected,60000);assert.equal(financialTotals(selected,period,'America/Cancun')[0].balance,0);
 assert.equal(filterFinancialOrders([order],{...period,from:'2026-10-29',to:'2026-10-29'},'America/Cancun').length,0);
 assert.equal(calendarDate('2026-10-29T03:00:00Z','America/Cancun'),'2026-10-28');
 assert.equal(utcDayBoundary('2026-10-28','America/Cancun'),'2026-10-28T05:00:00.000Z');assert.equal(utcDayBoundary('2026-10-28','America/Cancun',true),'2026-10-29T05:00:00.000Z');
 assert.equal((Date.parse(utcDayBoundary('2026-03-08','America/Tijuana',true))-Date.parse(utcDayBoundary('2026-03-08','America/Tijuana')))/3600000,23);
});
