import assert from 'node:assert/strict';
import { mkdir, writeFile, readFile, unlink } from 'node:fs/promises';
import {resolve} from 'node:path';

const pages = await (await fetch('http://127.0.0.1:9224/json/list')).json();
const targets = pages.filter(page => page.type === 'page' && (page.url === 'about:blank' || page.url.startsWith('http://127.0.0.1:1421/tests/')));
assert.equal(targets.length, 1);
const socket = new WebSocket(targets[0].webSocketDebuggerUrl);
let downloadSocket;
await new Promise((resolve, reject) => {
  socket.addEventListener('open', resolve, { once: true });
  socket.addEventListener('error', reject, { once: true });
});
let sequence = 0;
const pending = new Map();
const errors = [];
socket.addEventListener('message', event => {
  const message = JSON.parse(event.data);
  if (message.method === 'Runtime.exceptionThrown') errors.push(message.params.exceptionDetails.exception?.description ?? message.params.exceptionDetails.text);
  const request = pending.get(message.id);
  if (!request) return;
  clearTimeout(request.timer);
  pending.delete(message.id);
  if (message.error) request.reject(new Error(message.error.message));
  else request.resolve(message.result);
});
function send(method, params = {}) {
  return new Promise((resolve, reject) => {
    const id = ++sequence;
    const timer = setTimeout(() => { pending.delete(id); reject(new Error(`Tiempo agotado en ${method}`)); }, 10000);
    pending.set(id, { resolve, reject, timer });
    socket.send(JSON.stringify({ id, method, params }));
  });
}
async function evaluate(expression) {
  const result = await send('Runtime.evaluate', { expression, returnByValue: true, awaitPromise: true, userGesture: true });
  if (result.exceptionDetails) throw new Error(result.exceptionDetails.exception?.description ?? result.exceptionDetails.text);
  return result.result.value;
}
async function waitFor(expression) {
  for (let attempt = 0; attempt < 80; attempt++) {
    if (await evaluate(expression)) return;
    await new Promise(resolve => setTimeout(resolve, 100));
  }
  throw new Error(`No se cumplió ${expression}`);
}
const settle = () => evaluate('new Promise(resolve => requestAnimationFrame(() => requestAnimationFrame(() => resolve(true))))');
async function view(name) {
  await send('Page.navigate', { url: `http://127.0.0.1:1421/tests/ui-restored.html?view=${name}` });
  await waitFor('document.documentElement?.dataset.testReady === "true"');
  await settle();
}
async function screenshot(name) {
  const result = await send('Page.captureScreenshot', { format: 'png' });
  await writeFile(`artifacts/phase5/${name}.png`, Buffer.from(result.data, 'base64'));
}
async function button(text, root = 'document') {
  assert.ok(await evaluate(`(() => { const button = [...${root}.querySelectorAll('button')].find(button => button.textContent.trim() === ${JSON.stringify(text)}); if (!button || button.disabled) return false; button.focus(); button.click(); return true; })()`), text);
  await settle();
}
try {
  await mkdir('artifacts/phase5', { recursive: true });
  await send('Page.enable');
  await send('Runtime.enable');
  await send('Page.bringToFront');
  await send('Emulation.setDeviceMetricsOverride', { width: 1280, height: 900, deviceScaleFactor: 1, mobile: false });
  await view('repairs&stagesTest');
  await button('Historial (1)');
  await evaluate("document.querySelector('.event-composer').open=true;document.getElementById('repair-stage').focus()");
  await waitFor('!!document.querySelector("#repair-stage-options")');
  await evaluate("[...document.querySelectorAll('#repair-stage-options [role=option]')].find(option=>option.textContent==='test1').click()");
  await button('Quitar nombre del selector');
  await waitFor('!!document.querySelector(".stage-removal-dialog[open]")');
  assert.equal(await evaluate('document.activeElement.textContent.trim()'),'Cancelar');
  await button('Cancelar','document.querySelector(".stage-removal-dialog")');
  assert.equal(await evaluate('document.getElementById("repair-stage").value'),'test1');
  await button('Quitar nombre del selector');
  await send('Emulation.setDeviceMetricsOverride',{width:375,height:812,deviceScaleFactor:1,mobile:false});
  assert.ok(await evaluate('document.querySelector(".stage-removal-dialog").getBoundingClientRect().right<=innerWidth'));
  await screenshot('remove-stage-name-mobile');
  await button('Quitar nombre','document.querySelector(".stage-removal-dialog")');
  await waitFor('!document.querySelector(".stage-removal-dialog[open]")');
  await evaluate("document.getElementById('repair-stage').focus()");
  await waitFor('!!document.querySelector("#repair-stage-options")');
  assert.ok(await evaluate("![...document.querySelectorAll('#repair-stage-options [role=option]')].some(option=>option.textContent==='test1')"));
  assert.ok(await evaluate("document.querySelector('.steps').textContent.includes('test1')"));
  console.log('Nombre personalizado quitado del selector sin modificar el progreso ni el historial.');
  await button('Trabajo y notas');
  await evaluate("document.querySelector('.stage-management').open=true");
  assert.equal(await evaluate('document.querySelectorAll(".stage-management button").length'),1);
  await button('Eliminar etapa','document.querySelector(".stage-management")');
  await waitFor('!!document.querySelector(".stage-removal-dialog[open]")');
  await button('Cancelar','document.querySelector(".stage-removal-dialog")');
  assert.ok(await evaluate("document.querySelector('.steps').textContent.includes('test1')"));
  await button('Eliminar etapa','document.querySelector(".stage-management")');
  await evaluate('window.stageRemovalTest.fail=true');
  await button('Eliminar etapa','document.querySelector(".stage-removal-dialog")');
  await waitFor('!!document.querySelector(".stage-removal-dialog [role=alert]")');
  assert.ok(await evaluate("document.querySelector('.steps').textContent.includes('test1')"));
  await screenshot('remove-pending-stage-error');
  await evaluate('window.stageRemovalTest.fail=false');
  await button('Eliminar etapa','document.querySelector(".stage-removal-dialog")');
  await waitFor('!document.querySelector(".stage-removal-dialog[open]")');
  assert.equal(await evaluate('document.querySelectorAll(".steps li").length'),2);
  assert.ok(await evaluate('document.querySelector(".repair-work h2 small").textContent.includes("50%")'));
  await screenshot('pending-stage-removed-mobile');
  await button('Historial (1)');
  assert.ok(await evaluate('document.querySelector(".tl").textContent.includes("test1")'));
  assert.ok(await evaluate('document.querySelector(".tl").textContent.includes("Etapa eliminada del progreso")'));
  console.log('Etapa pendiente eliminada con historial conservado, porcentaje actualizado y guardado rechazado recuperable.');
  await send('Emulation.setDeviceMetricsOverride', { width: 1280, height: 900, deviceScaleFactor: 1, mobile: false });
  for (const name of ['clients', 'repairs', 'company', 'users', 'logbook']) {
    await view(name);
    const layout = await evaluate('({heading: document.querySelector("h1")?.innerText, width: innerWidth, content: document.documentElement.scrollWidth})');
    const headings = { clients: 'Órdenes', repairs: 'Reparaciones', company: 'Empresa', users: 'Usuarios', logbook: 'Finanzas' };
    assert.equal(layout.heading, headings[name], name);
    assert.ok(layout.content <= layout.width + 1, `${name}: desbordamiento horizontal`);
    if (name === 'repairs') {
      assert.equal(await evaluate('(() => { const amount = document.querySelector(".payment-balance strong"); const range = document.createRange(); range.selectNodeContents(amount); return range.getClientRects().length; })()'), 1);
    }
    await screenshot(name);
    console.log(JSON.stringify({ view: name, ...layout }));
  }

  await view('repairs');
  await waitFor('!!document.querySelector(".payment-actions button") && !document.querySelector(".payment-actions button").disabled');
  await button('+ Registrar abono');
  await evaluate(`(() => { for (const [id,value] of [['money-amount','0'],['money-method','Efectivo']]) {const field=document.getElementById(id);field.value=value;field.dispatchEvent(new Event('input',{bubbles:true}));} })()`);
  await button('Confirmar movimiento');
  assert.equal(await evaluate('window.paymentRequests.length'),0);
  assert.ok(await evaluate('document.querySelector("#app-notices").innerText.includes("mayor que cero")'));
  await evaluate(`(() => { for (const [id, value] of [['money-amount','100.50'],['money-method','Efectivo']]) { const field=document.getElementById(id); field.value=value; field.dispatchEvent(new Event('input',{bubbles:true})); } })()`);
  await evaluate('[...document.querySelectorAll(".payment-form button")].find(button=>button.textContent.trim()==="Confirmar movimiento").focus()');
  await send('Input.dispatchKeyEvent',{type:'keyDown',key:'Enter',code:'Enter',windowsVirtualKeyCode:13,text:'\r'});
  await send('Input.dispatchKeyEvent',{type:'keyUp',key:'Enter',code:'Enter',windowsVirtualKeyCode:13});
  await waitFor('window.paymentRequests.length === 1');
  assert.equal(await evaluate('window.paymentRequests[0].amountMinor'), 10050);
  await waitFor('!document.querySelector(".payment-form")');
  await screenshot('payment-confirmed');
  console.log('Registro de abono confirmado desde el formulario.');
  await button('+ Registrar abono');
  await evaluate(`(() => { for (const [id,value] of [['money-amount','20,05'],['money-method','Transferencia']]) {const field=document.getElementById(id);field.value=value;field.dispatchEvent(new Event('input',{bubbles:true}));} })()`);
  await button('Confirmar movimiento');
  await waitFor('window.paymentRequests.length === 2');
  assert.equal(await evaluate('window.paymentRequests[1].amountMinor'),2005);
  await waitFor('!document.querySelector(".payment-form")');

  await view('company');
  await evaluate("[...document.querySelectorAll('.company-nav button')].find(button => button.querySelector('strong')?.textContent === 'Cláusulas').click()");
  await waitFor('!!document.querySelector("#clause-add")');
  await evaluate('document.querySelector("#clause-add").focus(); document.querySelector("#clause-add").click()');
  await settle();
  assert.equal(await evaluate('document.activeElement.id'), 'clause-title');
  await evaluate(`(() => { for (const [id, value] of [['clause-title','Condición de prueba'],['clause-text','Texto para verificar el borrador.']]) { const field = document.getElementById(id); field.value = value; field.dispatchEvent(new Event('input', { bubbles: true })); } })()`);
  await button('Aplicar cláusula');
  assert.equal(await evaluate('document.activeElement.id'), 'clause-add');
  assert.ok(await evaluate('document.querySelector(".company-clause-list").innerText.includes("Condición de prueba")'));
  await button('Usuarios', 'document.querySelector("nav[aria-label=Secciones]")');
  assert.ok(await evaluate('!!document.querySelector("#company-title")'));
  await waitFor('!!document.querySelector(".draft-leave-dialog[open]")');
  assert.deepEqual(await evaluate('[...document.querySelector(".draft-leave-dialog[open]").querySelectorAll("button")].map(button=>button.textContent.trim())'),['Seguir editando','Descartar y salir','Guardar y salir']);
  await screenshot('company-draft-guard');
  await button('Descartar y salir','document.querySelector(".draft-leave-dialog[open]")');
  await waitFor('!!document.querySelector(".users-heading")');
  console.log('Borrador de cláusula, foco y guarda de navegación aprobados.');

  await view('clients');
  assert.equal(await evaluate('document.querySelectorAll("#equipment-history").length'),1);
  await button('Editar');
  await evaluate(`(() => { const field=document.getElementById('equipment-history');field.focus();field.value='Visita a otro taller; golpe en la esquina y humedad.';field.dispatchEvent(new Event('input',{bubbles:true})); })()`);
  await button('Guardar cambios');
  await waitFor('document.getElementById("equipment-history").readOnly');
  assert.equal(await evaluate('document.getElementById("equipment-history").value'),'Visita a otro taller; golpe en la esquina y humedad.');
  await screenshot('history-unified');
  await evaluate('document.querySelector(".ev-entry").closest("details").querySelector("summary").click()');
  await settle();
  console.log(JSON.stringify(await evaluate('({evidence: !!document.querySelector(".ev-entry"), buttons: [...document.querySelectorAll(".ev-entry button")].map(button => ({text:button.textContent.trim(), disabled:button.disabled}))})')));
  await button('Tomar foto', 'document.querySelector(".ev-entry")');
  await waitFor('!!document.querySelector("dialog.evidence-camera[open]")');
  assert.equal(await evaluate('document.activeElement.textContent.trim()'), 'Encender cámara');
  await screenshot('camera-off');
  await evaluate('document.querySelector("dialog.evidence-camera .ev-close").click()');
  await settle();
  assert.equal(await evaluate('!!document.querySelector("dialog.evidence-camera[open]")'), false);
  assert.ok(await evaluate('document.querySelector("dialog.evidence-gallery[open]").contains(document.activeElement)'));
  await evaluate('document.querySelector("dialog.evidence-gallery .ev-close").click()');
  await settle();
  assert.equal(await evaluate('document.activeElement.textContent.trim()'), 'Abrir galería');
  console.log('Apertura y cierre de cámara con foco restaurado aprobados.');
  await button('Abrir galería');
  await waitFor('!!document.querySelector(".ev-detail textarea")');
  await evaluate(`(() => {const field=document.querySelector('.ev-detail textarea');field.value='Golpe lateral documentado';field.dispatchEvent(new Event('input',{bubbles:true}));})()`);
  await button('Guardar descripción','document.querySelector("dialog.evidence-gallery")');
  await waitFor('document.querySelector(".ev-detail textarea").value === "Golpe lateral documentado" && document.querySelector(".ev-detail-actions .btn-p").disabled');
  await button('Retirar evidencia','document.querySelector("dialog.evidence-gallery")');
  await waitFor('document.querySelector(".ev-navigation").textContent.includes("0 archivos")');
  await button('Ver evidencias retiradas','document.querySelector("dialog.evidence-gallery")');
  await waitFor('!!document.querySelector(".ev-detail-actions button") && document.querySelector(".ev-detail-actions button").textContent === "Restaurar evidencia" && !document.querySelector(".ev-detail-actions button").disabled');
  await button('Restaurar evidencia','document.querySelector("dialog.evidence-gallery")');
  await button('Volver a evidencias activas','document.querySelector("dialog.evidence-gallery")');
  await waitFor('document.querySelector(".ev-detail textarea")?.value === "Golpe lateral documentado"');
  await screenshot('evidence-restored');
  await evaluate(`(() => {const input=document.querySelector('.ev-entry input[type=file]');const transfer=new DataTransfer();transfer.items.add(new File(['%PDF-1.7\\ncontenido de prueba\\n%%EOF\\n'],'informe-prueba.pdf',{type:'application/pdf'}));input.files=transfer.files;input.dispatchEvent(new Event('change',{bubbles:true}));})()`);
  await waitFor(`!!document.querySelector('.ev-stage a[download="informe-prueba.pdf"]')`);
  assert.equal(await evaluate('document.querySelector(".ev-stage h3").textContent'),'Documento PDF');
  const browserInfo=await(await fetch('http://127.0.0.1:9224/json/version')).json();
  downloadSocket=new WebSocket(browserInfo.webSocketDebuggerUrl);
  await new Promise(resolve=>downloadSocket.addEventListener('open',resolve,{once:true}));
  const downloadConfigured=new Promise((resolve,reject)=>downloadSocket.addEventListener('message',event=>{const result=JSON.parse(event.data);if(result.id===1){if(result.error)reject(new Error(result.error.message));else resolve();}}));
  downloadSocket.send(JSON.stringify({id:1,method:'Browser.setDownloadBehavior',params:{behavior:'allow',downloadPath:resolve('artifacts/phase5/downloads'),eventsEnabled:true}}));
  await downloadConfigured;
  await unlink('artifacts/phase5/downloads/informe-prueba.pdf').catch(error=>{if(error.code!=='ENOENT')throw error;});
  const pdfLink=await evaluate('(()=>{const a=document.querySelector(".ev-stage a[download]");a.scrollIntoView({block:"center"});const r=a.getBoundingClientRect();return {x:r.x+r.width/2,y:r.y+r.height/2};})()');
  await send('Input.dispatchMouseEvent',{type:'mousePressed',...pdfLink,button:'left',clickCount:1});
  await send('Input.dispatchMouseEvent',{type:'mouseReleased',...pdfLink,button:'left',clickCount:1});
  let pdfCopy;
  for(let attempt=0;attempt<50;attempt++){try{pdfCopy=await readFile('artifacts/phase5/downloads/informe-prueba.pdf');break;}catch{await new Promise(resolve=>setTimeout(resolve,100));}}
  assert.ok(pdfCopy?.toString().startsWith('%PDF-1.7'),'Falta la copia local del PDF');
  await button('Retirar evidencia','document.querySelector("dialog.evidence-gallery")');
  await button('Ver evidencias retiradas','document.querySelector("dialog.evidence-gallery")');
  await waitFor(`!!document.querySelector('.ev-stage a[download="informe-prueba.pdf"]')`);
  await button('Restaurar evidencia','document.querySelector("dialog.evidence-gallery")');
  await button('Volver a evidencias activas','document.querySelector("dialog.evidence-gallery")');
  await evaluate(`(async () => {if(!MediaRecorder.isTypeSupported('video/mp4'))throw new Error('MP4 de prueba no compatible');const canvas=document.createElement('canvas');canvas.width=64;canvas.height=64;const context=canvas.getContext('2d');const stream=canvas.captureStream(10);const recorder=new MediaRecorder(stream,{mimeType:'video/mp4'});const chunks=[];recorder.ondataavailable=e=>chunks.push(e.data);const stopped=new Promise(resolve=>recorder.onstop=resolve);recorder.start();let frame=0;const drawing=setInterval(()=>{context.fillStyle=frame++%2?'red':'blue';context.fillRect(0,0,64,64);},50);await new Promise(resolve=>setTimeout(resolve,1000));recorder.stop();await stopped;clearInterval(drawing);stream.getTracks().forEach(track=>track.stop());const transfer=new DataTransfer();const file=new File(chunks,'video-prueba.mp4',{type:'video/mp4'});if(!file.size)throw new Error('Grabación de prueba vacía');transfer.items.add(file);const input=document.querySelector('.ev-entry input[type=file]');input.files=transfer.files;input.dispatchEvent(new Event('change',{bubbles:true}));})()`);
  await waitFor('document.querySelector(".ev-stage video")?.readyState>=1');
  assert.ok(await evaluate('document.querySelector(".ev-stage video").controls'));
  await evaluate('document.querySelector(".ev-stage video").play()');
  await waitFor('document.querySelector(".ev-stage video").currentTime>0');
  await evaluate('document.querySelector(".ev-stage video").pause()');
  await unlink('artifacts/phase5/downloads/video-prueba.mp4').catch(error=>{if(error.code!=='ENOENT')throw error;});
  await evaluate('document.querySelector(".ev-video a[download]").click()');
  let videoCopy;
  for(let attempt=0;attempt<50;attempt++){try{videoCopy=await readFile('artifacts/phase5/downloads/video-prueba.mp4');break;}catch{await new Promise(resolve=>setTimeout(resolve,100));}}
  assert.ok(videoCopy?.includes(Buffer.from('ftyp')),'Falta la copia local del video');
  for(const width of [375,760,1280]){
    await send('Emulation.setDeviceMetricsOverride',{width,height:900,deviceScaleFactor:1,mobile:false});await settle();
    assert.ok(await evaluate('document.querySelector("dialog.evidence-gallery").scrollWidth<=document.querySelector("dialog.evidence-gallery").clientWidth+1'));
    await screenshot(`video-evidence-${width}`);
  }
  await button('Retirar evidencia','document.querySelector("dialog.evidence-gallery")');
  await button('Ver evidencias retiradas','document.querySelector("dialog.evidence-gallery")');
  await waitFor('!!document.querySelector(".ev-stage video")');
  await button('Restaurar evidencia','document.querySelector("dialog.evidence-gallery")');
  await button('Volver a evidencias activas','document.querySelector("dialog.evidence-gallery")');
  console.log('PDF y MP4: carga, copia de PDF, reproducción de video, retiro/restauración y tres anchos aprobados con IPC simulado.');
  await evaluate('document.querySelector("dialog.evidence-gallery .ev-close").click()');
  for(const width of [760,375]){
    await send('Emulation.setDeviceMetricsOverride',{width,height:900,deviceScaleFactor:1,mobile:false});
    await settle();
    assert.ok(await evaluate('document.documentElement.scrollWidth <= innerWidth + 1'),`Antecedentes a ${width}px`);
  }
  console.log('Antecedentes y descripción, retiro y restauración de evidencia aprobados.');
  await send('Emulation.setDeviceMetricsOverride',{width:1000,height:680,deviceScaleFactor:1,mobile:false});
  await view('clients');
  await evaluate('document.querySelector(".header-tools summary").click()');
  await button('Respaldos','document.querySelector(".header-tools-menu")');
  await waitFor('!!document.querySelector(".backup-dialog[open]")');
  assert.equal(await evaluate('document.activeElement.id'),'backup-master');
  await evaluate(`(()=>{const input=document.getElementById('backup-master');input.value='incorrecta';input.dispatchEvent(new Event('input',{bubbles:true}));})()`);
  await button('Abrir Respaldos','document.querySelector(".backup-dialog")');
  await waitFor('document.querySelector(".backup-error")?.textContent.includes("incorrectos")');
  await evaluate(`(()=>{const input=document.getElementById('backup-master');input.value='maestra-de-prueba';input.dispatchEvent(new Event('input',{bubbles:true}));})()`);
  await button('Abrir Respaldos','document.querySelector(".backup-dialog")');
  await waitFor('document.querySelector(".backup-dialog")?.textContent.includes("Aún no hay respaldos")');
  await button('Crear respaldo ahora','document.querySelector(".backup-dialog")');
  await waitFor('document.querySelector(".backup-dialog")?.textContent.includes("Respaldo creado")');
  await evaluate('document.querySelector(".backup-toggle input").click()');
  await waitFor('document.querySelector(".backup-dialog")?.textContent.includes("Respaldo al cerrar activado")');
  await screenshot('backups-configured');
  await send('Emulation.setDeviceMetricsOverride',{width:375,height:812,deviceScaleFactor:1,mobile:false});
  await settle();assert.ok(await evaluate('document.querySelector(".backup-dialog").scrollWidth <= document.querySelector(".backup-dialog").clientWidth + 1'));
  await screenshot('backups-mobile');
  await send('Emulation.setDeviceMetricsOverride',{width:1000,height:680,deviceScaleFactor:1,mobile:false});
  await button('Abrir carpeta Respaldos','document.querySelector(".backup-dialog")');
  await button('Restaurar un respaldo','document.querySelector(".backup-dialog")');
  await evaluate(`(()=>{const input=document.getElementById('backup-restore-password');input.value='maestra-de-prueba';input.dispatchEvent(new Event('input',{bubbles:true}));})()`);
  await button('Seleccionar archivo y restaurar','document.querySelector(".backup-dialog")');
  await waitFor('document.querySelector(".backup-dialog")?.textContent.includes("Reiniciar HRL CORE")');
  assert.ok(await evaluate('document.querySelector(".backup-dialog header button").disabled'));
  for(const level of ['limited','custom']){
    await view(`clients&level=${level}`);
    assert.equal(await evaluate('[...document.querySelectorAll(".header-tools-menu button")].some(button=>button.textContent.trim()==="Respaldos")'),false);
  }
  console.log('Respaldos: permiso Completo, clave maestra, creación manual, configuración y estado de restauración aprobados con IPC simulado.');
  await send('Emulation.setDeviceMetricsOverride',{width:1000,height:680,deviceScaleFactor:1,mobile:false});
  await send('Page.navigate',{url:'http://127.0.0.1:1421/tests/ui-phase4.html'});
  await waitFor('!!document.querySelector("#access-name[role=combobox]")');
  const point=await evaluate('(()=>{const r=document.getElementById("access-name").getBoundingClientRect();return {x:r.x+24,y:r.y+r.height/2};})()');
  await send('Input.dispatchMouseEvent',{type:'mousePressed',button:'left',clickCount:1,...point});
  await send('Input.dispatchMouseEvent',{type:'mouseReleased',button:'left',clickCount:1,...point});
  await waitFor('document.querySelectorAll("#access-name-options [role=option]").length === 4');
  await screenshot('login-users');
  const typeUser=value=>evaluate(`(()=>{const field=document.getElementById('access-name');field.value=${JSON.stringify(value)};field.dispatchEvent(new Event('input',{bubbles:true}));})()`);
  const key=async value=>{
    await send('Input.dispatchKeyEvent',{type:'keyDown',key:value,code:value,...(value==='Enter'?{windowsVirtualKeyCode:13,text:'\r'}:{})});
    await send('Input.dispatchKeyEvent',{type:'keyUp',key:value,code:value});
    await settle();
  };
  await typeUser('LU');
  await waitFor('document.querySelectorAll("#access-name-options [role=option]").length === 1');
  assert.equal(await evaluate('document.querySelector("#access-name-options [role=option]").textContent'),'lulu');
  await key('Enter');
  assert.equal(await evaluate('document.getElementById("access-name").value'),'lulu');
  assert.equal(await evaluate('window.loginRequests.length'),0);
  await key('ArrowDown');await key('Home');await key('ArrowDown');await key('Enter');
  assert.equal(await evaluate('document.getElementById("access-name").value'),'ana');
  await evaluate('document.getElementById("access-name").click()');
  await waitFor('document.querySelectorAll("#access-name-options [role=option]").length === 4');
  await evaluate('[...document.querySelectorAll("#access-name-options [role=option]")].find(option=>option.textContent==="hector").click()');
  assert.equal(await evaluate('document.getElementById("access-name").value'),'hector');
  await typeUser('sin-coincidencias');
  await waitFor('document.querySelector(".search-select-dropdown")?.textContent.includes("No hay usuarios que coincidan")');
  await key('Escape');
  assert.equal(await evaluate('document.getElementById("access-name").getAttribute("aria-expanded")'),'false');
  await typeUser('lulu');await key('Enter');
  await evaluate(`(()=>{const field=document.getElementById('access-password');field.value='clave-de-prueba';field.dispatchEvent(new Event('input',{bubbles:true}));})()`);
  await button('Iniciar sesión');
  await waitFor('window.loginRequests.length === 1');
  assert.deepEqual(await evaluate('window.loginRequests[0]'),{payload:{username:'lulu',password:'clave-de-prueba'}});
  await send('Page.navigate',{url:'http://127.0.0.1:1421/tests/ui-phase4.html?usersError'});
  await waitFor('document.body.textContent.includes("No se pudo cargar la lista")');
  assert.equal(await evaluate('document.getElementById("access-name").disabled'),false);
  assert.ok(await evaluate('[...document.querySelectorAll("button")].some(button=>button.textContent.trim()==="Volver a consultar usuarios")'));
  await screenshot('login-users-error');
  await send('Emulation.setDeviceMetricsOverride',{width:375,height:812,deviceScaleFactor:1,mobile:false});
  await evaluate('document.getElementById("access-name").focus()');
  await settle();
  assert.ok(await evaluate('document.documentElement.scrollWidth <= innerWidth+1'));
  await screenshot('login-users-mobile');
  await typeUser('hector');
  await evaluate(`(()=>{const field=document.getElementById('access-password');field.focus();field.value='clave-manual-de-prueba';field.dispatchEvent(new Event('input',{bubbles:true}));})()`);
  await button('Iniciar sesión');
  await waitFor('window.loginRequests.length === 1');
  assert.deepEqual(await evaluate('window.loginRequests[0]'),{payload:{username:'hector',password:'clave-manual-de-prueba'}});
  await send('Page.navigate',{url:'http://127.0.0.1:1421/tests/ui-phase4.html?setup'});
  await waitFor('document.querySelector(".access-main h1")?.textContent === "Define la clave maestra"');
  assert.equal(await evaluate('document.querySelector("[role=combobox]")'),null);
  console.log('Selector de acceso: clic, búsqueda, flechas/Enter, selección directa, Escape y fallo recuperable aprobados.');
  assert.deepEqual(errors, []);
  console.log('Revisión de navegador aprobada.');
} finally {
  downloadSocket?.close();
  socket.close();
}
