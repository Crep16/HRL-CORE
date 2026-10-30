import assert from 'node:assert/strict';
import { test } from 'node:test';
import { fileURLToPath } from 'node:url';
import { createServer } from 'vite';
import vue from '@vitejs/plugin-vue';

test('temporary authorization preserves identity, is scoped in the interface and is removed before ending the backend lease',async()=>{
 const previousWindow=globalThis.window,previousTauri=globalThis.isTauri;
 let userId='recipient',endRelease,holdEnd=false,grantRelease,holdGrant=false;
 const commands=[];
 globalThis.isTauri=true;
 globalThis.window={__TAURI_INTERNALS__:{async invoke(command,args){
  commands.push(command);
  if(command==='access_status')return {setupComplete:true,user:{id:userId,name:'Recipient',accessLevel:'limited'}};
  if(command==='access_permissions')return ['clients.view','repairs.view','notes.add'];
  if(command==='grant_temporary_access'){
   const result={id:'grant',recipientId:'recipient',approverId:'approver',approverName:'Superior',scope:args.input.scope,recordId:args.input.recordId,operations:['repairs.view','repairs.edit']};
   if(holdGrant)await new Promise(resolve=>grantRelease=resolve);
   return result;
  }
  if(command==='end_temporary_access'){if(holdEnd)await new Promise(resolve=>endRelease=resolve);return;}
  throw new Error(command);
 }}};
 const server=await createServer({root:fileURLToPath(new URL('..',import.meta.url)),configFile:false,plugins:[vue()],server:{middlewareMode:true,hmr:false},appType:'custom',logLevel:'error'});
 try{
  const {useAccess}=await server.ssrLoadModule('/src/composables/useAccess.ts');const access=useAccess();await access.refresh();access.setContext('repairs','A');
  assert.equal(access.can('repairs.edit'),false);
  const waiting=access.requestAccess('repairs.edit','repairs','A');assert.equal(access.accessRequest.value.operation,'repairs.edit');
  await access.approveTemporary('Superior','test-only-password','Trabajo autorizado','A');assert.equal(await waiting,true);
  assert.equal(access.user.value.id,'recipient');assert.equal(access.user.value.accessLevel,'limited');
  assert.equal(access.can('repairs.edit'),true);assert.equal(access.baseCan('repairs.edit'),false);
  assert.equal(access.canFor('repairs.edit','repairs','B'),false);assert.equal(access.canFor('company.edit','company',null),false);
  access.setContext('clients');assert.equal(access.can('repairs.edit'),false);access.setContext('repairs','A');
  holdEnd=true;const ending=access.endTemporary();assert.equal(access.can('repairs.edit'),false);endRelease();await ending;holdEnd=false;
  const canceled=access.requestAccess('repairs.edit','repairs','A');access.resolveRequest(false);assert.equal(await canceled,false);
  holdGrant=true;const pending=access.requestAccess('repairs.edit','repairs','A');
  const stale=access.approveTemporary('Superior','test-only-password','Trabajo autorizado','A');
  userId='different-recipient';await access.refresh();grantRelease();await assert.rejects(stale,/sesión cambió/);access.resolveRequest(false);assert.equal(await pending,false);
  assert.equal(access.can('repairs.edit'),false);assert.ok(!commands.includes('login'));
 }finally{await server.close();if(previousWindow===undefined)delete globalThis.window;else globalThis.window=previousWindow;if(previousTauri===undefined)delete globalThis.isTauri;else globalThis.isTauri=previousTauri;}
});
