import fs from 'node:fs';
import {gunzipSync} from 'node:zlib';
const h=fs.readFileSync('C:/Users/Hector/Documents/ChatGPT/memoria y proyecto de estadias/Bitácora HRL – Clientes.html','utf8');
const m=JSON.parse(h.match(/<script type="__bundler\/manifest">([\s\S]*?)<\/script>/)[1]);
for(const [id,e] of Object.entries(m)) {
 if(!/html|javascript/.test(e.mime))continue;
 const b=Buffer.from(e.data,'base64');const s=(e.compressed?gunzipSync(b):b).toString();
 const hits=[...s.matchAll(/.{0,200}(?:Iniciar sesi|Crear cuenta|maestra|Contraseña|Acceso completo).{0,400}/gi)].map(x=>x[0]);
 if(hits.length) { const t=s.match(/<script type="__bundler\/template">([\s\S]*?)<\/script>/); const inner=t?JSON.parse(t[1]):s; console.log(id,inner.match(/:root\{[^}]+/i)?.[0],inner.match(/<h1[\s\S]*?<\/form>/)?.[0]?.replace(/<[^>]+>/g,' ').slice(0,3000)); }
}
