# Propuesta de contrato del portal · versión 1

Fecha: 17 de noviembre de 2026. Referencias: HRLP-55–59 (HRL-55–59 originales), epic HRLP-7.

**Borrador para revisión. No aprobado ni implementado.** El usuario confirmó que todavía no existe contrato y pidió una idea para enviar los datos y hacerlos visibles en la web. Este documento propone decisiones; no afirma que ya estén conectados Firebase o el portal. No modifica el backlog original ni las reglas vigentes.

## Lo que haría el cliente

El taller entrega un enlace o QR privado de seguimiento. Al abrirlo, el cliente ve su equipo, etapa actual, avances marcados como públicos, nota pública y garantías habilitadas. Los cambios aparecen mientras la página está abierta. Si se autoriza mostrar pagos, ve total, cobrado y pendiente en la moneda de la orden. No puede editar la orden, registrar cobros ni consultar otras órdenes.

Ejemplo: el técnico guarda «Diagnóstico terminado» con visibilidad pública. La app conserva el avance localmente y lo coloca en la cola. Con Internet, lo envía; la web actualiza el seguimiento. Sin Internet, el taller sigue trabajando y la web muestra la última actualización recibida, con su fecha.

## Componentes propuestos

Flujo de escritura: **HRL CORE / SQLite → cola local → API HTTPS → Firestore**.

Flujo de consulta: **enlace privado → API de acceso → sesión limitada → documento de seguimiento en Firestore → web**.

