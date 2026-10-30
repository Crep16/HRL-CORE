import type { FinancialFilters, FinancialOrder } from '../types/finances';
import { companyDateTime } from './companyDate.ts';
export const financialClasses:Record<string,string>={all:'Todas',incomplete:'Incompletas',completed:'Completadas',paid_uncollected:'Pagadas sin entregar',delivered_unpaid:'Entregadas con saldo pendiente',pending:'Sin liquidar ni entregar'};
export function calendarDate(value:string|null,timezone:string):string {
 if(!value)return '';
 if(/^\d{2}\/\d{2}\/\d{4}$/.test(value)){const [day,month,year]=value.split('/');return `${year}-${month}-${day}`;}
 if(/(?:Z|[+-]\d{2}:\d{2})$/.test(value)){const date=new Date(value);return Number.isNaN(date.getTime())?'':companyDateTime(timezone,date).slice(0,10);}
 return /^\d{4}-\d{2}-\d{2}/.test(value)?value.slice(0,10):'';
}
export function inDateRange(value:string|null,filters:Pick<FinancialFilters,'from'|'to'>,timezone:string):boolean{
 if(!filters.from&&!filters.to)return true;
 const day=calendarDate(value,timezone);return !!day&&(!filters.from||day>=filters.from)&&(!filters.to||day<=filters.to);
}
export function filterFinancialOrders(orders:FinancialOrder[],filters:FinancialFilters,timezone:string){
 const query=filters.query.trim().normalize('NFD').replace(/[\u0300-\u036f]/g,'').toLowerCase();
 return orders.filter(order=>{
  if(filters.scope==='active'&&order.excluded||filters.scope==='excluded'&&!order.excluded)return false;
  if(!`${order.id} ${order.client}`.normalize('NFD').replace(/[\u0300-\u036f]/g,'').toLowerCase().includes(query))return false;
  if(filters.classification==='incomplete'&&order.classification==='completed')return false;
  if(!['all','incomplete'].includes(filters.classification)&&filters.classification!==order.classification)return false;
  if(filters.dateBasis==='payment')return !filters.from&&!filters.to||order.payments.some(payment=>inDateRange(payment.occurredAt,filters,timezone));
  return inDateRange(filters.dateBasis==='delivered'?order.deliveredAt:order.received,filters,timezone);
 });
}
export function financialTotals(orders:FinancialOrder[],filters:FinancialFilters,timezone:string){
 const totals=new Map<string,{currency:string;total:number;collected:number;received:number;adjusted:number;refunded:number;credit:number;balance:number;costs:number;profit:number;completedProfit:number;excludedCollected:number}>();
 for(const order of orders){
  let group=totals.get(order.currency);if(!group){group={currency:order.currency,total:0,collected:0,received:0,adjusted:0,refunded:0,credit:0,balance:0,costs:0,profit:0,completedProfit:0,excludedCollected:0};totals.set(order.currency,group);}
  const collected=filters.dateBasis==='payment'?order.payments.filter(payment=>inDateRange(payment.occurredAt,filters,timezone)).reduce((sum,payment)=>sum+payment.amountMinor,0):order.collectedMinor;
  if(order.excluded){group.excludedCollected+=collected;continue;}
  const movements=order.payments.filter(payment=>filters.dateBasis!=='payment'||inDateRange(payment.occurredAt,filters,timezone));
  group.received+=filters.dateBasis==='payment'?movements.filter(p=>!p.kind||p.kind==='receipt').reduce((sum,p)=>sum+p.amountMinor,0):(order.receivedMinor??order.collectedMinor);
  group.adjusted+=filters.dateBasis==='payment'?movements.filter(p=>p.kind==='adjustment').reduce((sum,p)=>sum+p.amountMinor,0):(order.adjustedMinor??0);
  group.refunded+=filters.dateBasis==='payment'?-movements.filter(p=>p.kind==='refund').reduce((sum,p)=>sum+p.amountMinor,0):(order.refundedMinor??0);
  group.credit+=order.creditMinor??0;
  group.total+=order.totalMinor;group.collected+=collected;group.balance+=order.balanceMinor;group.costs+=order.costsMinor;group.profit+=order.profitMinor;if(order.classification==='completed')group.completedProfit+=order.profitMinor;
 }
 return [...totals.values()];
}
export function utcDayBoundary(day:string,timezone:string,nextDay=false):string{
 if(!day)return '';
 const base=new Date(`${day}T00:00:00.000Z`);if(Number.isNaN(base.getTime())||base.toISOString().slice(0,10)!==day)throw new Error('Revisa el rango de fechas.');
 if(nextDay)base.setUTCDate(base.getUTCDate()+1);
 const desired=base.getTime();let candidate=desired;
 for(let i=0;i<4;i++){const local=companyDateTime(timezone,new Date(candidate));const represented=Date.parse(`${local}:00.000Z`);const correction=desired-represented;candidate+=correction;if(!correction)break;}
 return new Date(candidate).toISOString();
}
