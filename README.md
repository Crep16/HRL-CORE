# HRL CORE

Aplicación de escritorio offline-first para un taller de reparación de equipos
de cómputo. Usa Vue 3, TypeScript, Pinia, Tailwind, Tauri, Rust y SQLite.

## Operación

- Clientes: recepción, búsqueda y edición de órdenes, antecedentes en texto libre y papelera recuperable.
- Reparaciones: etapas, notas públicas e internas, garantías, refacciones y movimientos de dinero.
- Evidencias: fotografías JPG, PNG y WebP, descripción, retiro y restauración conservando el original.
- Bitácora y Finanzas: auditoría inmutable, saldos, costos y totales separados por moneda.
- Empresa y Usuarios: configuración de documentos, cuentas, permisos y autorización temporal.

El primer inicio solicita clave maestra y administrador. No hay cuentas ni
órdenes predeterminadas. Las operaciones se autorizan en Rust y guardan sus
cambios y auditoría en la misma transacción. Los importes usan centavos enteros;
abonos, ajustes y devoluciones conservan sus movimientos originales.

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

`legacy/frontend/`, `src-tauri/legacy/` y `src-tauri/migrations/legacy/`
conservan referencias y pruebas del sistema anterior. No se cargan en la
aplicación. Las pruebas Rust históricas siguen compilándose solo con `cfg(test)`.

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
cargo check --offline --manifest-path src-tauri/Cargo.toml
cargo test --offline --manifest-path src-tauri/Cargo.toml
```

`test:browser` comprueba la interfaz real con IPC simulado. Necesita Vite en
`127.0.0.1:1421` y Chrome con un perfil aislado y depuración en el puerto 9224;
guarda capturas en `artifacts/phase5`. No usa la base del taller.

El perfil Rust de desarrollo conserva información básica de depuración y
desactiva la caché incremental para reducir espacio en disco.

## Pendientes y documentación

La publicación en el portal requiere un contrato y entorno acordados. La
importación histórica, los formatos adicionales de evidencia y el traslado
de claves entre instalaciones necesitan definiciones previas. Cámara,
impresión física y aceptación con personal se validan en el equipo acordado.
No se genera instalador durante esta etapa por indicación del usuario.

Las reglas vigentes están en `docs/*.md`. Lee [AGENTS.md](AGENTS.md) antes de
modificar código y [arquitectura objetivo](docs/arquitectura-objetivo.md) para
seguir las dependencias y convenciones.
