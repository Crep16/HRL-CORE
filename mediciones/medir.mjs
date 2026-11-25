import { DatabaseSync } from 'node:sqlite';
import { readFileSync, writeFileSync, mkdirSync, readdirSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import os from 'node:os';
import { execFileSync } from 'node:child_process';
import { performance } from 'node:perf_hooks';

const root = new URL('../', import.meta.url).pathname.replace(/^\/(\w:)/, '$1').replaceAll('/', '\\');
const out = new URL('./mediciones.csv', import.meta.url);
const runDate = new Date().toISOString();
const commit = execFileSync('git', ['rev-parse', 'HEAD'], { cwd: root, encoding: 'utf8' }).trim();
const dirty = execFileSync('git', ['status', '--porcelain=v1'], { cwd: root, encoding: 'utf8' }).trim().length > 0;
const cpu = os.cpus()[0]?.model ?? 'desconocido';
const ram = os.totalmem();
const osName = `${os.type()} ${os.release()} ${os.arch()}`;
const columns = ['fecha_utc','commit','cambios_sin_confirmar','sistema_operativo','cpu','ram_bytes','medicion','n_ordenes','iteracion','tiempo_ms','resultado','comando'];
const rows = [];
const sql = (s) => s.replaceAll('"', '""');
const add = (measurement, n, iteration, ms, result, command) => rows.push([runDate,commit,dirty,osName,cpu,ram,measurement,n,iteration,ms,result,command].map(v => `"${sql(String(v))}"`).join(','));
const migrationDir = join(root, 'src-tauri', 'migrations');
const migrationFiles = readdirSync(migrationDir).filter(name => /^\d{3}_.*\.sql$/.test(name)).sort();
const commands = 'node mediciones/medir.mjs';
const countCommand = 'node mediciones/medir.mjs';
const now = '2026-11-22T12:00:00.000Z';

for (const count of [100, 1000, 10000]) {
  const db = new DatabaseSync(':memory:');
  db.exec('PRAGMA foreign_keys=ON');
  for (const file of migrationFiles) db.exec(readFileSync(join(migrationDir, file), 'utf8'));
  const level = db.prepare("SELECT id FROM niveles_acceso WHERE codigo='complete'").get().id;
  const role = db.prepare("SELECT id FROM roles WHERE codigo='administrator'").get().id;
  const status = db.prepare('SELECT id FROM estados_orden ORDER BY id LIMIT 1').get().id;
  db.prepare('INSERT INTO usuarios(nombre,nombre_acceso,hash_contrasena,id_nivel,id_rol) VALUES(?,?,?,?,?)')
    .run('Usuario sintético', 'medicion', 'hash-sintetico', level, role);
  const client = db.prepare('INSERT INTO clientes(nombre,telefono) VALUES(?,?)');
  const equipment = db.prepare('INSERT INTO equipos(id_cliente,articulo,marca,modelo,serie) VALUES(?,?,?,?,?)');
  const order = db.prepare('INSERT INTO ordenes(folio,id_equipo,id_creador,id_estado,fecha_recepcion,falla) VALUES(?,?,?,?,?,?)');
  db.exec('BEGIN');
  for (let i = 1; i <= count; i++) {
    client.run(`Cliente Sintético ${String(i).padStart(5, '0')}`, '5550000000');
    const clientId = Number(db.prepare('SELECT last_insert_rowid() AS id').get().id);
    equipment.run(clientId, 'Laptop', 'Marca sintética', 'Modelo sintético', `SER-${String(i).padStart(8, '0')}`);
    const equipmentId = Number(db.prepare('SELECT last_insert_rowid() AS id').get().id);
    order.run(`HRL-${String(i).padStart(8, '0')}`, equipmentId, 1, status, now, 'Falla sintética');
  }
  db.exec('COMMIT');

  const searches = [
    ['busqueda_folio', db.prepare('SELECT o.id FROM ordenes o WHERE o.folio=?'), `HRL-${String(count).padStart(8, '0')}`],
    ['busqueda_nombre_cliente', db.prepare('SELECT o.id FROM ordenes o JOIN equipos e ON e.id=o.id_equipo JOIN clientes c ON c.id=e.id_cliente WHERE c.nombre=?'), `Cliente Sintético ${String(count).padStart(5, '0')}`],
    ['busqueda_numero_serie', db.prepare('SELECT o.id FROM ordenes o JOIN equipos e ON e.id=o.id_equipo WHERE e.serie=?'), `SER-${String(count).padStart(8, '0')}`],
  ];
  for (const [name, statement, value] of searches) {
    for (let iteration = 1; iteration <= 10; iteration++) {
      const start = performance.now();
      const result = statement.get(value);
      const elapsed = performance.now() - start;
      if (!result) throw new Error(`Sin resultado para ${name}, n=${count}`);
      add(name, count, iteration, elapsed.toFixed(6), result.id, commands);
    }
  }
  db.close();
}

const catalog = new DatabaseSync(':memory:');
catalog.exec('PRAGMA foreign_keys=ON');
for (const file of migrationFiles) catalog.exec(readFileSync(join(migrationDir, file), 'utf8'));
const tables = Number(catalog.prepare("SELECT count(*) AS n FROM sqlite_schema WHERE type='table' AND name NOT LIKE 'sqlite_%'").get().n);
let foreignKeys = 0;
for (const { name } of catalog.prepare("SELECT name FROM sqlite_schema WHERE type='table' AND name NOT LIKE 'sqlite_%'").all()) {
  foreignKeys += catalog.prepare(`PRAGMA foreign_key_list("${name.replaceAll('"', '""')}")`).all().length;
}
const commandCount = (readFileSync(join(root, 'src-tauri', 'src', 'lib.rs'), 'utf8').match(/commands::[\w:]+,/g) ?? []).length;
add('conteo_tablas', 0, 1, '', tables, countCommand);
add('conteo_claves_foraneas', 0, 1, '', foreignKeys, countCommand);
add('conteo_migraciones_activas', 0, 1, '', migrationFiles.length, countCommand);
const legacyMigrations = readdirSync(join(migrationDir, 'legacy')).filter(name => name.endsWith('.sql')).length;
add('conteo_migraciones_legacy', 0, 1, '', legacyMigrations, countCommand);
add('conteo_comandos_tauri', 0, 1, '', commandCount, countCommand);
catalog.close();

const header = columns.join(',');
mkdirSync(new URL('.', out), { recursive: true });
writeFileSync(out, `${header}\n${rows.join('\n')}\n`, 'utf8');
console.log(`Registros crudos escritos: ${rows.length}; archivo: mediciones/mediciones.csv`);
