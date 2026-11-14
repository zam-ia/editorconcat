# Medición y normalización de audio P1

**Rama:** `feature/concat-p1-audio-metering`

**Base:** `bc248d265115ef9470e9d7b8901d3b6c4204ed4c`

**Estándares:** ITU-R BS.1770 y EBU R128

## Decisión técnica

Se revisaron `libebur128`, `ebur128`, `loudnorm` y `astats`. Concat ya enlaza FFmpeg/libavfilter
y su filtro `ebur128` ofrece M, S, I, LRA y true-peak sobremuestreado por metadata. Por eso no se
añadió otra biblioteca nativa: evita duplicar DSP, licencias, memoria y tamaño del ejecutable.
`astats` no se usa como sustituto de true-peak; el máximo de muestras se calcula y presenta aparte.

`libebur128` 1.2.6 (MIT) sigue siendo una referencia válida, pero incorporarlo ahora sería
redundante. `loudnorm` permanece disponible en el catálogo existente; esta entrega usa ganancia
no destructiva para que undo/redo y el proyecto sigan editables.

## Ruta de señal

1. El clip elegido o la timeline audible se aplana con el mismo plan que exportación.
2. `mix_to_file_with_control` procesa stream, recorte, velocidad, volumen/ride, fades, filtros y suma.
3. La mezcla temporal AAC se analiza con `ebur128=metadata=1:peak=true`.
4. Se propone una ganancia constante con objetivo LUFS y techo dBTP, o modo solo atenuar.
5. Un `Command::Batch` actualiza volumen y todas las keys de volumen como un único undo.
6. Se vuelve a renderizar y medir; la UI muestra el resultado real posterior, no solo una predicción.

Los archivos fuente nunca se modifican. Los resultados se invalidan al cambiar revisión, selección
o contexto sonoro. El trabajo corre fuera del hilo de UI, informa progreso y admite cancelación.

## Métricas

- **I:** LUFS integrados con compuerta.
- **M / S:** máximo momentáneo (400 ms) y corto plazo (3 s).
- **LRA:** loudness range cuando la señal permite calcularlo.
- **True-peak:** pico reconstruido en dBTP por el sobremuestreo de FFmpeg.
- **Sample peak:** máximo de muestras decodificadas en dBFS.
- **Clipping:** cantidad de muestras a o por encima de escala completa.

Silencio no recibe un LUFS ficticio: I y picos se devuelven como ausentes.

## Tolerancias y referencia reproducible

La prueba de integración genera una mezcla estéreo multipista de 48 kHz con niveles distintos,
fade-in, fade-out y `highpass`, la codifica antes/después y analiza ambos archivos. Resultado de la
corrida del 13-11-2026:

| Lectura | Valor |
|---|---:|
| Antes | −25.84 LUFS |
| Después | −16.00 LUFS |
| True-peak posterior | −14.56 dBTP |

Aceptación: ±0.5 LU respecto al objetivo integrado y no más de +0.1 dB sobre el techo dBTP. La
prueba separada de inter-sample peak exige que true-peak supere sample peak por más de 2 dB en una
señal desplazada de alta frecuencia.

## Compatibilidad y límites

- No cambia el schema: persiste únicamente volumen y keyframes 0.2.x ya existentes.
- No reemplaza Slint, FFmpeg ni WGPU y no incorpora modelos de IA.
- Es medición offline contextual, no un medidor en vivo durante playback.
- La mezcla final modifica las ramas audibles actuales; no existen todavía buses, solo ni routing.
- La normalización transparente no usa compresión dinámica. Si LUFS y dBTP no pueden cumplirse a
  la vez, se aplica el máximo gain seguro, se muestra la limitación y no se declara éxito falso.
- El modo solo atenuar nunca aumenta ganancia, aunque por ello no alcance el objetivo LUFS.

## Pruebas y QA

`scripts/ci/qa-contract.ps1` exige evidencia textual de al menos una prueba esperada y falla aunque
Cargo devuelva código 0 cuando el filtro coincide con cero tests. Los contratos P1 cubren señal real,
exportación, cancelación, undo/redo, persistencia y compatibilidad sin migración.

Resultados Windows del 13-11-2026:

- `cargo check -p concat`: PASS (incluye generación/chequeo Slint).
- `cargo test -p concat-media --lib`: 120 PASS, 1 ignorada por ser exclusiva de Android.
- `qa-contract.ps1 -SkipUi`: 8/8 PASS, incluida la regresión audiovisual de 35 estados.
- El enlace del test monolítico `concat --lib` con símbolos completos se canceló después de más de
  30 minutos en MSVC. No hubo error de código; el perfil `quick` también se canceló al iniciar una
  recompilación total redundante. Por eso no se declara smoke de un ejecutable nuevo en esta rama.
