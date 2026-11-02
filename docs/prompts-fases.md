# Prompts por fase para Codex

Cómo usarlos: trabaja en una rama nueva (`git checkout -b refactor/arquitectura`). Pega **un prompt a la vez**,
revisa el resultado, ejecuta las pruebas y haz commit antes de pasar a la siguiente fase.
Antes de empezar, haz commit o stash de tus cambios pendientes (`package.json` y `README.md` aparecen modificados y
`extract-reference.mjs` sin seguimiento) para que la rama de refactor parta limpia. Antes de la fase 1 coloca en el repo: `AGENTS.md` (raíz), `docs/arquitectura-objetivo.md`,
`docs/esquema-objetivo.sql` y `docs/prompts-fases.md`.

---

## Fase 0 — Inventario (solo lectura)

```
Lee AGENTS.md, docs/arquitectura-objetivo.md y docs/esquema-objetivo.sql.
NO modifiques ningún archivo en esta fase.

1. Recorre src-tauri/src/ y src/. Entrégame un inventario: cada comando Tauri, qué tablas toca,
   y qué reglas de negocio contiene (permisos, saldos, folios, revisiones).
2. Lista los estados de orden reales y las etapas reales que usa el código (para sustituir el catálogo provisional
   del esquema).
3. Dime qué partes del código dependen de los campos JSON (datos_json, operaciones_json, solicitud_json).
4. Dime dónde hay SQL fuera de un único módulo, y dónde hay lógica de negocio en componentes Vue.
5. Dime qué hay en src/data/ y src/utils/ (¿catálogos, datos de prueba, reglas de negocio?), cuántas líneas tiene
   src/WorkshopApp.vue y qué responsabilidades mezcla, y qué contiene extract-reference.mjs.
6. Lista las migraciones actuales de src-tauri/migrations/ y cómo se cargan (¿en orden por nombre?, ¿tabla de control?).
   Dime si el backend usa Rust puro o también hay un proceso Node (la descripción del proyecto menciona Node.js local).
7. Propón el mapa archivo-actual → capa-destino. No muevas nada.
Termina con una lista de dudas que necesiten decisión mía.
```

## Fase 1 — Base de datos nueva

```
Aplica la Fase 1 de AGENTS.md. No toques la UI.

- src-tauri/migrations/ ya contiene migraciones del esquema anterior. Muévelas a src-tauri/migrations/legacy/
  (git mv, sin borrarlas), asegúrate de que el cargador NO las ejecute, y crea
  src-tauri/migrations/001_esquema_objetivo.sql a partir de docs/esquema-objetivo.sql como nueva línea base.
  Esto está autorizado por AGENTS.md §5 porque no hay datos de producción; recrea la base local de desarrollo desde cero.
- Sustituye el catálogo provisional de estados_orden y etapas por los valores reales que encontraste en la Fase 0.
- Implementa el módulo db/ (conexión con PRAGMA foreign_keys=ON por conexión, ejecución de migraciones numeradas).
- Escribe pruebas (cargo test) que creen una base vacía y verifiquen: FK activas, inmutabilidad de abonos/ajustes/
  devoluciones/gastos/auditoría, último administrador, cierre único de accesos temporales, catálogo de permisos
  de solo lectura, vista_saldos_orden con los ejemplos de docs/reglas-pagos.md
  (cobrado 750 y precio 600 → saldo a favor 150; abono 500 corregido a 300 → ajuste -200).
- No migres datos antiguos. Si quedan dudas sobre datos existentes, pregúntame.
Al terminar, resume qué cambió y qué quedó pendiente.
```

## Fase 2 — Dominio y repositorios

```
Aplica la Fase 2. No toques la UI ni los commands todavía.

- Crea domain/ con Money (centavos), Balance, PermissionKey y los structs de cada agregado.
- Crea repositories/ con todo el SQL, un repositorio por agregado, usando solo las tablas del esquema nuevo.
- Ningún repositorio contiene reglas de negocio ni llama a un service.
- Pruebas de cada repositorio contra una base SQLite en memoria creada con las migraciones.
Resume los repositorios creados y sus funciones públicas.
```

