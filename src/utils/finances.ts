import { companyDateTime } from './companyDate.ts';
export function calendarDate(value:string|null,timezone:string):string {
 if(!value)return '';
 if(/^\d{2}\/\d{2}\/\d{4}$/.test(value)){const [day,month,year]=value.split('/');return `${year}-${month}-${day}`;}
 if(/(?:Z|[+-]\d{2}:\d{2})$/.test(value)){const date=new Date(value);return Number.isNaN(date.getTime())?'':companyDateTime(timezone,date).slice(0,10);}
 return /^\d{4}-\d{2}-\d{2}/.test(value)?value.slice(0,10):'';
}
export function utcDayBoundary(day:string,timezone:string,nextDay=false):string{
 if(!day)return '';
 const base=new Date(`${day}T00:00:00.000Z`);if(Number.isNaN(base.getTime())||base.toISOString().slice(0,10)!==day)throw new Error('Revisa el rango de fechas.');
 if(nextDay)base.setUTCDate(base.getUTCDate()+1);
 const desired=base.getTime();let candidate=desired;
 for(let i=0;i<4;i++){const local=companyDateTime(timezone,new Date(candidate));const represented=Date.parse(`${local}:00.000Z`);const correction=desired-represented;candidate+=correction;if(!correction)break;}
 return new Date(candidate).toISOString();
}
