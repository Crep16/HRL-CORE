# Mediciones de HRL CORE

## Alcance y entorno

Ejecución: 22 de noviembre de 2026, hora local America/Cancun (las filas crudas guardan la marca UTC, 2026-11-23T02:59:37.555Z). Commit medido: `d8074557b7be8fb5cd3cb29967e078e8d25e4806`. El árbol Git tenía cambios sin confirmar: cuatro documentos modificados y dos archivos nuevos preexistentes en `docs/`; además, los archivos de esta medición. No se sobrescribió ninguno de esos cambios preexistentes. El benchmark usa exclusivamente clientes, equipos, órdenes y valores sintéticos.

| Campo | Valor capturado |
|---|---|
| Sistema operativo | Windows_NT 10.0.19045 x64 (Windows 10) |
| CPU | 13th Gen Intel(R) Core(TM) i7-13620H; 16 procesadores lógicos visibles |
| RAM | 33,982,623,744 bytes (32 GiB nominales; valor devuelto por `os.totalmem()`) |
| Runtime del benchmark | Node.js v24.15.0, módulo integrado `node:sqlite` |
| Repeticiones de búsqueda | 10 por combinación de tamaño y campo |
| Base del benchmark | SQLite en memoria; migraciones numeradas activas 001–006, no incluye `legacy/` |

El comando de las búsquedas y conteos fue `node mediciones/medir.mjs`. La duración se tomó con `performance.now()` en milisegundos. Para cada búsqueda se preparó una sentencia SQL equivalente, se solicitaron por igualdad folio, nombre exacto o serie exacta, y se consultó el último registro sintético. Las órdenes se insertaron en una transacción; la búsqueda se midió después, por separado. No se midió la escritura de la base en disco ni el tiempo de renderizado de Vue. El CSV contiene cada repetición individual, junto al comando, fecha UTC, commit, estado del árbol y entorno.

## 1. Búsqueda en base sintética

Los valores son milisegundos por una consulta SQL en memoria. La búsqueda por nombre usa `JOIN` de orden, equipo y cliente y no dispone de índice en `clientes.nombre` en el esquema medido. Las consultas por folio y serie corresponden a las restricciones e índices del esquema/migraciones cargados.

| Órdenes | Búsqueda | Mínimo (ms) | Mediana (ms) | Máximo (ms) | Repeticiones |
|---:|---|---:|---:|---:|---:|
| 100 | Folio exacto | 0.000900 | 0.001050 | 0.006200 | 10 |
| 100 | Nombre exacto de cliente | 0.006000 | 0.006300 | 0.009400 | 10 |
| 100 | Número de serie exacto | 0.000900 | 0.001000 | 0.001800 | 10 |
| 1,000 | Folio exacto | 0.000900 | 0.001000 | 0.009500 | 10 |
| 1,000 | Nombre exacto de cliente | 0.048800 | 0.049950 | 0.063000 | 10 |
| 1,000 | Número de serie exacto | 0.001100 | 0.001100 | 0.001900 | 10 |
| 10,000 | Folio exacto | 0.001000 | 0.001100 | 0.006300 | 10 |
| 10,000 | Nombre exacto de cliente | 0.504100 | 0.521100 | 0.779800 | 10 |
| 10,000 | Número de serie exacto | 0.001100 | 0.001300 | 0.004600 | 10 |

La primera ejecución de cada sentencia se incluyó; no hubo calentamiento descartado. Las cifras de orden submilisegundo se deben leer como tiempos de consulta observados por este reloj y runtime, no como latencia de una búsqueda completa dentro de HRL CORE.

## 2–6. Operaciones y arranque

| Medición solicitada | Resultado | Motivo / alcance |
|---|---|---|
| Crear orden completa mediante el servicio Rust | No medida | Requiere instanciar `OrderService`, acceso, auditoría, publicación y SQLite con estado válido. El benchmark aislado no invoca servicios Rust; no presento una inserción SQL como si fuera la función de dominio. |
| Registrar abono mediante Rust | No medida | No se ejecutó `LedgerService::record` con sesión, autorización, idempotencia y auditoría. |
| Registrar ajuste mediante Rust | No medida | Misma dependencia del servicio financiero y un movimiento/solicitud sintéticos válidos. |
| Registrar devolución mediante Rust | No medida | Requiere estado de saldo a favor y autorización válidos en el servicio. |
| Recalcular saldo con 1,000 órdenes | No medida | No se cronometró `vista_saldos_orden` ni el método del repositorio/servicio bajo 1,000 órdenes. |
| Resumen de Finanzas con 1,000 y 10,000 órdenes | No medida | No se invocó `ReportService::finances`; requiere inicialización de servicios, sesión y autorización. |
| Respaldo cifrado de 1,000 órdenes + 200 evidencias | No medido | No se generó un `.hrlbackup`. El flujo usa autorización, custodia de clave del sistema, archivos de evidencia y Argon2id/AES-GCM. La suite Rust tuvo fallos `Crypto` en pruebas de DPAPI/respaldos; no sustituí este resultado por un benchmark criptográfico distinto. |
| Restauración del respaldo | No medida | No hubo archivo de respaldo generado en este ensayo, por lo que tampoco tamaño ni tiempo de restauración. |
| Arranque de la app compilada | No medido | No inicié una instancia de escritorio conectada a rutas de configuración del perfil: no podía garantizar que no accediera a una base o llaves existentes fuera del entorno sintético. La compilación de frontend no equivale al arranque de Tauri. |