La API externa puede implementarse con Cloud Functions HTTP, que admite manejadores de petición y rutas. La URL real y su despliegue deben acordarse con el responsable; no se usa una URL supuesta. [Funciones HTTP de Firebase](https://firebase.google.com/docs/functions/http-events).

La primera versión sería de una sola dirección: la app del taller es la autoridad de los datos y la web los consulta. No habría ediciones remotas ni conciliación de órdenes entre varios equipos escritores en esta versión. Esto simplifica los conflictos y debe aprobarse expresamente.

## Campos y visibilidad propuestos

Se construye un objeto público con campos permitidos; no se serializa la orden completa para quitar después algunos secretos. Los campos desconocidos se rechazan en la API.

| Datos | Propuesta para versión 1 |
|---|---|
| Identificación | Identificador público aleatorio, folio, tipo/marca/modelo del equipo. No número de serie por defecto. |
| Estado | Estado técnico y etapa actual; recepción, entrega estimada y entrega efectiva si existen. No convertir automáticamente estado técnico en entrega o pago. |
| Línea de tiempo | Solo eventos con `public=true`, y solo si la visibilidad de línea de tiempo está habilitada. |
| Nota | Solo `public_note`, si está habilitada la sección de notas públicas. |
| Garantías | Nombre, días y fechas aplicables, cuando la visibilidad de garantías lo permita. |
| Resumen de pago | Solo si está habilitado: moneda y total/cobrado/pendiente calculados; importes en centavos enteros. No historial completo ni utilidad. |
| Marca del taller | Nombre comercial y contacto público seleccionados; el logo requeriría un recurso público independiente aprobado. |
| Control | Versión de contrato, revisión de publicación y fecha de confirmación del servidor. |

**Nunca publicar:** contraseña del equipo; notas internas; contraseñas, hashes o claves de usuarios; clave maestra; llaves de cifrado/DPAPI; credenciales de Firebase; rutas locales; respaldos; auditoría interna; costos/gastos/utilidad; identidad o permisos del personal; nombre, teléfono, correo o domicilio del cliente. Los secretos del enlace no forman parte del documento público.

Propuesta de alcance inicial: no enviar fotografías, videos, documentos ni listas de refacciones. Requerirían un contrato adicional de selección, privacidad, acceso y retiro. Se conservan localmente las funciones vigentes.

Ocultar una sección debe eliminar también sus datos del documento remoto; no basta con ocultar un componente de la página. Cada envío reemplaza la proyección pública completa de esa revisión, sin conservar campos que dejaron de estar autorizados.

## Acceso propuesto

El enlace usa un secreto aleatorio de alta entropía, independiente del folio. Se recomienda transportarlo en el fragmento del enlace y canjearlo mediante POST; no incluirlo en registros, analítica ni mensajes de error. El servidor conserva su hash y permite revocarlo o rotarlo. Quien recibe el enlace podrá ver el seguimiento: el responsable debe aceptar este modelo o elegir acceso con cuenta del cliente.

La API canjea el enlace válido por una sesión Firebase limitada a una orden. Las reglas verifican identidad, orden, vigencia, publicación activa y versión del permiso. Impiden listar otras órdenes y toda escritura desde el navegador. Los tokens personalizados permiten integrar una autenticación propia con Firebase; el esquema exacto de claims y expiración se implementará y probará después de aprobar el contrato. [Tokens personalizados](https://firebase.google.com/docs/auth/admin/create-custom-tokens).

El emisor de HRL CORE utiliza una identidad de publicación vinculada al taller/instalación. La API valida el ID token y el permiso de publicar; no confía en un `tallerId` que llegue en el cuerpo. No se reutilizan ni envían las contraseñas locales del taller. El alta, revocación y almacenamiento de la credencial del emisor quedan por definir. [Verificación de ID tokens](https://firebase.google.com/docs/auth/admin/verify-id-tokens).

La cuenta de servicio/Admin SDK permanece en el servidor. Como las bibliotecas de servidor omiten las reglas Firestore, la API debe validar autorización, pertenencia al taller y contenido en cada escritura; las reglas del cliente no la sustituyen. [Seguridad de Firestore](https://firebase.google.com/docs/firestore/security/get-started).

## Mensaje de publicación propuesto

Ruta ilustrativa: `POST /v1/publicaciones`. No corresponde a un servicio existente. Cabeceras: token de publicación e identificador de solicitud. El servidor valida que la solicitud pertenezca al taller autenticado.

```json
{
  "versionContrato": 1,
  "idSolicitud": "uuid-del-envio",
  "idPublico": "identificador-aleatorio-de-la-orden",
  "revisionPublicacion": 4,
  "accion": "publicar",
  "seguimiento": {
    "folio": "ORD-001",
    "equipo": {"tipo": "Laptop", "marca": "Marca", "modelo": "Modelo"},
    "estadoTecnico": "En diagnóstico",
    "etapaActual": "Diagnóstico",
    "notaPublica": "Diagnóstico terminado.",
    "eventos": [],
    "garantias": []
  }
}
```

Los valores son ejemplos, no datos reales ni nuevos catálogos. Si se muestra pago, se añade un objeto explícito con moneda e importes enteros. `accion=despublicar` transmite control, sin datos de seguimiento. La respuesta confirmada incluye `idSolicitud`, revisión aceptada y fecha del servidor; una conexión exitosa sin confirmación válida no marca el envío como terminado.

## Cola, reintentos y conflictos

Al confirmar localmente un cambio autorizado, el servicio guarda la proyección pendiente y la auditoría en la misma transacción. La nueva persistencia debe usar migración nueva y tablas normalizadas, con encabezados, eventos y garantías separados; no JSON de negocio en SQLite. El JSON del ejemplo es el formato de transporte.

Cada revisión pendiente conserva datos inmutables para repetir exactamente el mismo envío. El proceso de red trabaja después de la transacción local y nunca espera Internet para guardar una reparación. No agrega un bloqueo de red a `commands/`; las reglas y la orquestación siguen en servicios, el SQL en repositorios.

El servidor registra solicitud y actualización de seguimiento atómicamente. Mismo identificador y mismo contenido devuelve la confirmación previa; mismo identificador con otro contenido se rechaza. Las transacciones de Firestore permiten aplicar conjuntamente varias escrituras o ninguna. [Transacciones de Firestore](https://firebase.google.com/docs/firestore/manage-data/transactions).

La revisión de publicación es monotónica por orden y emisor autorizado, sin comparar relojes de Windows. Un envío atrasado no reemplaza una revisión más reciente. Los conflictos no se resuelven sobrescribiendo a ciegas. Restaurar respaldos o cambiar de equipo exige reconciliar revisión y emisor antes de reanudar la cola; ese procedimiento es una decisión pendiente.

Ante timeout, caída de red o error transitorio se conserva la solicitud y se reintenta con espera creciente y variación aleatoria. Ante credencial inválida se pausa hasta reautorizar. Un contenido inválido queda con error visible para corregir, sin reintento infinito. Se envía por orden y se evita concurrencia sobre la misma revisión.

Antes de enviar se revalida la autorización de publicación. Una revisión nueva que oculta datos invalida las pendientes anteriores todavía no enviadas. Si ya hay un envío en curso, no se promete revocarlo antes de que el servidor lo procese: se envía después la revisión restrictiva y se informa de la confirmación real.

## Actualización de la web y despublicación

Tras autenticar el enlace, la web escucha únicamente el documento autorizado. Firestore ofrece suscripciones a cambios mediante `onSnapshot`. La pantalla muestra fecha de última confirmación y el estado de conexión; no presenta datos en caché como recién sincronizados. [Actualizaciones en tiempo real](https://firebase.google.com/docs/firestore/query-data/listen).

Despublicar crea una revisión nueva, deshabilita el acceso remoto e invalida sesiones/enlaces según la política de acceso. La pantalla elimina los datos visibles cuando recibe la revocación. Estando el taller sin Internet, la app debe indicar «despublicación pendiente»: no puede prometer retirada remota inmediata. Ningún sistema recupera copias o capturas ya obtenidas por el cliente.

## Pruebas necesarias y orden de trabajo

| Jira | Entrega verificable |
|---|---|
| HRLP-55 | Aprobar este contrato versionado y registrar responsable, fecha y decisiones. Sigue pendiente la aprobación. |
| HRLP-56 | Proyección autorizada y cola persistente: evento privado excluido, reinicio sin pérdida, ocultación invalida pendientes y operación sin Internet. |
| HRLP-57 | Pruebas de contrato: campos prohibidos, desconocidos, permisos entre órdenes/talleres, centavos, duplicados, conflictos y revisiones atrasadas. Usar emulador para reglas y API. |
| HRLP-58 | API real, credencial de instalación, envío confirmado/reintentos y web suscrita. Probar pérdida de respuesta después de guardar, reconexión y actualización observable por el cliente. |
| HRLP-59 | Control por orden, vista de lo publicado y despublicación con confirmación; un enlace revocado deja de obtener datos. |

No se consideran Hecho HRLP-56–59 por disponer de este documento. La integración debe probarse contra un entorno de prueba antes de producción.

## Decisiones para aprobar

1. Responsable del portal y repositorio/URL de su implementación; proyecto Firebase y entorno de prueba. **No verificado:** no se ha inspeccionado el portal ni una configuración remota.
2. Aceptar publicación en una sola dirección y un único emisor activo por taller para versión 1.
3. Aceptar enlace/QR privado canjeable por sesión de una orden; definir vencimiento, rotación y quién recibe el enlace. Alternativa: cuenta propia del cliente.
4. Aprobar lista de campos, resumen de pago opcional y exclusión inicial de medios/series/datos personales.
5. Definir quién configura el modo automático/manual, qué sucede con datos ya publicados al restaurar un respaldo y cómo transferir el emisor a otra instalación.
6. Definir conservación de datos, región, presupuesto y alertas, responsable de credenciales/despliegue y procedimiento de incidentes. No se ha creado infraestructura ni generado cargos.

La aprobación debe mencionar esta versión y estas decisiones. Hasta entonces se puede revisar la propuesta, pero no elegir reglas de publicación definitivas ni marcar terminado el contrato.
