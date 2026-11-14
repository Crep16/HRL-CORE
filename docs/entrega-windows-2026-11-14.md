# Compilación de Windows del 14 de noviembre de 2026

Código de referencia `770d63abcae1567d0d4cb4ea8ed5ab1e6574abe3`, rama `refactor/arquitectura`, versión 0.1.0 y destino Windows x64.

## Archivos generados

| Archivo | Tamaño en bytes | SHA-256 |
| --- | ---: | --- |
| `src-tauri/target/release/bundle/nsis/hrl-core_0.1.0_x64-setup.exe` | 3095457 | `5E9AC72CECD20914592AAD7733B33000E24068EA8740CFDC1A38939D3F0D2D34` |
| `src-tauri/target/release/hrl-core.exe` | 8156672 | `7A485845502FAC452667956130D80649C8C1C80A3637D5D6E87D74A069C69999` |

Comando ejecutado desde la raíz del repositorio.

```powershell
npm.cmd run tauri -- build --bundles nsis --ci -- --offline
```

El frontend y la compilación Rust de producción terminaron correctamente. El primer intento produjo el ejecutable y encontró acceso denegado al recrear NSIS. El reintento con acceso fuera del aislamiento actualizó la caché de herramientas y generó el instalador. Tauri descargó NSIS 3.13 y `nsis_tauri_utils` 0.5.3 desde sus repositorios oficiales y validó sus hashes. El argumento `--offline` se aplicó a Cargo.

## Validaciones de referencia

- `cargo check --offline` aprobado.
- 93/93 pruebas Rust aprobadas, incluyendo SQLite, auditoría, concurrencia y DPAPI.
- 38/38 pruebas frontend aprobadas.
- Build TypeScript y Vite aprobado.
- Revisión de navegador headless aprobada con datos aislados e IPC simulado.

La instalación, reapertura y recuperación conjunta con la versión instalada corresponden a HRL-181. La comprobación de cámara, impresora y entorno Windows de destino corresponde a HRL-132/141. La aceptación por personal del taller corresponde a HRL-182/183. El procedimiento está en `instalacion-recuperacion-local.md`.

El reporte de cambios y pruebas de vistas está en `fase-5-vistas-y-validacion.md`. La integración del portal conserva pendiente su contrato y entorno de intercambio.
