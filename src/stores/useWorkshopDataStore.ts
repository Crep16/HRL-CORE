import { defineStore } from 'pinia';
import { workshopApi } from '../services/api/workshop.ts';
import { useAuthStore } from './useAuthStore.ts';
import { normalizeIpcError } from '../services/api/errors.ts';
export const useWorkshopDataStore = defineStore('workshop-data', () => {
  const auth=useAuthStore();
  async function run<T>(operation:()=>Promise<T>):Promise<T>{
    const version=auth.sessionVersion;
    try{
      const result=await operation();
      if(version!==auth.sessionVersion)throw {code:'SESSION_CHANGED',message:'La sesión cambió durante la operación.'};
      return result;
    }catch(cause){const error=normalizeIpcError(cause);if(error.code==='UNAUTHENTICATED'&&version===auth.sessionVersion)auth.clearSession();throw error;}
  }
  return {
    photos:(...args:Parameters<typeof workshopApi.photos>)=>run(()=>workshopApi.photos(...args)),
    readPhoto:(...args:Parameters<typeof workshopApi.readPhoto>)=>run(()=>workshopApi.readPhoto(...args)),
    addPhoto:(...args:Parameters<typeof workshopApi.addPhoto>)=>run(()=>workshopApi.addPhoto(...args)),
    updatePhoto:(...args:Parameters<typeof workshopApi.updatePhoto>)=>run(()=>workshopApi.updatePhoto(...args)),
    logbook:(...args:Parameters<typeof workshopApi.logbook>)=>run(()=>workshopApi.logbook(...args)),
    finances:(...args:Parameters<typeof workshopApi.finances>)=>run(()=>workshopApi.finances(...args)),
    expense:(...args:Parameters<typeof workshopApi.expense>)=>run(()=>workshopApi.expense(...args)),
    repair:(...args:Parameters<typeof workshopApi.repair>)=>run(()=>workshopApi.repair(...args)),
    saveRepair:(...args:Parameters<typeof workshopApi.saveRepair>)=>run(()=>workshopApi.saveRepair(...args)),
    stages:(...args:Parameters<typeof workshopApi.stages>)=>run(()=>workshopApi.stages(...args)),
    hideStageName:(...args:Parameters<typeof workshopApi.hideStageName>)=>run(()=>workshopApi.hideStageName(...args)),
    summary:(...args:Parameters<typeof workshopApi.summary>)=>run(()=>workshopApi.summary(...args)),
    payments:(...args:Parameters<typeof workshopApi.payments>)=>run(()=>workshopApi.payments(...args)),
    recordPayment:(...args:Parameters<typeof workshopApi.recordPayment>)=>run(()=>workshopApi.recordPayment(...args)),
  };
});
