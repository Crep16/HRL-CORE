# Tamaño por sesión

## Acceso normalizado

`App.vue` observa la sesión de `useAuthStore` mediante
`useWindowProfile`. El adaptador `services/api/window.ts` solicita
`sync_normalized_window_profile` sin enviar tamaños ni perfiles. El servicio
de acceso consulta `hrl_core_3fn.db` y mantiene su exclusión de cambios de sesión
hasta terminar el ajuste nativo. La sesión del taller anterior no participa.
Los fallos de ventana se muestran con una opción de reintento y no cambian
las credenciales ni la sesión.

El acceso conserva `access.css`: Segoe UI, columna lateral de 245 px,
campos de al menos 44 px y títulos de 25 px. Los campos de contraseña usan
el control de revelado nativo de WebView2; no añaden un botón duplicado.
La recuperación usa los mismos estilos y puede desplazarse verticalmente
dentro del tamaño fijo del acceso.

Validación del ajuste: build y `cargo check` correctos; 17 pruebas frontend
correctas. Rust: 79 de 80, incluida la transición de perfiles normalizados y
las dos pruebas de geometría. Persiste el fallo ya observado de custodia
DPAPI (`Crypto`), ajeno a este ajuste. Contrastes del acceso: texto 14.97:1,
ayuda 5.56:1, botón principal 5.18:1 y enlace 8.32:1.
La inspección nativa no pudo completarse: la herramienta detectó HRL pero
falló al asociar su ventana al proceso. No se acredita un recorrido visual
ni de redimensionamiento nativo.

El backend decide el perfil a partir de la sesión real:

- Acceso: 1000 × 680, tamaño fijo, sin maximizar.
- Configuración inicial: 1000 × 760, fijo durante sus dos pasos.
- Taller: tamaño inicial 1280 × 760, mínimo 1120 × 680 y redimensionamiento habilitado.

Cerrar sesión desmaximiza y centra la ventana, y restablece el tamaño exacto
de acceso. Consultar nuevamente la sesión mientras se trabaja no cambia
el tamaño elegido por el usuario. Los límites anteriores se retiran antes
de aplicar los siguientes, para que el mínimo del taller no impida encoger
la pantalla de acceso ni su máximo impida ampliar el taller.

La orden de sincronizar no recibe un perfil elegido por el frontend: consulta
la sesión y se serializa con alta, acceso y cierre de sesión. No se conceden
permisos genéricos de redimensionamiento al frontend.

Pruebas: transiciones acceso → taller → acceso, cierre desde maximizado,
conservación del tamaño del taller ante consultas repetidas y tamaño constante
de configuración inicial. Son pruebas de política con una ventana simulada
que aplica límites; no sustituyen el recorrido de una ventana nativa real.
Compilación Rust, TypeScript y Vite correctas.
