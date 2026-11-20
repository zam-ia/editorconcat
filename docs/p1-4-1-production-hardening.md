# P1.4.1 — Production hardening

Fecha de validación: 2026-11-20. Rama: `feature/concat-p1-4-1-production-hardening`.

## Problema confirmado

La solicitud de exportación activaba `EncodeOptions.hardware`, pero H.264 en Windows
siempre enumeraba `libx264` antes que los codificadores acelerados. Como `libx264`
abre en cualquier equipo, Quick Sync, NVENC, AMF y Media Foundation nunca llegaban a
intentarse. El resultado era una exportación por CPU incluso cuando la interfaz
indicaba que la aceleración estaba disponible.

## Corrección

- Modo acelerado H.264 en Windows: `h264_qsv`, `h264_nvenc`, `h264_amf`,
  `h264_mf`, y finalmente `libx264`.
- Modo compatible/no acelerado: conserva `libx264` como primera opción.
- Si el dispositivo no puede abrir un encoder, el motor prueba el siguiente sin
  abortar la exportación.
- El log registra el encoder que realmente abrió, junto con codec, resolución,
  tasa, preset y CRF.

## Prueba de capacidad local

Sonda FFmpeg de 300 fotogramas, `testsrc2`, 1440×2560 a 30 fps y 12 Mb/s. Sirve para
confirmar apertura y descartar rutas imposibles; no reemplaza el benchmark del
proyecto completo ni demuestra calidad visual equivalente.

| Encoder | Resultado | Tiempo de pared | Observación |
|---|---:|---:|---|
| `h264_qsv` | abre | 3.990 s / 75.18 fps | 2.31× el throughput de CPU |
| `h264_mf` | abre | 2.850 s / 105.30 fps | ruta más rápida de la sonda; no se etiqueta como GPU explícita |
| `libx264 veryfast` | abre | 9.240 s / 32.48 fps | fallback CPU |
| `h264_nvenc` | falla | — | no hay dispositivo NVIDIA compatible |
| `h264_amf` | falla | — | no hay dispositivo AMD compatible |

## Timeline de audio

Al soltar un audio donde la pista elegida ya está ocupada, la operación crea una
nueva pista compatible y conserva el inicio solicitado. Se validaron seis audios
simultáneos (A1–A6), y las pistas visuales continúan rechazando audio. La operación
usa el mismo comando transaccional del proyecto, por lo que participa de undo/redo
y persistencia sin cambiar el esquema 0.2.x.

## Evidencia automatizada

- `cargo test -p concat-media windows_h264_hardware_mode_leads_with_qsv_and_keeps_x264_fallback`
- `cargo test -p concat-project overlapping_sounds_open_a2_through_a6_and_visual_tracks_refuse_audio`
- `cargo check -p concat`
- `cargo build --release -p concat`
- `scripts/ci/export-encoder-benchmark.ps1`

## Pendiente antes de declarar P1.4.1 cerrada

- Benchmark reproducible del proyecto G&A completo en software y automático.
- Medición de decode, composición, conversión RGBA→YUV, encode y mux por separado.
- VMAF/SSIM y validación del archivo exportado para fijar presets de calidad.
- Pruebas de estrés de 1.000 clips y cancelación/reanudación bajo carga.

Estos puntos permanecen explícitamente pendientes: la corrección no se presenta
como cierre de toda la fase ni como prueba de calidad equivalente entre encoders.
