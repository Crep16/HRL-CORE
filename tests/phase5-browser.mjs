import assert from 'node:assert/strict';
import { mkdir, writeFile } from 'node:fs/promises';

const pages = await (await fetch('http://127.0.0.1:9224/json/list')).json();
const targets = pages.filter(page => page.type === 'page' && (page.url === 'about:blank' || page.url.startsWith('http://127.0.0.1:1421/tests/')));
assert.equal(targets.length, 1);
const socket = new WebSocket(targets[0].webSocketDebuggerUrl);
await new Promise((resolve, reject) => {
  socket.addEventListener('open', resolve, { once: true });
  socket.addEventListener('error', reject, { once: true });
});
let sequence = 0;
const pending = new Map();
const errors = [];
socket.addEventListener('message', event => {
  const message = JSON.parse(event.data);
  if (message.method === 'Runtime.exceptionThrown') errors.push(message.params.exceptionDetails.text);
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
  const result = await send('Runtime.evaluate', { expression, returnByValue: true, awaitPromise: true });
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
  await send('Emulation.setDeviceMetricsOverride', { width: 1280, height: 900, deviceScaleFactor: 1, mobile: false });
  for (const name of ['clients', 'repairs', 'company', 'users', 'logbook']) {
    await view(name);
    const layout = await evaluate('({heading: document.querySelector("h1")?.innerText, width: innerWidth, content: document.documentElement.scrollWidth})');
    const headings = { clients: 'Órdenes', repairs: 'Reparaciones', company: 'Empresa', users: 'Usuarios', logbook: 'Bitácora interna' };
    assert.equal(layout.heading, headings[name], name);
    assert.ok(layout.content <= layout.width + 1, `${name}: desbordamiento horizontal`);
    if (name === 'repairs') {
      assert.equal(await evaluate('(() => { const amount = document.querySelector(".payment-balance strong"); const range = document.createRange(); range.selectNodeContents(amount); return range.getClientRects().length; })()'), 1);
    }
    await screenshot(name);
    console.log(JSON.stringify({ view: name, ...layout }));
  }

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
  assert.ok(await evaluate('document.body.innerText.includes("Guarda o descarta")'));
  await screenshot('company-draft-guard');
  console.log('Borrador de cláusula, foco y guarda de navegación aprobados.');

  await view('clients');
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
  assert.deepEqual(errors, []);
  console.log('Revisión de navegador aprobada.');
} finally {
  socket.close();
}
