# Scopes profesionales de color (P1.2)

## Alcance

Concat calcula **histograma RGB + luma, waveform luma, RGB parade y
vectorscope** a partir del frame RGBA8 ya procesado por el compositor. Por
ello los instrumentos reflejan capas, transiciones, efectos, ajustes y títulos
que se ven en el monitor, sin modificar el proyecto ni los medios originales.

El análisis se ejecuta fuera del hilo de interfaz sobre una copia reducida a un
máximo de 320 píxeles por lado. Los cuatro instrumentos se obtienen en una sola
pasada y se reutilizan mientras no cambien revisión, tiempo, tamaño o espacio
de color. Durante reproducción se limita la frecuencia para evitar trabajo
innecesario.

## Escalas de señal

- SDR usa código de pantalla Rec.709: `Y' = 0.2126 R' + 0.7152 G' + 0.0722 B'`.
- Vectorscope usa `Cb = (B' - Y') / 1.8556` y `Cr = (R' - Y') / 1.5748`.
- En timelines HLG/PQ el monitor existente entrega una vista SDR tone-mapped.
  La interfaz lo indica explícitamente; no se presenta como medición HDR de la
  señal fuente.
- Los extremos 0/255 se muestran como posible clipping de pantalla. No se
  confunden con valores lineales ni con exposición absoluta HDR.

## UX y rendimiento

Los scopes viven debajo del panel **Ajustar**, con selector de instrumento,
vista ampliable, refresco manual, etiqueta de señal y tiempo de análisis. La
selección de instrumento es estado de interfaz; no crea un comando ni ensucia
el guardado. Los resultados obsoletos se invalidan por revisión y tiempo.

## Evidencia reproducible

`cargo test -p concat-host --lib scopes` valida negro, blanco, gris neutro,
gradiente espacial, barras RGB, clipping, centro cromático, primarios,
inmutabilidad del frame y tamaño de los bitmaps. `cargo check -p concat`
verifica el contrato completo Rust/Slint/host.

## Limitaciones

Esta primera entrega es display-referred y CPU. No sustituye un monitor HDR
calibrado ni mide nits de la señal fuente. El muestreo reducido prioriza una UI
ligera; para inspección píxel a píxel sigue siendo necesario exportar y medir el
archivo final con una herramienta de referencia.
