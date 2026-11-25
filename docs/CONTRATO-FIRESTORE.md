# Contrato del portal · versión 1 simplificada (propuesta del portal web)

Fecha: 19 de noviembre de 2026. **Borrador para unificar con
`HRL-CORE/docs/backlog/contrato-portal-propuesta-v1.md`. No aprobado ni implementado.**

Este documento conserva los campos, la lista de exclusiones y el criterio de cola de la propuesta
v1 de HRL CORE, y propone cambiar dos piezas: **el envío** (sin API intermedia) y **el acceso del
cliente** (sin canje de enlace por sesión). Al final se comparan ambas versiones y se listan las
decisiones pendientes.

## 1. Por qué una versión simplificada

La propuesta v1 usa una API HTTPS en Cloud Functions, tokens personalizados y sesiones por
orden. Cloud Functions exige el plan **Blaze** de Firebase (facturación con tarjeta), y la API,
los tokens y su despliegue no caben antes del cierre del Sprint 5 (27 de noviembre de 2026).

Esta versión funciona en el plan gratuito (**Spark**): HRL CORE escribe directamente en Cloud
Firestore y las **reglas de seguridad** cumplen el papel de validación que la v1 asignaba a la
API. Si después se necesita la API, el documento público no cambia; solo cambia quién escribe.

## 2. Componentes

Flujo de escritura: **HRL CORE / SQLite → cola local → API REST de Firestore → `seguimiento/{idPublico}`**.

Flujo de consulta: **enlace o QR con `idPublico` → portal web → lectura de ese documento (en tiempo real)**.

- Un solo proyecto de Firebase, base Cloud Firestore, región por definir.
- Una sola dirección: HRL CORE es la autoridad; la web solo lee.
- Un único emisor activo por taller: una cuenta de Firebase Authentication (correo y contraseña)
  creada para la instalación del taller. Las reglas solo aceptan escrituras de su UID.
- No se usa Admin SDK ni cuenta de servicio: todas las escrituras pasan por las reglas.

## 3. Identificador público (`idPublico`)

- Es el ID del documento y va en el enlace o QR: `https://<portal>/seguimiento/<idPublico>`.
- Aleatorio, generado por HRL CORE con un generador criptográfico, **independiente del folio**.
- Formato propuesto: `XXXX-XXXX-XXXX` con el alfabeto `ABCDEFGHJKMNPQRSTUVWXYZ23456789`
  (≈ 59 bits). Se puede teclear en el portal si el cliente no usa el QR.
  *Pendiente de revisión:* si solo se usa enlace/QR, puede subir a 22 caracteres base64url (128 bits).
- Quien tiene el enlace puede ver el seguimiento (igual que en la v1). La alternativa con cuenta
  del cliente queda fuera de esta versión.

## 4. Documento público `seguimiento/{idPublico}`

Se construye con una lista de campos permitidos; nunca se serializa la orden completa. Las reglas
rechazan cualquier campo que no esté en esta tabla.

| Campo | Tipo | Oblig. | Origen en HRL CORE |
|---|---|---|---|
| `versionContrato` | int = 1 | sí | constante |
| `idPublico` | string | sí | igual al ID del documento |
| `revision` | int ≥ 1 | sí | revisión de publicación, monotónica por orden |
| `publicado` | bool | sí | `false` = documento retirado (ver §6) |
| `actualizadoEn` | timestamp | sí | **hora del servidor** (`REQUEST_TIME`), no del reloj de Windows |
| `accesoHasta` | timestamp | no | vencimiento del enlace, si se decide usarlo |
| `equipo` | map `{tipo, marca, modelo}` | sí | datos del equipo; sin número de serie |
| `estadoTecnico` | string | sí | estado actual de la orden, como texto |
| `etapas` | list de `{nombre, completada, completadaEn?}` | sí | `orden_etapas` en su orden de `posicion` |
| `eventos` | list de `{fecha, titulo, detalle}` | sí | solo `avances` con `visible_cliente = 1` |
| `notaPublica` | string | no | `seguimientos.nota_publica`, si la sección está habilitada |
| `garantias` | list de `{cobertura, dias}` | no | `garantias`, si la sección está habilitada |

Ocultar una sección en HRL CORE quita sus campos en la siguiente revisión: cada envío **reemplaza
el documento completo**, así que no quedan restos de campos que dejaron de autorizarse.

El portal calcula el progreso como etapas completadas entre etapas totales; así funcionan también
las etapas con nombre personalizado.

**Pendientes, fuera de la lista hasta que se decidan** (si se envían, las reglas los rechazan):
`folio` y `pago` (`{moneda, totalCentavos, cobradoCentavos, pendienteCentavos}`, enteros).

**Nunca publicar** (igual que la v1): contraseña del equipo; notas internas; contraseñas, hashes o
claves de usuarios; clave maestra; llaves de cifrado/DPAPI; credenciales de Firebase; rutas
locales; respaldos; auditoría interna; costos, gastos y utilidad; identidad o permisos del
personal; nombre, teléfono, correo o domicilio del cliente; fotografías, videos y documentos.

## 5. Revisiones

- `revision` es un entero que HRL CORE incrementa en cada publicación de la orden (incluida la
  despublicación). No se comparan relojes.
- Las reglas aceptan una escritura solo si `revision` es **mayor** que la guardada. Un envío
  atrasado o duplicado se rechaza y no reemplaza uno más reciente.
- Cada revisión pendiente es inmutable: el reintento envía exactamente el mismo contenido.
- **Restaurar un respaldo o cambiar de equipo** puede dejar la revisión local por debajo de la
  remota. Antes de reanudar la cola, HRL CORE lee el documento remoto y continúa desde
  `revisionRemota + 1`. *Pendiente:* quién autoriza esa reconciliación y qué pasa con datos ya
  publicados que el respaldo no tiene.

