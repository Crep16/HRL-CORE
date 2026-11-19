# Acceso temporal con autorización

Requisito del usuario e implementación local del 29 de octubre de 2026.

Una opción restringida abre una solicitud de autorización temporal. Una persona
con el permiso correspondiente introduce su usuario y contraseña y un motivo.
La contraseña se verifica con el hash de su cuenta y se vacía en la interfaz
tras cada intento o cancelación. No se guarda en el préstamo, la Bitácora ni el
almacenamiento del navegador. La clave maestra no sustituye esta autenticación.

## Alcance y duración

El operador conserva su usuario y sus permisos permanentes. La persona que
autoriza presta únicamente las facultades que posee de manera permanente en el
apartado solicitado. No puede prestar las que recibió a su vez temporalmente.
Reparaciones queda además ligada al folio autorizado; prestar permisos para A
no permite editar B. Clientes, Empresa, Bitácora y Usuarios corresponden a sus
apartados. El abono inicial pertenece a la recepción; los costos directos se
registran en Bitácora con su comprobación adicional de permiso.

Un aviso identifica a quien autorizó y permite Finalizar acceso temporal. Salir
del apartado, cambiar de orden de Reparaciones o cerrar sesión retira el
préstamo. Antes de salir se guardan o descartan los cambios según el flujo
existente. Una recarga del taller también finaliza la autorización anterior.
La administración de usuarios conserva la comprobación de clave maestra.

En cada consulta u operación, Rust vuelve a comprobar el alcance, el operador
activo y el permiso vigente de quien autorizó. Su revocación o desactivación
impide nuevos usos. Los intentos de contraseña incorrecta no cambian la sesión;
cinco fallos activan una espera de cinco segundos antes del siguiente intento.

## Auditoría

La migración 011 guarda el préstamo en `temporary_access_grants`, con las
identidades de ambas personas, apartado, folio cuando corresponde, motivo y
permisos concedidos. Solo permite finalizarlo una vez, sin reescribir ni borrar
sus datos iniciales. Los disparadores registran inicio y fin en la Bitácora.

Cada operación que usa una facultad prestada registra `access.temporary.use`
en la misma transacción que el cambio. La operación original conserva como
autor al operador, con referencia al autorizador en el evento de préstamo.
Un fallo de auditoría impide conceder el acceso o confirmar el cambio.
Reintentar un movimiento de dinero ya confirmado no duplica su evento de uso.

Si falla la escritura del fin, la autorización se retira de memoria igualmente.
Si la aplicación se interrumpe, la autorización no se recupera al reiniciar:
la consulta inicial cierra los préstamos que no pertenecen a la sesión actual.
El evento explica que el fin se detectó al recuperar la sesión y que la hora
exacta del cierre anterior es desconocida; no la inventa.

## Comprobaciones y límites

Pruebas Rust: alcance por orden y apartado, autoría del operador, autorizador
con permisos personalizados, rechazo de permisos que no posee, revocación,
usuario desactivado, contraseña incorrecta sin cambio de sesión, espera tras
fallos, rollback de concesión/uso, retirada aun si falla el registro de fin,
cambio de sesión, recuperación tras interrupción y devolución con reintento.

Pruebas Vue: solicitud/cancelación, identidad conservada, alcance de interfaz,
retirada inmediata al finalizar, respuesta de autorización para una sesión que
ya cambió y diálogo real compilado con cierre mediante Escape y vaciado de la
contraseña después del rechazo. Son comprobaciones automatizadas, no una
certificación de la aplicación nativa ni de su aspecto visual.

Dirección visual conservada: blanco y gris, azul petróleo, Bricolage Grotesque e
Instrument Sans; ENERGY 1 / RHYTHM 1 / MOTION 1. Se aplican antislop durante,
antislop-ui, antislop-human y UI/UX Pro Max. El diálogo usa semántica nativa,
foco inicial y devolución de foco al cerrar, campos etiquetados y mensajes de
error. No se agregan imágenes, métricas ni animaciones. El recorrido visual
nativo completo sigue pendiente.
