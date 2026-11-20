# Selección de encoder P1.4.1

En Windows, el modo acelerado prueba `h264_qsv`, `h264_nvenc`, `h264_amf`,
`h264_mf` y finalmente `libx264`. Cada fallo abre el siguiente candidato.
Quick Sync encabeza la lista porque en este equipo abre a la resolución objetivo
y representa hardware Intel explícito. Media Foundation fue más rápida en la
sonda, pero no se etiqueta como GPU sin poder demostrar el dispositivo interno.

El modo no acelerado conserva `libx264` primero. El encoder que realmente abre se
registra en el log de exportación con codec, resolución, FPS, preset y CRF.
