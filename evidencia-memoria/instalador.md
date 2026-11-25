# Instalador

| Pregunta | Hallazgo |
|---|---|
| ¿Existe configuración para generar instalador? | **[EVIDENCIA] Sí.** `src-tauri/tauri.conf.json` tiene `bundle.targets: "all"`. La guía `docs/instalacion-recuperacion-local.md` indica como salida `src-tauri/target/release/bundle/`. |
| ¿Qué herramienta lo genera? | **[EVIDENCIA] Tauri 2 CLI / empaquetado Tauri**, invocado por el script npm `tauri build`. No se encontró configuración de Inno Setup o WiX en las rutas buscadas. |
| Versión declarada | **[EVIDENCIA] 0.1.0** en `package.json`, `src-tauri/Cargo.toml` y `src-tauri/tauri.conf.json`. La versión de Tauri está declarada como dependencia mayor `2`; la versión exacta instalable de `tauri-cli` no está fijada en esos archivos. |
| Comando | **[EVIDENCIA]** `npm.cmd run tauri build` (la guía lo escribe `npm run tauri build`; en el entorno PowerShell reportado, usaron `npm.cmd` por política de scripts). `beforeBuildCommand` ejecuta `npm run build`. |
| ¿Se probó el instalador? | En `docs/evidencia/` y reportes revisados no hay registro de ejecución o instalación de paquete generado. La guía pide conservar versión y SHA-256 del instalador entregado, lo que es instrucción, no constancia de prueba. **NO ENCONTRADO EN EL REPOSITORIO** como prueba ejecutada. |
| Rutas de configuración | `src-tauri/tauri.conf.json`; `src-tauri/Cargo.toml`; `package.json`; `docs/instalacion-recuperacion-local.md`. |

**[INFERIDO]** La configuración permite producir artefactos bundle para las plataformas soportadas por el entorno. Eso no prueba que haya un instalador ya generado, firmado, instalado o entregado; no se comprobó contenido de `target/release/bundle/`.
