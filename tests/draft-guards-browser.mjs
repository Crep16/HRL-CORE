import assert from 'node:assert/strict';
import {mkdir, writeFile} from 'node:fs/promises';

const pages = await (await fetch('http://127.0.0.1:9224/json/list')).json();
const page = pages.find(page => page.type === 'page' && (page.url === 'about:blank' || page.url.startsWith('http://127.0.0.1:1421/tests/')));
assert.ok(page, 'Falta el navegador aislado de pruebas en el puerto 9224.');
const socket = new WebSocket(page.webSocketDebuggerUrl);
await new Promise((resolve, reject) => {socket.addEventListener('open', resolve, {once:true}); socket.addEventListener('error', reject, {once:true});});
let sequence = 0;
const pending = new Map(), errors = [];
socket.addEventListener('message', event => {
  const message = JSON.parse(event.data);
  if(message.method === 'Page.javascriptDialogOpening' && message.params.type === 'beforeunload') {
    void send('Page.handleJavaScriptDialog',{accept:true}).catch(error=>errors.push(error.message));
  }
  if(message.method === 'Runtime.exceptionThrown') errors.push(message.params.exceptionDetails.exception?.description ?? message.params.exceptionDetails.text);
  const request = pending.get(message.id);
  if(!request) return;
  clearTimeout(request.timer); pending.delete(message.id);
  if(message.error) request.reject(new Error(message.error.message)); else request.resolve(message.result);
});
function send(method, params = {}) {
  return new Promise((resolve, reject) => {
    const id = ++sequence;
    const timer = setTimeout(() => {pending.delete(id); reject(new Error(`Tiempo agotado: ${method}`));}, 10000);
    pending.set(id, {resolve, reject, timer}); socket.send(JSON.stringify({id, method, params}));
  });
}
async function evaluate(expression) {
  const result = await send('Runtime.evaluate', {expression, returnByValue:true, awaitPromise:true});
  if(result.exceptionDetails) throw new Error(result.exceptionDetails.exception?.description ?? result.exceptionDetails.text);
  return result.result.value;
}
async function waitFor(expression) {
  for(let i=0;i<80;i++) {if(await evaluate(expression)) return; await new Promise(resolve=>setTimeout(resolve,100));}
  throw new Error(`No se cumplió: ${expression}`);
}
async function button(text, root='document') {
  assert.ok(await evaluate(`(() => {const b=[...${root}.querySelectorAll('button')].find(b=>b.textContent.trim()===${JSON.stringify(text)});if(!b||b.disabled)return false;b.focus();b.click();return true;})()`),text);
}
async function fill(selector,value) {
  await evaluate(`(() => {const el=document.querySelector(${JSON.stringify(selector)});el.value=${JSON.stringify(value)};el.dispatchEvent(new Event('input',{bubbles:true}));})()`);
}
async function view(name) {
  await send('Page.navigate',{url:`http://127.0.0.1:1421/tests/ui-restored.html?view=${name}`});
  await waitFor('document.documentElement.dataset.testReady === "true"');
}
const open='document.querySelector(".draft-leave-dialog[open]")';
try {
  await mkdir('artifacts/draft-guards',{recursive:true});
  await send('Page.handleJavaScriptDialog',{accept:true}).catch(()=>{});
  await send('Page.enable'); await send('Runtime.enable');
  await send('Emulation.setDeviceMetricsOverride',{width:1280,height:900,deviceScaleFactor:1,mobile:false});
  await view('company');
  await fill('#company-name','Borrador de empresa');
  await button('Usuarios'); await waitFor(open);
  assert.equal(await evaluate('document.activeElement.textContent.trim()'),'Seguir editando');
  await send('Input.dispatchKeyEvent',{type:'keyDown',key:'Escape',code:'Escape',windowsVirtualKeyCode:27});
  await send('Input.dispatchKeyEvent',{type:'keyUp',key:'Escape',code:'Escape',windowsVirtualKeyCode:27});
  await waitFor(`!${open}`);
  assert.equal(await evaluate('document.querySelector("#company-name").value'),'Borrador de empresa');
  assert.equal(await evaluate('document.activeElement.textContent.trim()'),'Usuarios');
  await button('Usuarios'); await waitFor(open);
  await evaluate('window.draftGuardTest.failSave=true');
  await button('Guardar y salir',open); await waitFor(`${open}?.querySelector('[role="alert"]')`);
  assert.equal(await evaluate('document.querySelector("#company-name").value'),'Borrador de empresa');
  await evaluate('window.draftGuardTest.failSave=false');
  await button('Guardar y salir',open); await waitFor('document.querySelector(".users-heading")');
  assert.equal(await evaluate('window.draftGuardTest.saveCalls'),2);
  await button('Empresa'); await waitFor('document.querySelector("#company-name")');
  assert.equal(await evaluate('document.querySelector("#company-name").value'),'Borrador de empresa');
  await fill('#company-name','Descartar este cambio'); await button('Usuarios'); await waitFor(open);
  await button('Descartar y salir',open); await waitFor('document.querySelector(".users-heading")');
  await button('Empresa'); await waitFor('document.querySelector("#company-name")');
  assert.equal(await evaluate('document.querySelector("#company-name").value'),'Borrador de empresa');
  await fill('#company-name','Prueba de cierre de sesión'); await button('Cerrar sesión'); await waitFor(open);
  assert.equal(await evaluate('window.draftGuardTest.logoutCalls'),0);
  await button('Seguir editando',open); await waitFor(`!${open}`);
  assert.equal(await evaluate('window.draftGuardTest.logoutCalls'),0);
  await button('Cerrar sesión'); await waitFor(open); await button('Descartar y salir',open);
  await waitFor('window.draftGuardTest.logoutCalls===1');

  await view('clients'); await waitFor('document.querySelector(".sheet .who")');
  await button('Editar'); await fill('#cliente','Borrador de recepción');
  await button('Empresa'); await waitFor(open); await button('Seguir editando',open); await waitFor(`!${open}`);
  assert.equal(await evaluate('document.querySelector("#cliente").value'),'Borrador de recepción');
  await button('Empresa'); await waitFor(open); await button('Descartar y salir',open);
  await waitFor('document.querySelector("#company-name")');

  await button('Clientes'); await waitFor('document.querySelector(".sheet .who")');
  await button('Editar'); await fill('#trabajo','Trabajo guardado al salir');
  await button('Empresa'); await waitFor(open); await button('Guardar y salir',open);
  await waitFor('document.querySelector("#company-name")');
  await button('Clientes'); await waitFor('document.querySelector("#trabajo")');
  assert.equal(await evaluate('document.querySelector("#trabajo").value'),'Trabajo guardado al salir');
  await button('Empresa'); await waitFor('document.querySelector("#company-name")');

  for(const width of [360,540,768,1280]) {
    await send('Emulation.setDeviceMetricsOverride',{width,height:900,deviceScaleFactor:1,mobile:false});
    await fill('#company-name',`Ventana de ${width}`); await button('Cerrar sesión'); await waitFor(open);
    const layout=await evaluate(`(() => {const d=${open},r=d.getBoundingClientRect();return {left:r.left,right:r.right,width:innerWidth,scroll:d.scrollWidth,client:d.clientWidth,buttons:[...d.querySelectorAll('button')].map(b=>({height:b.getBoundingClientRect().height,width:b.getBoundingClientRect().width})),modal:d.matches(':modal')};})()`);
    assert.ok(layout.modal); assert.ok(layout.left>=0 && layout.right<=layout.width);
    assert.ok(layout.scroll<=layout.client+1);
    assert.ok(layout.buttons.every(b=>b.height>=44 && b.width>=44));
    for(let step=0;step<5;step++) {
      await send('Input.dispatchKeyEvent',{type:'keyDown',key:'Tab',code:'Tab',windowsVirtualKeyCode:9});
      await send('Input.dispatchKeyEvent',{type:'keyUp',key:'Tab',code:'Tab',windowsVirtualKeyCode:9});
      assert.ok(await evaluate(`${open}.contains(document.activeElement)`));
    }
    // Un diálogo modal conserva foco incluso con el contenedor inerte durante el cierre nativo.
    await evaluate(`document.querySelector('.app').inert=true; ${open}.querySelector('button').focus()`);
    assert.equal(await evaluate('document.activeElement.textContent.trim()'),'Seguir editando');
    await evaluate("document.querySelector('.app').inert=false");
    const shot=await send('Page.captureScreenshot',{format:'png'});
    await writeFile(`artifacts/draft-guards/company-${width}.png`,Buffer.from(shot.data,'base64'));
    await button('Seguir editando',open); await waitFor(`!${open}`);
  }
  assert.deepEqual(errors,[]);
  await send('Page.navigate',{url:'about:blank'});
  console.log('Guardas: navegación, guardado rechazado/confirmado, descarte, Escape/foco, cierre de sesión y 4 anchos: OK.');
} finally {socket.close();}
