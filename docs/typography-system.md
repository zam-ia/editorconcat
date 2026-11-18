# Sistema tipográfico P1.3

## Implementación

El selector profesional se alimenta de `fontdb` y `ttf-parser`; no deduce
variantes a partir del nombre del archivo. Cada familia expone sus caras reales
con peso, cursiva, PostScript name, origen, variable/estática, monoespaciado,
clase tipográfica y cobertura de la muestra española.

- Catálogo virtualizado con muestra viva: `¡Nuestra historia comienza aquí!`.
- Búsqueda y filtros: todas, favoritas, recientes, serif, sans serif,
  monoespaciadas, manuscritas e importadas.
- Favoritos tipográficos separados de favoritos de efectos.
- Historial de 12 fuentes recientes, persistente y sin duplicados de mayúsculas.
- Selección de una variante real cambia familia, peso y cursiva conjuntamente.
- Importación TTF/OTF/TTC conserva el flujo existente, registra el archivo en el
  proyecto y actualiza el catálogo en memoria sin releerlo en cada apertura.
- Una familia ausente sigue apareciendo como ausente en el inspector; no se
  sustituye silenciosamente en el documento.

La preferencia de favoritos/recientes no modifica el esquema del proyecto. Los
proyectos 0.2.x conservan su `fontFamily`, `fontWeight` e `italic` existentes.

## Rendimiento

El descubrimiento del sistema ocurre fuera del hilo Slint. El resultado se
agrupa una vez, los metadatos de fuentes importadas se cachean y la lista usa
`ListView`; filtrar no vuelve a analizar archivos.

## Evidencia

- `concat_text::tests::font_catalogue_groups_real_faces_without_duplicate_variants`
- `concat_text::tests::spanish_acceptance_text_has_composable_glyphs`
- `prefs::tests::typography_favourites_and_recents_are_separate_and_persistent`
- `prefs::tests::recent_fonts_are_unique_and_bounded`

## Límites explícitos

Los ejes de una fuente variable se detectan, pero P1.3 no expone todavía un
editor arbitrario de ejes. Las categorías se basan en metadatos OS/2; una fuente
sin esa clasificación queda como Display y no se clasifica por su nombre.
