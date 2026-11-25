# Entorno de desarrollo y tecnologías observadas

| Elemento | Dato comprobable | Evidencia |
|---|---|---|
| Rust | `rustc 1.98.1 (48a229cea 2026-09-01)` | `docs/evidencia/pruebas-2026-11-23.md` |
| Cargo | `cargo 1.98.1 (797e8a9bc 2026-08-05)` | Mismo reporte |
| Node.js | `v24.15.0` | Mismo reporte |
| npm | `12.0.2` vía `npm.cmd` | Mismo reporte. `npm --version` falló por política que bloqueó `npm.ps1`. |
| Frontend | Vue 3, Vite, TypeScript, Tailwind CSS; Pinia figura en `package.json` | `package.json`, `src/`, `vite.config.ts` |
| Backend | Tauri 2, Rust, SQLx 0.8 con SQLite; Tokio; Serde; reqwest 0.13 | `src-tauri/Cargo.toml` |
| Firebase | Transporte REST implementado hacia Firestore (`firestore.googleapis.com`); autenticación Firebase por API REST descrita en `docs/contrato/INTEGRACION-HRL-CORE.md` y código de `src-tauri/src/services/portal_transport.rs`. No se encontró dependencia del Firebase SDK de JS en `package.json`. | Rutas indicadas |
| Firebase CLI | No aparece dependencia/script de `firebase-tools` en package del proyecto. El contrato cita CLI solo para el emulador del repositorio portal externo. **No encontrado como instalado/verificado aquí.** | `package.json`, `docs/contrato/CONTRATO-FIRESTORE.md` |
| DPAPI | Uso en Windows mediante almacén de claves y dependencia condicional `windows-sys 0.61`; pruebas protegidas por `#[cfg(windows)]`. | `src-tauri/Cargo.toml`, `src-tauri/src/services/portal_transport.rs`, `src-tauri/src/services/key_store.rs`, `src-tauri/src/services/tests.rs` |
| Pruebas navegador | Scripts propios con Node Test Runner y Chrome headless/CDP; `test:browser` ejecuta `tests/phase5-browser.mjs` y reporta recorridos, no framework Playwright/Cypress en dependencias. | `package.json`, `tests/phase5-browser.mjs`, `docs/evidencia/pruebas-2026-11-23.md` |
| Cámara | API de cámara del navegador/Tauri identificada por `EvidenceCamera.vue` y `useEvidenceCamera.ts`; el reporte del 23-11 menciona abrir/cerrar cámara y restaurar foco, pero no identifica modelo físico. | `src/components/orders/EvidenceCamera.vue`; `src/composables/useEvidenceCamera.ts`; `docs/evidencia/ui-tests-2026-11-23.md` |
| Impresión | Notas de entrada/salida y CSS de impresión; configuración incluye papel `letter`, `ticket` o `ticket60`, orientación y copias. No se identifica marca/modelo de impresora. | `src/utils/printCompanyNote.ts`; `src/assets/styles/company-note.css`; `src-tauri/migrations/001_esquema_objetivo.sql`; `docs/instalacion-recuperacion-local.md` |

**[EVIDENCIA]** El reporte de ejecución identifica Windows, PowerShell y la necesidad de levantar Vite y Chrome headless/CDP en puertos locales para la prueba de navegador. La impresora se considera genérica compatible con carta y la cámara, webcam genérica; el repositorio no justifica especificar hardware concreto.
