# Contrato del portal · versión 1 simplificada · adaptación HRL CORE

Fecha: 19 de noviembre de 2026. Partes: HRL CORE (Héctor López Cano) y portal web (Víctor).

**Base elegida por Héctor:** propuesta simplificada de Víctor, con proyección pública
separada del modelo local. HRL CORE conserva sus funciones, permisos y reglas de dinero.
Esta adaptación local corrige pagos con saldo a favor y exige vencimiento al registrar
entrega. No acredita aceptación ni despliegue de esos ajustes en el portal.

Este es el documento de trabajo de HRL CORE para la versión 1. Los documentos en
`docs/backlog/contrato-portal-propuesta-v1.md` y `docs/CONTRATO-FIRESTORE.md` se
conservan como antecedentes. Los pendientes de integración están en la sección 13.

Fuentes consultadas en la rama `feat/firebase-seguimiento`:

- [Contrato del portal](https://github.com/VictorJose2073/Proyecto-estadias/blob/feat/firebase-seguimiento/docs/CONTRATO-FIRESTORE.md), blob `84890ec05ea7d3790d7462739846d77ce49365a3`.
- [Reglas del portal](https://github.com/VictorJose2073/Proyecto-estadias/blob/feat/firebase-seguimiento/firestore.rules), blob `e672aa9906c1bc10e738d66c34c953b36b59c1c0`.
- [Huella de referencia](https://github.com/VictorJose2073/Proyecto-estadias/blob/feat/firebase-seguimiento/tests/reglas/huella.js).

Las reglas están en `firestore.rules` y sus pruebas en `tests/reglas/` (repositorio del portal).

## 1. Decisiones tomadas

| Tema | Decisión |
|---|---|
| Arquitectura | HRL CORE escribe directo a Cloud Firestore con Firebase Authentication, sin Cloud Functions (plan Spark). Se conservan la cola local y los reintentos. |
| Folio interno | No se publica ni se muestra al cliente. |
| Pagos | Se muestran moneda, total, cobrado y pendiente, en centavos enteros. Sin historial de cobros ni costos internos. |
| Publicación | Automática por defecto; cada orden puede cambiarse a manual (`seguimientos.modo_publicacion`). |
| Etapas | Se publican también las personalizadas, en su orden y con su estado de completada. |
| Acceso | Enlace/QR y un código que el cliente puede teclear; aleatorio e independiente del folio. |
| Vencimiento | Siete días después de registrar la **entrega efectiva**. Terminar la reparación o marcarla lista para entrega no inicia el plazo. |

## 2. Componentes

- Escritura: **HRL CORE / SQLite → cola local → API REST de Firestore → `seguimiento/{idPublico}`**.
- Consulta: **enlace, QR o código tecleado → portal web → lectura de ese documento en tiempo real**.
- Un solo proyecto de Firebase. Una sola dirección: HRL CORE es la autoridad; la web solo lee.
- Un único emisor activo por taller: una cuenta de Firebase Authentication de la instalación. Las
  reglas solo aceptan escrituras de su UID. No se usa Admin SDK ni cuenta de servicio.

## 3. Código de acceso (`idPublico`)

- Es el ID del documento. Enlace: `https://<portal>/seguimiento/<idPublico>`; el QR contiene ese
  enlace; el cliente también puede teclear el código en el portal.
- Aleatorio, con generador criptográfico, independiente del folio. Se genera al publicar por
  primera vez y se imprime en el comprobante.
- Formato de trabajo: `XXXX-XXXX-XXXX`, alfabeto `ABCDEFGHJKMNPQRSTUVWXYZ23456789` (≈ 59 bits).
  El portal ignora mayúsculas/minúsculas y espacios al teclear. *Pendiente:* longitud y formato definitivos.
- Quien tiene el código puede ver el seguimiento hasta que venza o se despublique.

## 4. Documento `seguimiento/{idPublico}`

Se construye con lista de campos permitidos; nunca se serializa la orden completa. Las reglas
rechazan cualquier campo fuera de esta tabla.

| Campo | Tipo | Oblig. | Origen en HRL CORE / validación |
|---|---|---|---|
| `versionContrato` | int = 1 | sí | constante |
| `idPublico` | string | sí | igual al ID del documento |
| `revision` | int ≥ 1 | sí | revisión de publicación (§6) |
| `publicado` | bool | sí | `false` = retirado (§7) |
| `huella` | string, 64 hex | sí | SHA-256 del contenido (§6) |
| `actualizadoEn` | timestamp | sí | hora del servidor (`REQUEST_TIME`); las reglas exigen `== request.time` |
| `recibidoEn` | timestamp | no | fecha de recepción |
| `entregaEstimada` | timestamp | no | si existe |
| `entregadoEn` | timestamp | no | entrega efectiva al cliente |
| `accesoHasta` | timestamp | no | exactamente `entregadoEn + 7 días`; prohibido sin `entregadoEn` |
| `equipo` | map | sí | `{tipo, marca, modelo}`, exactamente esas claves; `tipo` no vacío ≤ 60, `marca` ≤ 60, `modelo` ≤ 100; sin número de serie |
| `estadoTecnico` | string | sí | no vacío, ≤ 100; texto del catálogo de HRL CORE |
| `etapas` | list ≤ 30 | sí | `orden_etapas` por `posicion`, incluidas las personalizadas |
| `eventos` | list ≤ 200 | sección | solo `avances` con `visible_cliente = 1` |
| `notaPublica` | string ≤ 4000 | sección | `seguimientos.nota_publica` |
| `garantias` | list ≤ 20 | sección | tabla `garantias` |
| `pago` | map | sección | ver abajo |

`pago`: exactamente `{moneda, totalCentavos, cobradoCentavos, pendienteCentavos}`.
`moneda` es un código ISO 4217 de tres letras mayúsculas (p. ej. `MXN`); los tres importes son
enteros ≥ 0 y `pendienteCentavos == max(totalCentavos − cobradoCentavos, 0)`.
`cobradoCentavos` corresponde al cobro neto real de `vista_saldos_orden`, incluidos
ajustes y devoluciones. Puede superar al total cuando existe saldo a favor: total
60000, cobrado 75000, pendiente 0. No se recorta el cobrado ni se publica el detalle
del saldo a favor. Un importe decimal, un pendiente inconsistente o un campo extra
(p. ej. historial de abonos) se rechaza. Los enteros de transporte no superan
9007199254740991, para conservar exactitud en JavaScript.

**Nunca publicar:** folio interno; contraseña del equipo; número de serie; notas internas;
contraseñas, hashes o claves de usuarios; clave maestra; llaves de cifrado/DPAPI; credenciales de
Firebase; rutas locales; respaldos; auditoría interna; costos, gastos, utilidad e historial de
cobros; identidad o permisos del personal; nombre, teléfono, correo o domicilio del cliente;
fotografías, videos y documentos.

## 5. Secciones ocultas y elementos de listas

**Representación única:**

- **Sección oculta = campo omitido.** Aplica a `eventos`, `notaPublica`, `garantias` y `pago`.
- **Lista vacía (`[]`) = sección visible sin elementos.** El portal muestra la sección con un texto
  como «Aún no hay actividad».
- `equipo`, `estadoTecnico` y `etapas` son obligatorios y no se pueden ocultar.
- Como cada envío reemplaza el documento completo, ocultar una sección elimina sus datos remotos
  en la siguiente revisión.

**Elementos** (las reglas validan tipo y tamaño de la lista, pero no recorren sus elementos):

| Lista | Elemento | Validación |
|---|---|---|
| `etapas` | `{nombre, completada, completadaEn?}` | `nombre` string no vacío ≤ 100; `completada` bool; `completadaEn` timestamp solo si `completada` |
| `eventos` | `{fecha, titulo, detalle}` | `fecha` timestamp; `titulo` no vacío ≤ 200; `detalle` ≤ 2000 (puede ser `""`) |
| `garantias` | `{cobertura, dias}` | `cobertura` no vacía ≤ 200; `dias` int ≥ 0 |

Responsables de esta validación:

1. **HRL CORE** la aplica al construir la proyección, antes de encolar. Un elemento inválido deja
   la revisión con error visible y no se envía.
2. **El portal** ignora los elementos que no cumplan el formato, sin dejar de mostrar el resto.
3. **Pruebas** de contrato en ambos lados con los casos de esta tabla.

## 6. Revisiones y huella

- `revision` es un entero que HRL CORE incrementa en cada publicación de la orden, incluida la
  despublicación. No se comparan relojes. Cada revisión pendiente es inmutable.
- Las reglas aceptan una escritura solo si `revision` es **mayor** que la guardada.
- `huella` = SHA-256 en hexadecimal minúsculo del **JSON canónico** del documento sin
  `actualizadoEn` ni `huella`:
  - claves de los objetos ordenadas (orden de código UTF-16, como `Array.prototype.sort`), en
    todos los niveles; las listas conservan su orden;
  - sin espacios; texto en UTF-8 sin escapar caracteres no ASCII;
  - fechas como texto ISO 8601 en UTC con milisegundos (`2026-11-19T17:30:00.000Z`);
  - enteros como números JSON.
- Implementación de referencia: `tests/reglas/huella.js`. Vector de prueba:

```
JSON canónico:
{"equipo":{"marca":"HP","modelo":"Pavilion 15","tipo":"Laptop"},"estadoTecnico":"En diagnóstico","etapas":[{"completada":false,"nombre":"Diagnóstico"}],"idPublico":"ABCD-EFGH-JKMN","pago":{"cobradoCentavos":0,"moneda":"MXN","pendienteCentavos":0,"totalCentavos":0},"publicado":true,"recibidoEn":"2026-11-19T17:30:00.000Z","revision":1,"versionContrato":1}

SHA-256:
67a5a5d3280cacb5f2f5300869ac8dd41804508a82d3784b1a05f3f851646dab
```

**Reconciliación tras un rechazo o una respuesta perdida.** HRL CORE lee el documento remoto
(el emisor puede leerlo aunque haya vencido) y compara con la revisión que intentó enviar:

| Remoto | Resultado |
|---|---|
| misma `revision` y misma `huella` | **Confirmada** (llegó; se perdió la respuesta). |
| misma `revision` y otra `huella` | **Conflicto**: error visible; no se sobrescribe ni se marca confirmada. |
| `revision` mayor | **Superada**: se descarta; la cola sigue con la más reciente. |
| `revision` menor o no existe | El rechazo fue por contenido: error visible, sin reintento infinito. |

**Restaurar un respaldo o cambiar de equipo:** la política de reconciliación sigue
pendiente. Leer la revisión remota evita retroceder el contador, pero no autoriza
republicar contenido desactualizado. La reanudación automática requiere definir
ese procedimiento.

## 7. Despublicación, revocación y vencimiento

- **Despublicar** = nueva revisión con solo `{versionContrato, idPublico, revision, publicado:
  false, actualizadoEn, huella}` y, si existen, `entregadoEn` y `accesoHasta`. No se borra: si se
  borrara, un envío atrasado podría recrear la orden. Las reglas prohíben borrar.
- El portal, al recibir `publicado: false`, retira los datos de la pantalla aunque esté abierta.
- **Rotar el código** (p. ej. si se entregó a otra persona): nuevo `idPublico`, publicar ahí y
  despublicar el anterior.
- **Vencimiento:** al registrar la entrega efectiva, HRL CORE publica `entregadoEn` y
  `accesoHasta = entregadoEn + 7 días`. Las reglas exigen exactamente ese plazo. Después de
  `accesoHasta`, el cliente ya no puede leer; el emisor sí, para reconciliar.
- `entregadoEn` y `accesoHasta` aparecen juntos o se omiten ambos. No se admite una
  entrega publicada sin vencimiento.
- Sin internet, la despublicación queda en la cola y HRL CORE muestra «despublicación pendiente».
  Nada recupera copias o capturas que el cliente ya tomó.

## 8. Publicación automática y manual

- **Automática** (por defecto): cada cambio autorizado genera una revisión pendiente.
- **Manual**: los cambios se acumulan y se genera la revisión cuando el usuario publica.
- Pasar de automática a manual no despublica; despublicar es una acción explícita.

## 9. Envío y reintentos (HRL CORE)

- Al confirmar un cambio autorizado, el servicio guarda la proyección pendiente y la auditoría en
  la **misma transacción** SQLite. La red trabaja después y nunca bloquea el guardado local.
- Por orden, solo se envía la revisión pendiente más reciente; una nueva deja obsoletas las
  anteriores no enviadas. Un envío a la vez por orden.
- Inicio de sesión del emisor con Identity Toolkit (`accounts:signInWithPassword`); se guarda el
  *refresh token* protegido localmente (p. ej. DPAPI), no la contraseña en texto plano.
- Envío con `documents:commit`: escritura `update` del documento completo (sin `updateMask`) y
  transformación `actualizadoEn = REQUEST_TIME`.

| Resultado | Acción |
|---|---|
| Éxito | Confirmada con el `updateTime` del servidor. |
| Sin red, timeout, `UNAVAILABLE`, 5xx, 429 | Reintentar con espera creciente y variación aleatoria (2 s, 4 s, 8 s… hasta 5 min). |
| `UNAUTHENTICATED` | Renovar el token; si falla, pausar la cola hasta reautorizar el emisor. |
| `PERMISSION_DENIED` | Reconciliar según la tabla de §6. |
| `INVALID_ARGUMENT` | Error visible para corregir; sin reintento. |

## 10. Portal web

- Lee solo `seguimiento/{idPublico}` con `onSnapshot`; no puede listar ni escribir.
- Muestra `actualizadoEn` como «última actualización» y avisa si está mostrando datos en caché
  sin conexión.
- No muestra datos si el documento no existe, venció o tiene `publicado: false`.
- Calcula el progreso como etapas completadas entre etapas totales.

## 11. Reglas de seguridad (resumen)

- `get`: cualquiera mientras no venza; el emisor siempre. `list`: nadie.
- `create`/`update`: solo el emisor, campos permitidos, tipos y tamaños, `pago` consistente,
  vencimiento de 7 días, hora del servidor, huella con formato válido y revisión creciente.
- `delete`: nadie.

## 12. Pruebas de las reglas

El borrador del portal informa 35 casos en `tests/reglas/reglas.test.js` al
19/11/2026. Esa suite no se ha ejecutado en HRL CORE y no verifica los ajustes
locales de esta adaptación.
Requieren Java 21 y `firebase-tools`: `cd tests/reglas && npm install && npm test`.

Cubren: lectura, listado, escritura sin sesión o con otra cuenta, `folio` y datos personales,
`idPublico` distinto, hora del equipo, secciones omitidas y listas vacías, equipo incompleto o con
serie, pago decimal, descuadrado, con moneda inválida o con historial, revisiones
mayor/igual/menor, reconciliación por huella, despublicación válida e inválida, borrado y
vencimiento (plazo exacto, sin entrega, lectura del cliente antes/después y del emisor después).

Se deben agregar casos de saldo a favor permitido y entrega sin vencimiento rechazada.

### Primera fase local: proyección pública

`src-tauri/src/domain/portal.rs` construye un documento nuevo con campos públicos
explícitos. Valida sus elementos, excluye avances privados, omite secciones ocultas,
preserva etapas personalizadas y calcula el vencimiento desde un instante real de
entrega. Sus tipos encapsulan datos validados y no reciben clientes, folios,
contraseñas, personal ni libros completos.

La huella coincide con el vector JavaScript del portal. `actualizadoEn` queda fuera
del objeto local; el futuro emisor lo agregará como transformación `REQUEST_TIME`.
El módulo no almacena JSON de negocio, no escribe en SQLite y no transmite datos.

La fase de persistencia incorpora la migración 006 y conecta las escrituras de
órdenes, reparaciones y pagos con la cola. Las órdenes nuevas usan publicación
automática y pago visible por defecto; las preferencias ya guardadas se conservan.
La captura y su auditoría comparten la transacción del cambio local. Un dato que
excede los límites del portal deja un error de publicación sin impedir el trabajo
local. Un fallo de almacenamiento o auditoría revierte toda la transacción.

Los encabezados, etapas, eventos y garantías de cada revisión son inmutables y
normalizados. No se guarda JSON ni el saldo calculado: el pago se reconstruye con
el precio de esa revisión y los identificadores máximos de los libros inmutables
en el momento de captura. Los estados de envío se guardan por separado.

Las fechas de calendario de recepción, entrega estimada y avances se transportan
como medianoche UTC de ese día; expresan una fecha, no una hora real del evento.
La entrega efectiva usa un instante UTC registrado al marcar la entrega, sin
atribuir horas a entregas históricas. El código sigue el formato de trabajo del
portal: doce caracteres criptográficos, en tres grupos de cuatro.

## 13. Pendientes para cerrar el contrato

1. Longitud y formato definitivos del código de acceso.
2. Catálogo real de estados técnicos de HRL CORE (`src/data/orderStatuses.ts` u otro).
3. Responsables de implementación de cada parte.
4. Fecha de la prueba de punta a punta.
5. Proyecto de Firebase: configuración web y UID del emisor (credenciales por privado).
6. Incorporar en el portal los ajustes de pagos y vencimiento de esta adaptación.
7. Procedimiento autorizado de reconciliación tras restaurar un respaldo o cambiar de equipo.
8. Registrar un instante UTC de entrega: hoy `ordenes.fecha_entrega` conserva solo
   la fecha (`YYYY-MM-DD`). No se inventará una hora para órdenes ya entregadas.
9. Probar vencimiento con el portal abierto y sin conexión; el portal debe retirar
   los datos de pantalla al vencer, sin depender únicamente de recibir otro cambio.
