import assert from 'node:assert/strict';
import {mkdir,writeFile} from 'node:fs/promises';
const targets=await (await fetch('http://127.0.0.1:9335/json/list')).json();
const target=targets.find(v=>v.type==='page');assert.ok(target);
const socket=new WebSocket(target.webSocketDebuggerUrl);await new Promise((resolve,reject)=>{socket.addEventListener('open',resolve,{once:true});socket.addEventListener('error',reject,{once:true});});
let sequence=0;const requests=new Map(),errors=[];
socket.addEventListener('message',event=>{const m=JSON.parse(event.data);if(m.method==='Runtime.exceptionThrown')errors.push(m.params.exceptionDetails.exception?.description??m.params.exceptionDetails.text);const p=requests.get(m.id);if(!p)return;requests.delete(m.id);clearTimeout(p.timer);if(m.error)p.reject(new Error(m.error.message));else p.resolve(m.result);});
function send(method,params={}){return new Promise((resolve,reject)=>{const id=++sequence;const timer=setTimeout(()=>{requests.delete(id);reject(new Error(`Tiempo agotado: ${method}`));},10000);requests.set(id,{resolve,reject,timer});socket.send(JSON.stringify({id,method,params}));});}
async function evaluate(expression){const r=await send('Runtime.evaluate',{expression,returnByValue:true,awaitPromise:true,userGesture:true});if(r.exceptionDetails)throw new Error(r.exceptionDetails.exception?.description??r.exceptionDetails.text);return r.result.value;}
async function wait(expression){for(let i=0;i<100;i++){if(await evaluate(expression))return;await new Promise(r=>setTimeout(r,100));}throw new Error(`No se cumplió: ${expression}`);}
const button=text=>`[...document.querySelectorAll('button')].find(b=>b.getClientRects().length>0 && b.textContent.trim()===${JSON.stringify(text)})`;
async function click(text){await wait(`${button(text)} && !${button(text)}.matches(':disabled')`);await evaluate(`${button(text)}.focus();${button(text)}.click()`);}
async function field(id,value){await wait(`document.getElementById(${JSON.stringify(id)}) && !document.getElementById(${JSON.stringify(id)}).matches(':disabled')`);await evaluate(`(()=>{const input=document.getElementById(${JSON.stringify(id)});input.value=${JSON.stringify(value)};input.dispatchEvent(new Event('input',{bubbles:true}));})()`);}
async function view(page){await send('Page.navigate',{url:`http://127.0.0.1:1421/tests/ui-restored.html?view=${page}`});await wait('document.documentElement.dataset.testReady === "true"');}
await mkdir('artifacts/portal',{recursive:true});
async function capture(name){const screenshot=await send('Page.captureScreenshot',{format:'png',captureBeyondViewport:false});await writeFile(`artifacts/portal/${name}.png`,Buffer.from(screenshot.data,'base64'));}
try{
  await send('Page.enable');await send('Runtime.enable');await send('Emulation.setDeviceMetricsOverride',{width:1440,height:1000,deviceScaleFactor:1,mobile:false});
  await view('repairs');await wait('document.querySelector(".portal-publication")?.textContent.includes("K7QM-2XPA-9RTD")');
  await click('Publicar cambios');await wait('window.portalTest.actions.includes("publish")');
  await click('Retirar seguimiento');await wait('document.querySelector("dialog[open]")!==null');await click('Cancelar');
  await wait('document.activeElement?.textContent.trim()==="Retirar seguimiento"');
  assert.equal(await evaluate('window.portalTest.enabled'),true);
  await click('Retirar seguimiento');await wait('document.querySelector("dialog[open]")!==null');await click('Confirmar');await wait('window.portalTest.enabled===false');
  await click('Cambiar código');await wait('document.querySelector("dialog[open]")!==null');await click('Confirmar');await wait('document.querySelector(".portal-publication")?.textContent.includes("ABCD-EFGH-JKMN")');
  await evaluate('window.portalTest.state="conflict"');await click('Actualizar estado');await click('Reconciliar y publicar');await wait('document.querySelector("dialog[open]")!==null');await click('Confirmar');await wait('window.portalTest.actions.includes("reconcile")');
  await evaluate('window.portalTest.state="confirmed"');await click('Actualizar estado');await wait('document.querySelector(".portal-publication")?.textContent.includes("Publicado")');
  await evaluate('document.querySelector(".portal-publication").scrollIntoView({block:"center"})');await capture('reparacion');
  await send('Emulation.setDeviceMetricsOverride',{width:390,height:844,deviceScaleFactor:1,mobile:false});
  await evaluate('document.querySelector(".portal-publication").scrollIntoView({block:"center"})');
  assert.equal(await evaluate('document.documentElement.scrollWidth<=document.documentElement.clientWidth'),true);await capture('reparacion-movil');
  await send('Emulation.setDeviceMetricsOverride',{width:1440,height:1000,deviceScaleFactor:1,mobile:false});await view('company');
  await wait('document.querySelector(".portal-settings")!==null');await evaluate('document.querySelector(".portal-settings").open=true');
  await wait('document.getElementById("portal-project")?.value==="hrl-web"');
  await wait('document.querySelector(".company-preview .note-tracking img")?.naturalWidth>0');
  assert.ok(!await evaluate('document.querySelector("#app-notices")?.textContent.includes("No se pudo preparar el QR")'));
  await field('portal-url','https://seguimiento.example.invalid');await click('Guardar configuración');
  await wait('document.querySelector("#app-notices")?.textContent.includes("Configuración guardada")');
  await wait('document.querySelector(".company-preview .note-tracking")?.textContent.includes("https://seguimiento.example.invalid")');
  await field('publisher-email','publisher@example.invalid');await field('publisher-password','test-only-password');await click('Conectar cuenta');
  await wait('document.querySelector(".portal-settings summary").textContent.includes("Cuenta conectada")');assert.equal(await evaluate('document.getElementById("publisher-password").value'),'');
  await click('Reanudar envío');await wait('document.querySelector(".portal-settings summary").textContent.includes("envío activo")');
  await click('Pausar envío');await wait('document.querySelector(".portal-settings summary").textContent.includes("envío pausado")');
  await click('Actualizar estado');
  await capture('configuracion');
  await click('Desconectar cuenta');await wait('document.querySelector(".portal-settings summary").textContent.includes("Sin cuenta")');
  await wait('document.querySelector(".company-preview .note-tracking img")?.naturalWidth>0');
  assert.ok(await evaluate('document.querySelector(".company-preview .note-tracking").textContent.includes("K7QM-2XPA-9RTD")'));
  await evaluate('document.querySelector(".portal-settings").open=false;document.querySelector(".company-preview .note-tracking").scrollIntoView({block:"center"})');await capture('comprobante');
  await send('Emulation.setDeviceMetricsOverride',{width:390,height:844,deviceScaleFactor:1,mobile:false});
  assert.equal(await evaluate('document.documentElement.scrollWidth<=document.documentElement.clientWidth'),true);
  assert.deepEqual(errors,[]);console.log('PASS: publicación, cancelación, retirada, rotación, confirmación, configuración, conexión, renovación de estado, pausa, desconexión, QR y ancho móvil; sin excepciones de JavaScript.');
}finally{socket.close();}