## 7. Conteos verificables

| Conteo / ejecución | Resultado observado | Comando o fuente |
|---|---:|---|
| Tablas en esquema tras aplicar 001–006 | 47 | `node mediciones/medir.mjs`; conteo de `sqlite_schema` excluyendo `sqlite_%` |
| Claves foráneas declaradas | 59 | `node mediciones/medir.mjs`; suma de `PRAGMA foreign_key_list` en todas las tablas |
| Migraciones numeradas activas | 6 (001–006) | `node mediciones/medir.mjs`; archivos numerados en `src-tauri/migrations/` |
| Migraciones históricas en `legacy/` | 13 | `node mediciones/medir.mjs`; archivos `.sql` en `src-tauri/migrations/legacy/` |
| Comandos registrados en `generate_handler!` | 51 | Conteo de referencias `commands::...` en `src-tauri/src/lib.rs` |
| Pruebas Rust | 127 aprobadas, 4 fallidas, 131 totales | `cargo test --offline --manifest-path src-tauri/Cargo.toml` |
| Pruebas interfaz | 71 aprobadas, 0 fallidas, en ocho scripts | Ocho comandos `npm.cmd run test:...` enumerados abajo |
| Cobertura | No configurada/disponible | `cargo llvm-cov --version` y `cargo tarpaulin --version`: ambos comandos no existen; `package.json` no define script de cobertura |
| Compilación frontend | Correcta | `npm.cmd run build` (vue-tsc + Vite) |

Las cuatro pruebas Rust fallidas fueron `dpapi_vault_keeps_only_the_encrypted_refresh_token_and_supports_rotation`, `portable_backups_restore_books_assets_and_device_key_without_the_original_installation`, `windows_key_custody_reopens_dpapi_key_without_storing_it_in_sqlite` y `windows_startup_requires_the_original_key_when_encrypted_credentials_exist`. Cada una terminó con `Err(Crypto)` al desempaquetar. El resumen de la ejecución registró 127 `ok`, 4 fallidas, 0 ignoradas, duración 81.61 s.

Comandos frontend, cada uno ejecutado individualmente:

| Comando | Aprobadas | Fallidas |
|---|---:|---:|
| `npm.cmd run test:ui` | 14 | 0 |
| `npm.cmd run test:company` | 7 | 0 |
| `npm.cmd run test:ipc` | 14 | 0 |
| `npm.cmd run test:finances` | 4 | 0 |
| `npm.cmd run test:phase5` | 12 | 0 |
| `npm.cmd run test:payments` | 4 | 0 |
| `npm.cmd run test:backups` | 6 | 0 |
| `npm.cmd run test:portal` | 10 | 0 |

## Limitaciones y elementos no medidos

- Los resultados de búsqueda son de una sola computadora y de SQLite en memoria con datos sintéticos; no predicen rendimiento de disco, otras computadoras, cargas simultáneas o búsquedas parciales de la interfaz.
- No se midieron tiempos de las funciones Rust solicitadas, finanzas, operaciones contables, respaldo/restauración ni arranque Tauri. No hay cifra estimada o extrapolada para esos puntos.
- No se obtuvo cobertura de pruebas porque no existe una herramienta configurada/instalada en el entorno.
- Las pruebas Rust no quedaron completamente aprobadas: cuatro fallaron con `Crypto`, como se detalla arriba.
- No se midieron tiempos de creación/inserción de las 100/1,000/10,000 filas, ni tamaño de base o de `.hrlbackup`.
- El acceso a WMI para CPU/RAM fue denegado. CPU, cantidad de procesadores lógicos y memoria total se obtuvieron mediante Node.js (`os.cpus()` y `os.totalmem()`).

Los artefactos creados para esta tarea son `mediciones/medir.mjs`, `mediciones/mediciones.csv` y este informe. No se editaron archivos de la aplicación.
