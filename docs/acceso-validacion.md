# Validación de acceso local

HRL-12, 27 de octubre de 2026.

El primer uso configura la clave maestra y el administrador en una sola
transacción. Los formularios existentes llaman a Rust; desaparece el botón
que permitía entrar sin sesión. El taller se monta tras autenticar y cerrar
sesión destruye esa instancia. Los borradores de órdenes o Empresa bloquean
el cierre hasta guardarlos o descartarlos; Reparaciones intenta guardar antes.

Pruebas Rust: alta con hashes Argon2id, claves independientes, rechazo de
credenciales incorrectas y usuarios desactivados, repetición del primer uso,
rollback de configuración ante fallo del alta y reapertura de un archivo
SQLite con cuentas conservadas y sesión vacía. Los datos de estas pruebas
son temporales y no configuran el administrador del usuario.

La aplicación nativa recompilada mostró el formulario de primer uso sin
errores de acceso a los comandos. La lectura de accesibilidad verificó
etiquetas y estructura. La captura y el clic automatizados fallaron por
indisponibilidad de captura de la ventana; no se acredita un recorrido
completo de la interfaz. No se creó ninguna cuenta real durante la prueba.

Dirección conservada: acceso de HRL para el personal del taller, ENERGY 1 /
RHYTHM 1 / MOTION 1. Mismos logo, paleta, tipografía y disposición. No se
generaron recursos ni se añadieron animaciones. El foco de cada formulario
es su acción principal. El color azul existente señala acciones y foco.

Delivery Gate del incremento:

- PASS, comportamiento backend: pruebas de alta, acceso, sesión, persistencia
  y fallo transaccional ejecutadas contra SQLite.
- PASS, compilación: TypeScript/Vite y pruebas Rust completados.
- PASS, contenido: mensajes describen el acceso real y la ausencia confirmada
  de recuperación automática; sin estadísticas, testimonios o enlaces inventados.
- La verificación visual y el recorrido nativo completos quedan pendientes;
  este registro no certifica la interfaz como entrega final de HRL-12.

Referencia del uso de Argon2: https://docs.rs/argon2/0.5.3/argon2/ .
