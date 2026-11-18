# Subtítulos profesionales P1.3

## Identidad y compatibilidad

Los subtítulos nuevos se crean con `Command::AddSubtitleClip`. Siguen siendo
clips de texto normales para el renderizador, pero guardan
`extra.role = "subtitle"`. El campo `extra` ya forma parte del documento 0.2.x,
por lo que no hay migración ni ruptura de esquema.

La exportación SRT usa primero esa identidad explícita y excluye títulos de
apertura/lower thirds. En proyectos antiguos sin rol conserva el fallback de la
pista de títulos más ocupada para no perder compatibilidad.

## Edición

- Contenido, inicio y duración continúan siendo editables como clips normales.
- El inspector permite **Aplicar a subtítulos** desde un título de referencia.
- La operación reemplaza solo la apariencia, conserva palabras y tiempos, y se
  ejecuta como un único `Command::Batch`; un undo revierte toda la pista.
- Importar/exportar SRT conserva cues cortos y texto multilínea.

## Evidencia

- `srt::tests::explicit_subtitles_are_not_confused_with_title_cards`
- `srt::tests::subtitle_track_style_is_one_atomic_undo_step`
- `panes::captions::tests::imported_srt_preserves_short_cues_and_multiline_text`

## Límites explícitos

El motor de transcripción actual entrega segmentos, no timestamps verificables
por palabra. Por ello P1.3 no presenta resaltado karaoke palabra a palabra como
función terminada. VTT y reglas automáticas avanzadas de dos líneas quedan para
un slice posterior con pruebas de round-trip propias.