## 6. Revocación y despublicación

- **Despublicar** = nueva revisión con solo los campos de control:
  `{versionContrato, idPublico, revision, publicado: false, actualizadoEn}`. Las reglas exigen que
  un documento con `publicado: false` no traiga datos de seguimiento.
- Se usa este documento "retirado" en lugar de borrar: si se borrara, un envío atrasado podría
  volver a crear la orden. Las reglas prohíben borrar desde HRL CORE.
- El portal, al recibir `publicado: false`, quita los datos de la pantalla y muestra que el
  seguimiento ya no está disponible. Como escucha en tiempo real, ocurre aunque la página esté abierta.
- **Rotar el enlace** (p. ej. si se envió a la persona equivocada): HRL CORE genera un `idPublico`
  nuevo, publica ahí y despublica el anterior. El enlace viejo deja de mostrar datos.
- **Vencimiento opcional:** si se define `accesoHasta`, las reglas dejan de permitir la lectura
  después de esa fecha. *Pendiente:* si se usa y con qué plazo (p. ej. fin de la garantía).
- Sin internet, la despublicación queda en la cola y HRL CORE muestra «despublicación pendiente»;
  no se promete retiro remoto inmediato. Nada recupera copias o capturas que el cliente ya tomó.

## 7. Envío y reintentos (HRL CORE)

Igual que la v1 en lo local:

- Al confirmar un cambio autorizado, el servicio guarda la proyección pendiente y la auditoría en
  la **misma transacción** SQLite. La red trabaja después y nunca bloquea el guardado local.
- Por orden, solo se envía la revisión pendiente más reciente; una revisión nueva deja obsoletas
  las pendientes anteriores no enviadas (también si oculta datos).
- Un envío a la vez por orden.

Envío propuesto, desde Rust con la API REST:

1. Inicio de sesión del emisor con Identity Toolkit (`accounts:signInWithPassword`); se guarda el
   *refresh token* protegido localmente (p. ej. DPAPI), no la contraseña en texto plano.
2. `documents:commit` con una escritura `update` del documento completo (sin `updateMask`, para
   reemplazarlo) y una transformación `actualizadoEn = REQUEST_TIME`.

Respuesta y clasificación de errores:

| Resultado | Acción |
|---|---|
| Éxito | Marcar la revisión como confirmada con el `updateTime` del servidor. |
| Sin red, timeout, `UNAVAILABLE`, 5xx, 429 | Conservar y reintentar con espera creciente y variación aleatoria (p. ej. 2 s, 4 s, 8 s… hasta 5 min). |
| `UNAUTHENTICATED` | Renovar el token; si falla, pausar la cola hasta reautorizar el emisor. |
| `PERMISSION_DENIED` | Leer el documento remoto. Si su `revision` ≥ la enviada, la revisión ya llegó (respuesta perdida) o fue superada: marcar confirmada/obsoleta. Si no, el contenido es inválido: error visible, sin reintento infinito. |
| `INVALID_ARGUMENT` | Error visible para corregir; sin reintento. |

Esto cubre el caso de la v1 «respuesta perdida después de guardar» sin una tabla de solicitudes en
el servidor: la revisión monotónica hace la idempotencia.

## 8. Portal web

- Lee solo `seguimiento/{idPublico}` con `onSnapshot`; no puede listar ni escribir.
- Muestra la fecha de la última confirmación (`actualizadoEn`) y avisa si está mostrando datos en
  caché sin conexión, sin presentarlos como recién sincronizados.
- Si el documento no existe, venció o tiene `publicado: false`, no muestra datos.

## 9. Comparación con la propuesta v1

| Tema | v1 (HRL CORE) | v1 simplificada (esta) |
|---|---|---|
| Envío | API HTTPS en Cloud Functions + Admin SDK | REST de Firestore + reglas |
| Plan de Firebase | Blaze (tarjeta) | Spark (gratis) |
| Acceso del cliente | Secreto en el enlace → canje por sesión de una orden | `idPublico` aleatorio en el enlace → lectura directa |
| Revocación | Invalidar secreto/sesión | Documento retirado + rotación de `idPublico` |
| Vencimiento | Por sesión | `accesoHasta` opcional |
| Idempotencia | Tabla de solicitudes en el servidor | `revision` estrictamente creciente |
| Validación de contenido | Código de la API | Reglas: campos permitidos, tipos, tamaños |
| Límite conocido | — | Las reglas no recorren listas: el contenido de cada evento/etapa se valida en HRL CORE y en pruebas |
| Campos, exclusiones, cola local | — | Sin cambios |

## 10. Decisiones pendientes

1. Aprobar la versión (completa o simplificada) y registrar responsable y fecha.
2. Mostrar o no el **folio**.
3. Mostrar o no el **resumen de pago** y en qué secciones.
4. **Modo de publicación** (`manual` / `auto`) y quién lo configura.
5. **Catálogo** de estados técnicos y etapas que se publican (incluidas las personalizadas).
6. Longitud del `idPublico` y si se usa `accesoHasta`.
7. Reconciliación tras restaurar respaldo o cambiar de equipo.
8. Región, conservación de documentos retirados, responsable de credenciales y despliegue.

## 11. Pruebas mínimas (emulador de Firestore)

- Leer un `idPublico` existente: permitido. Listar la colección: rechazado.
- Escribir sin sesión o con otra cuenta: rechazado.
- Campo no permitido (p. ej. `folio`, `telefono`): rechazado.
- `revision` igual o menor a la guardada: rechazado.
- Despublicar con datos de seguimiento: rechazado. Borrar: rechazado.
- Leer después de `accesoHasta`: rechazado.
