# Tamaño por sesión

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