## Fase 3 — Servicios y comandos

```
Aplica la Fase 3. No toques la UI.

- Crea services/ con las reglas descritas en docs/*.md: PaymentService, OrderService, RepairService,
  AccessService, EvidenceService, CompanyService, UserService y AuditService.
- Cada servicio recibe sus repositorios por constructor y abre UNA transacción por operación,
  escribiendo la auditoría dentro de ella.
- Reescribe commands/ para que solo validen la forma de entrada y deleguen a services.
  Conserva los nombres y las firmas de los comandos para no romper el frontend.
- Pruebas de servicio por regla: permisos separados de abonar/devolver, no exceder pendiente, devolución completa,
  idempotencia (mismo id_solicitud + misma huella = misma respuesta; otra huella = error), guardado desactualizado
  de reparaciones rechazado, papelera bloquea edición, acceso temporal acotado a apartado y orden.
Elimina el código antiguo que ya no se use. Resume qué reglas quedaron cubiertas por pruebas.
```

## Fase 4 — ViewModels (Pinia) y capa API

```
Aplica la Fase 4 (MVVM).

- Crea src/services/api/ como único lugar con invoke() de Tauri.
- Crea src/models/ con los tipos (y clases de dominio como Money donde aporte encapsulamiento).
- Crea stores Pinia y composables que contengan el estado de pantalla, validación de formularios,
  guardia de borradores y orquestación. Mueve a ellos la lógica que hoy está dentro de componentes .vue.
- Instala Pinia si no está en package.json y regístrala en main.ts.
- src/WorkshopApp.vue concentra la aplicación: identifica sus responsabilidades y extrae su estado y lógica a stores
  (sesión, órdenes, pagos, reparaciones, bitácora, empresa, usuarios) y composables. No lo partas en vistas todavía (Fase 5).
- Revisa src/utils/ y src/data/: mueve a stores/models cualquier regla de negocio que contengan.
- Aún no rediseñes componentes: solo cambia sus fuentes de datos hacia stores/composables.
Verifica que npm run build y las pruebas del frontend pasen. Resume qué lógica se movió y desde dónde.
```

## Fase 5 — Vistas y limpieza

```
Aplica la Fase 5.

- Divide src/WorkshopApp.vue en src/views/ (una vista por apartado: Acceso, Clientes, Reparaciones, Bitácora, Empresa,
  Usuarios) y deja WorkshopApp.vue solo como contenedor con la navegación.
- Reduce los componentes .vue a presentación: props, eventos, plantilla y estilos.
- Verifica que ningún componente importe de services/api ni llame a invoke().
- Elimina cualquier uso residual de datos_json, de tablas antiguas y de archivos obsoletos.
- Actualiza docs/sqlite.md para que describa el esquema nuevo.
- Haz un recorrido manual del flujo: Acceso → Clientes → Evidencias → Reparaciones → Pagos → Entrega → Nota de salida,
  y lista cualquier diferencia de comportamiento respecto a docs/*.md.
Entrega un resumen final con: estructura de carpetas resultante, pruebas ejecutadas y pendientes.
```

---

## Qué revisar tú entre fases (para poder defenderlo)

- Fase 1: abre el `.sql` y localiza una tabla de cada tipo (catálogo, entidad, tabla puente, libro inmutable).
- Fase 2: ¿hay SQL fuera de `repositories/`? (`grep -rn "SELECT\|INSERT" src-tauri/src` fuera de esa carpeta debe dar vacío).
- Fase 3: elige una regla (p. ej. devolución completa) y sigue su recorrido command → service → repository.
- Fase 4: ¿algún `.vue` sigue importando `@tauri-apps/api`? No debería.
- Siempre: pide a Codex que te explique una decisión y comprueba que puedas repetirla con tus palabras.
