# Timeline profesional P1.4

## Roles persistentes

Las pistas nuevas tienen uno de tres roles: `mainVisual`, `visual` o `audio`. Los documentos 0.2.x sin rol conservan `legacy`, su orden y su mezcla. No existe migración silenciosa ni reordenamiento de proyectos antiguos.

Una timeline nueva contiene A1 en la región inferior, V1 como columna vertebral y dos overlays visuales. Las pistas añadidas arriba son visuales; las añadidas abajo son de audio. El encabezado deriva etiquetas A1, A2, V1, V2… del rol y el orden actual, y V1 recibe una distinción visual propia.

## Edición y sincronización

- V1 conserva inserción magnética, trim y borrado ripple existentes.
- `sync_lock` desplaza overlays visuales cuando un ripple nace en V1.
- El audio independiente no se mueve por ese ripple.
- Las pistas visuales rechazan clips de audio en movimientos nuevos.
- La pista de audio base conserva colocaciones visuales heredadas para no romper documentos 0.2.x que utilizaron pistas genéricas.
- Snapping y ripple siguen siendo operaciones separadas.

No hay un máximo artificial de pistas. La vista reutiliza el modelo de filas existente y no crea decodificadores por pista; thumbnails, waveforms y scopes continúan siendo perezosos/cacheados.

## Compatibilidad

Los campos `role`, `name` y `syncLock` tienen defaults tolerantes y se omiten cuando equivalen al modelo heredado. Visibilidad, mute, bloqueo, orden temporal, links, efectos y exportación no se reinterpretan al abrir un proyecto anterior.

## Pendientes

El redireccionamiento visual con mensaje explícito al arrastrar un clip a una región incompatible y el ajuste interactivo de altura de pista no forman parte de este corte. La validación del modelo impide audio en pistas visuales, pero la pista A1 conserva la excepción de compatibilidad descrita arriba.
