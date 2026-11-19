# HRL CORE

Aplicación de escritorio offline-first para un taller de reparación de equipos
de cómputo. Usa Vue 3, TypeScript, Pinia, Tailwind, Tauri, Rust y SQLite.

## Operación

- Clientes: recepción, búsqueda y edición de órdenes, antecedentes en texto libre y papelera recuperable.
- Reparaciones: etapas, notas públicas e internas, garantías, refacciones y movimientos de dinero.
- Evidencias: fotografías JPG, PNG y WebP, documentos PDF y videos MP4; captura con cámara, descripción, retiro y restauración conservando el original.
- Finanzas: cobros, saldos, costos y ganancias estimadas de trabajos pagados y con deuda, separados por moneda y entrega.
- Empresa y Usuarios: configuración de documentos, cuentas, permisos y autorización temporal.

El primer inicio solicita clave maestra y administrador. No hay cuentas ni
órdenes predeterminadas. Las operaciones se autorizan en Rust y guardan sus
cambios y auditoría en la misma transacción. Los importes usan centavos enteros;
abonos, ajustes y devoluciones conservan sus movimientos originales.

**Herramientas → Respaldos** está disponible para acceso Completo con clave
maestra. Permite crear copias cifradas, abrir su carpeta y activar el respaldo
al cerrar. El archivo `.hrlbackup` incluye base, recursos y custodia portable;
se abre con la clave maestra vigente al crearlo. Copia los archivos a un USB o
disco externo para cubrir la pérdida de la computadora. Una instalación vacía
permite restaurar desde el acceso inicial. Véase [respaldos portables](docs/respaldos-portables.md).

## Arquitectura y datos

El frontend usa vistas, stores y composables. Solo `src/services/api/` llama a
Tauri. El backend se organiza en `commands/`, `services/`, `repositories/`,
`domain/` y `db/`. El SQL del negocio está en los repositorios.

La base vigente es `hrl_core_3fn.db`. El esquema inicial está en
`docs/esquema-objetivo.sql`; las extensiones se aplican mediante migraciones
numeradas en `src-tauri/migrations/`. No se almacena negocio como JSON.
La migración 003 añade antecedentes libres sin borrar los campos anteriores.

Las fotografías y el logo se guardan en `assets-3fn`; las claves cifradas del
equipo requieren su custodia DPAPI original. Un respaldo útil debe incluir
base, recursos y custodia. Copiar solamente SQLite no basta.
Consulta [recuperación local](docs/instalacion-recuperacion-local.md).

`src-tauri/legacy/` y `src-tauri/migrations/legacy/` conservan las dependencias
de las pruebas de regresión Rust. No se cargan en la aplicación de producción;
siguen compilándose solo con `cfg(test)`. `schema.sql` se conserva porque una
prueba comprueba que coincide con la migración base.

`docs/` conserva las reglas funcionales y técnicas vigentes. Los diseños,
exportaciones y el frontend archivado no forman parte del código activo.
`dist/` se genera con `npm.cmd run build`; no se versiona y Tauri lo regenera
mediante `beforeBuildCommand` antes de empaquetar. Los perfiles y capturas de
prueba de `artifacts/` tampoco se versionan.

## Desarrollo y validación

En PowerShell con scripts deshabilitados, usa `npm.cmd`.

```powershell
npm.cmd install
npm.cmd run tauri -- dev
```

`npm.cmd run dev` sirve la interfaz; el acceso a los datos locales requiere
la aplicación Tauri. Para validar sin generar instaladores:

```powershell
npm.cmd run build
npm.cmd run test:ipc
npm.cmd run test:ui
npm.cmd run test:company
npm.cmd run test:finances
npm.cmd run test:phase5
npm.cmd run test:payments
npm.cmd run test:backups
cargo check --offline --manifest-path src-tauri/Cargo.toml
cargo test --offline --manifest-path src-tauri/Cargo.toml
```

`test:browser` comprueba la interfaz real con IPC simulado. Necesita Vite en
`127.0.0.1:1421` y Chrome con un perfil aislado y depuración en el puerto 9224;
guarda capturas en `artifacts/phase5`. No usa la base del taller.

El perfil Rust de desarrollo conserva información básica de depuración y
desactiva la caché incremental para reducir espacio en disco.

## Pendientes y documentación

El proyecto de trabajo en Jira es **HRL Proyecto**, clave **HRLP**. La publicación
en el portal requiere aprobar el contrato y conectar el servicio externo.
La importación de la base antigua requiere confirmar qué datos se conservarán.
Cámara, impresión física, instalación, restauración entre perfiles y aceptación
con personal se validan en el equipo acordado; las pruebas simuladas no acreditan
ese recorrido.

Las reglas vigentes se enumeran en [documentación](docs/README.md). Lee [AGENTS.md](AGENTS.md) antes de
modificar código y [arquitectura objetivo](docs/arquitectura-objetivo.md) para
seguir las dependencias y convenciones.
