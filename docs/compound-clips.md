# Clips compuestos P1.4

## Modelo no destructivo

Un clip compuesto es un clip contenedor que referencia una timeline interna ordinaria mediante `compoundTimelineId`. Los hijos conservan identificadores, pistas, tiempos relativos, medios, efectos, keyframes, transiciones y audio. No se genera un MP4 ni se modifica el archivo fuente.

La creación exige al menos dos clips, conserva huecos y solapamientos, retira solamente la selección del padre y la sustituye por un bloque. Es una única operación de undo/redo. Los compuestos anidados se rechazan para impedir ciclos hasta que exista un grafo explícito de dependencias.

## Uso

- `Crear clip compuesto` aparece con una selección múltiple.
- `Abrir clip compuesto` selecciona su timeline interna, que usa las mismas herramientas de edición.
- La pestaña de la timeline permite regresar a la principal.
- `Descomponer clip compuesto` restaura los hijos y crea pistas compatibles cuando la pista original ya no existe.

El trim del contenedor modifica la ventana visible (`source_start`/duración) y no borra hijos. Al descomponer se restauran todos los hijos originales, incluidos los que quedaron fuera de una ventana recortada.

## Preview y exportación

El aplanado expande la timeline interna dentro de la ventana del contenedor, una sola vez. Multiplica volumen, opacidad y escala por los valores globales; suma posición/rotación y antepone/aplica los efectos globales a los hijos visuales. El audio interno conserva fades, filtros y canales y se mezcla una sola vez.

## Límites actuales

- No se permiten compuestos dentro de compuestos.
- Copiar un contenedor todavía comparte su timeline interna; no hay opción de duplicación independiente en esta entrega.
- La velocidad global del contenedor no se expone porque la semántica audiovisual completa aún no está implementada.
