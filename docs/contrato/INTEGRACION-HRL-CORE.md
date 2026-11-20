# Conexión de HRL CORE con el portal

Estado de implementación: backend, comandos, configuración en Empresa, controles
de publicación en Reparaciones y QR del comprobante disponibles. La validación
local usa bases temporales y el recorrido de navegador usa IPC simulado. No se
ha probado una conexión real ni se ha publicado información en Firebase.

La app conserva las órdenes completas en SQLite y envía únicamente la proyección
definida en [el contrato local](CONTRATO-FIRESTORE.md). La conexión es de una sola
dirección. El portal no modifica órdenes ni recibe datos privados.

## Configuración

La conexión requiere un usuario Completo con permiso de editar Empresa. El correo
y la contraseña corresponden al publicador de Firebase Auth, no al usuario local
de HRL CORE ni al buzón.

Datos comunicados por el responsable del portal:

- Proyecto: `hrl-web`.
- UID: `Ziwbd42Rt9cLZbGsM91LAKAvotO2`.
- API key web y URL HTTPS definitiva: se introducen en la configuración.
- Código de trabajo: doce caracteres de `ABCDEFGHJKMNPQRSTUVWXYZ23456789`,
  agrupados `XXXX-XXXX-XXXX`, según la definición recibida del portal.

Guardar la configuración deja el envío pausado. Conectar verifica el UID de la
cuenta. La contraseña no se guarda: el refresh token se protege mediante DPAPI
del usuario de Windows en `keys/portal-publicador.dpapi`. El ID token permanece
en memoria y se renueva cuando corresponde. No se incluyen credenciales en
respuestas IPC, auditoría ni errores.

Reanudar es una acción explícita y consulta primero las revisiones pendientes:
misma revisión y huella confirma; misma revisión y otra huella es conflicto;
revisión remota superior deja el envío superado. Reconciliar una orden prepara,
con autorización explícita, el contenido local en una revisión superior a la remota.

## Funcionamiento local

- Las órdenes nuevas usan modo automático y pago visible. Se respetan las
  preferencias de seguimientos ya guardados.
- Órdenes, reparaciones y pagos capturan revisión y auditoría en su transacción.
- Pasar a manual invalida lo pendiente no enviado. Publicar captura el contenido
  actual. Retirar genera un documento sin seguimiento.
- Rotar prepara la retirada del código anterior y la publicación del nuevo.
- Un único proceso de envío por instalación evita concurrencia. No mantiene
  transacciones locales abiertas mientras espera a Firebase.
- Los errores transitorios conservan el envío y reintentan con espera creciente
  y variación, hasta cinco minutos. La credencial revocada pausa el envío.
- Los límites del portal no bloquean una orden local válida; dejan error de
  publicación sin recortar el contenido.
- La cola es normalizada e inmutable. No guarda JSON ni saldos calculados: el
  pago se reconstruye con el precio capturado y marcas de los libros inmutables.
- La entrega efectiva registra su instante UTC y vence siete días después. Las
  entregas históricas que solo tienen fecha no reciben una hora inventada.

## Respaldos

La credencial del publicador no se incluye en los respaldos portables. Al restaurar
hay que conectar de nuevo la cuenta; la conexión deja el envío pausado. Antes de
reanudar se revisan conflictos y órdenes superadas. No se incrementa a ciegas la
revisión ni se vuelve a publicar automáticamente un contenido retirado en remoto.

Los códigos creados después del respaldo pueden faltar en la base restaurada.
Su inventario y transferencia requieren coordinación con el responsable del
portal; la app no lista documentos ni adivina códigos perdidos.

## Estado de publicación en la aplicación

La confirmación corresponde a la última revisión del código activo, nunca a una
revisión anterior ni a un código retirado. Si el contenido actual no cumple los
límites públicos, se muestra error aunque antes hubiera una publicación válida.
Una retirada pendiente no muestra la fecha de confirmación del contenido anterior.

Las respuestas IPC anteriores a una rotación o retirada no sustituyen el estado
nuevo. Solo se permite una acción de publicación simultánea por orden; una
consulta durante esa acción solicita volver a actualizar al terminar.
Las consultas de configuración conservan los cambios escritos durante su espera
y no pueden reemplazar una configuración guardada posteriormente.

Pruebas de regresión: `tests/portal.test.ts` y
`src-tauri/src/services/portal_sender_tests.rs`. El recorrido
`tests/portal-browser.mjs` comprueba configuración, conexión, pausa/reanudación,
publicación, retirada, rotación, reconciliación, confirmación, QR y ancho móvil
con IPC simulado. No acredita el despliegue ni los permisos de Firebase real.

Validación local del 19 de noviembre de 2026: `cargo check --offline`, build
TypeScript/Vite, 128 pruebas Rust (40 de módulos legacy) y 68 pruebas TypeScript
aprobadas. El recorrido de navegador del párrafo anterior también pasó. La suite
Rust usa bases temporales y DPAPI del perfil ejecutor, no la base del taller.

## Validación remota pendiente

No se ha publicado una orden real ni verificado el despliegue de Firebase. Faltan
la cuenta publicadora y la URL definitiva para la prueba de punta a punta con Víctor.

Las reglas recibidas aún exigen `pendiente = total - cobrado`. Las reglas locales
propuestas usan `max(total - cobrado, 0)` para respetar el saldo a favor. El portal
debe incorporar ese ajuste y exigir juntos `entregadoEn` y `accesoHasta`.

Retirar o rotar sin Internet queda pendiente hasta confirmación remota. Un envío
en curso puede terminar antes que la revisión restrictiva. El portal debe ocultar
el contenido vencido y manejar la caché; no se recuperan copias ya recibidas.

Las pruebas usan un gateway sustituible para respuesta perdida, conflicto,
revocación y fallo de auditoría tras confirmar en remoto. El REST usa enteros como
texto y `timestampValue`, reemplaza el documento completo y agrega `actualizadoEn`
mediante `REQUEST_TIME`.

Referencias: [Firebase Auth REST](https://firebase.google.com/docs/reference/rest/auth)
y [escrituras REST de Firestore](https://firebase.google.com/docs/firestore/reference/rest/v1/Write).
