# Auditoría funcional P0 de Concat Studio

**Fecha:** 12 de noviembre de 2026  
**Base examinada:** EditorConcat sobre Concat 0.2.8  
**Especificación:** `CONCAT_STUDIO_Especificacion_Maestra_UX_Funcional_v1.md`

Esta auditoría distingue presencia visual de funcionamiento real. Un módulo solo figura como
`Probado` cuando existe evidencia automatizada del procesamiento; una pantalla o un manifiesto
sin recorrido hasta preview/export queda como `Visual` o `Parcial`.

## Estados

- **Ausente:** no se encontró implementación utilizable.
- **Visual:** existe interfaz, pero no se demostró procesamiento.
- **Parcial:** funciona un subconjunto o falta una parte del contrato.
- **Funcional:** UI, modelo y motor están conectados.
- **Probado:** además tiene prueba automatizada relevante.

## Mapa de responsabilidades real

| Área | Implementación actual |
|---|---|
| Proyecto y migraciones | `concat-project`: modelo serializable, migraciones y comandos transaccionales |
| Historial | `concat-project::Editor` y `concat-host::Session`: aplicar, undo y redo |
| Medios | `concat-media`: probe, demux, decode/encode, audio y frames |
| Índice/caché | `concat-host`: proyectos, proxies, thumbnails, waveform, relink y consolidación |
| Timeline y UX | `concat` + Slint: biblioteca, visor, inspector, timeline y diálogos |
| Efectos | `concat-effects`: manifiestos, parámetros, WGSL, FFmpeg y LUT `.cube` |
| Composición | `concat-render`: plan común, compositor WGPU y fallback de referencia |
| Preview | `concat-host::preview` y `playback`: mismo plan semántico del exportador |
| Exportación | `concat-export` + `concat-host::export`: flatten, render y encode |
| Texto y formas | `concat-text` + `concat-host::titles`: raster común para visor/export |
| Voz/visión local | `concat-speech` y `concat-vision`, activados por capacidades/modelos |
| Automatización | `concat-api`, `concat-server` y `concat-mcp` |

El producto es una aplicación nativa en Rust con UI Slint, FFmpeg/libav para medios y WGPU para
composición. No es Electron ni una interfaz web. Introducir WebCodecs o Web Audio no aportaría al
stack actual sin una justificación adicional.

## Matriz funcional

| Herramienta | Estado | Motor | Persistencia | Preview | Export | Prueba/evidencia | Bloqueante principal |
|---|---|---|---|---|---|---|---|
| Crear proyecto y formato | Probado | Modelo racional de FPS y ajustes de video | Sí | Sí | Sí | `concat-project`, `dialogs/project.slint`, pruebas de export | Aspecto de píxel y presets de espacio de trabajo aún no son una UX completa |
| Guardado atómico | Probado | Temporal, flush y rename | Sí | N/A | N/A | `concat-host/src/projects.rs` y pruebas de recuperación | Falta historial visible de versiones para el usuario |
| Autosave y recuperación | Probado | Timer + snapshots con protección de orden | Sí | N/A | N/A | `concat/src/lib.rs`, pruebas de autosave retrasado | Intervalo/configuración y UX de recuperación pueden ampliarse |
| Importación y probe | Probado | FFmpeg/libav y metadata de streams | Sí | Sí | Sí | `concat-media`, pruebas de import/export | La UI no expone cada tag técnico solicitado por la especificación |
| Miniaturas y ondas | Funcional | Trabajos de fondo y caché | Caché | Sí | N/A | `concat-host`, `media-pane.slint` | Falta diagnóstico de colas y uso de caché |
| Preview de biblioteca y marcado “Añadido” | Funcional | Decode separado del timeline | Estado UI | Sí | N/A | `media-pane.slint`, `studio.rs` | Pendiente prueba UI automatizada estable en Windows |
| Duplicados de medios | Funcional | Ruta/identidad de media | Sí | N/A | N/A | `AddMedia` y modelo de media | No existe deduplicación por hash de contenido completa |
| Proxies | Probado | Reencode automático 1/4, cacheado; originales en export | Caché | Sí | Original | `concat-host/src/proxy.rs` y sus pruebas | Calidad 1/2–1/4 no es seleccionable por clip/proyecto |
| Medios faltantes y relink | Probado | Detección + búsqueda recursiva + comando de relink | Sí | Placeholder/diálogo | Tras relink | `panes/relink.rs`, `missing_media` tests | Falta empaquetado portable con informe completo al usuario |
| Pistas y orden Z | Probado | Tracks ordenados, visible/mute/lock | Sí | Sí | Sí | comandos de pista y `concat-host/tests/export.rs` | Solo/buses de audio no alcanzan el contrato profesional |
| Mover, trim y split | Probado | Comandos transaccionales con límites | Sí | Sí | Sí | `concat-project` y export end-to-end | Timing persistido aún usa muchos `f64` segundos |
| Lift y ripple | Probado | `RemoveClips` y trims con `ripple` | Sí | Sí | Sí | pruebas de comandos/export | Falta UX explícita para todos los alcances de ripple en grupos |
| Insertar y sobrescribir | Funcional | Colocación con modo ripple/overlap | Sí | Sí | Sí | `placement.rs`, operaciones de drop | Slip, slide y rolling profesionales no están completos |
| Snapping | Funcional | Clips, playhead y guías de visor/timeline | Preferencia | Sí | N/A | `timeline`, `preview-pane.slint` | Faltan marcadores/keyframes como objetivos universales de snap |
| Audio/video enlazado | Probado | Detach/reattach y edición agrupada | Sí | Sí | Sí | export end-to-end | Falta UI avanzada de grupos arbitrarios y sync-lock por pista |
| Freeze y reverse | Probado | Still derivado y copia invertida cancelable | Sí | Sí | Sí | pruebas de export y `reverse.rs` | Reverse de audio requiere comunicar limitaciones con mayor claridad |
| Transform, crop, flip y blend | Probado | Grafo/compositor WGPU | Sí | Sí | Sí | pruebas de export y compositor | Punto de ancla editable y motion blur aún parciales/ausentes |
| Velocidad constante y rampa | Probado | Speed map con resolución para video/audio | Sí | Sí | Sí | `speed.rs`, `resolve.rs`, export end-to-end | Flujo óptico no está implementado; timing sigue basado en segundos flotantes |
| Máscaras | Probado | Lineal, espejo, círculo, rectángulo, corazón y estrella | Sí | Sí | Sí | `ClipMask`, compositor y tests | Polígono, trazado libre y booleanos suma/resta/intersección ausentes |
| Estabilización | Probado | Análisis + transform/crop compartido | Sí | Sí | Sí | `concat-host/tests/stabilize.rs` | Falta un panel de diagnóstico del análisis y confianza |
| Texto, caja, trazo y sombra | Probado | `concat-text`, raster no destructivo | Sí | Sí | Sí | pruebas de texto/títulos/export | Controles avanzados de sombra (spread y X/Y separados) no están completos |
| Animaciones entrada/salida/bucle | Funcional | Presets convertidos a keys/curvas | Sí | Sí | Sí | `motion.rs`, compositor y tests existentes | Catálogo no cubre aún toda la especificación; falta prueba por cada preset |
| Keyframes y curvas de easing | Probado | Propiedades absolutas, hold/linear/ease/bezier | Sí | Sí | Sí | `KeyProperty`, keyframe pane y tests | No todas las propiedades nuevas son animables; falta multi-selección general |
| Subtítulos locales | Funcional | Speech local y clips de texto | Sí | Sí | Sí | `concat-speech`, caption workflows | Export `.srt/.vtt` y resaltado palabra a palabra requieren auditoría adicional |
| Transiciones | Probado | Resolución A/B con handles y compositor | Sí | Sí | Sí | export end-to-end para catálogo base | Alineación inicio/centro/final y crossfade de audio configurable incompletos |
| Efectos y filtros | Probado | WGSL/FFmpeg parametrizado con bypass/keys | Sí | Sí | Sí | catálogo, render tests y export end-to-end | No todos tienen benchmark o fallback equivalente documentado |
| Ajustes básicos de color | Probado | Shader `concat.adjust` | Sí | Sí | Sí | WGSL, manifests, pruebas del catálogo/render | Falta calibración cuantitativa completa por control y material real |
| Lift/Gamma/Gain | Probado | Ruedas cromáticas + luminancia separada | Sí | Sí | Sí | `grading.slint`, `grading.rs`, shader/tests | Falta RGB parade para comparación |
| Curvas Luma/R/G/B | Probado | Editor de puntos y curva real | Sí | Sí | Sí | `grading.slint`, manifest y tests | Histograma bajo curva no implementado |
| HSL selectivo | Parcial | Funciones/bandas HSL en efectos y presets | Sí | Sí | Sí | WGSL de catálogo | No existe aún el editor universal de ocho bandas con eyedropper |
| LUT `.cube` | Probado | Parser, LUT GPU y cadena FFmpeg | Sí | Sí | Sí | `cube.rs`, `looks.rs`, render tests | Falta advertencia automática Log→Rec.709 y relink UX del LUT |
| Scopes | Ausente | — | — | — | N/A | No se encontró waveform/parade/vectorscope de color | Requiere módulo de análisis, UI y presupuesto GPU/CPU |
| Volumen en dB y fades | Probado | Gain lineal persistido, UI logarítmica; fades reales | Sí | Sí | Sí | inspector, mix y export tests | Curvas de fade seleccionables no están completas |
| Denoise/mejora de voz | Parcial | Cadena DSP/FFmpeg y módulos locales | Sí | Sí | Sí | paquetes y host jobs | Falta suite cuantitativa 0/50/100, latencia y artefactos por modo |
| Normalización LUFS | Parcial | Filtro/acción disponible | Sí | Sí | Sí | catálogo/inspector | Falta reporte antes/después y prueba independiente de LUFS/true peak |
| Mezclador, buses y ducking | Parcial | Mezcla por clips/pistas; helpers de música/voz | Sí | Sí | Sí | playback/export/sounds | No hay mezclador completo A1–An con buses, solo, plugins y automatización |
| Export H.264/AAC | Probado | Render/encode cancelable y lectura de resultado | N/A | Mismo plan | Sí | `concat-host/tests/export.rs` | Cola/pausa/stems y matriz completa de codecs dependen de capacidades |
| Export 4K/FPS/configuración | Funcional | Resolución y tasa configurables | Preset/proyecto | N/A | Sí | diálogo y exporter | Rendimiento/codec depende de hardware; falta benchmark visible por equipo |
| IA local opcional | Parcial | Speech, cutout, enhance y modelos descargables | Sí/caché | Sí | Sí | crates speech/vision y jobs | Cada modelo necesita capability gate, licencia, memoria y QA propio |
| Diagnóstico de RAM/VRAM/jobs | Parcial | Logs y estados internos | Logs | Parcial | Parcial | jobs/crash/logging | Falta panel único visible con métricas y recuperación |

## Hallazgos P0/P1

1. **P0 — reloj del proyecto:** el modelo conserva FPS racional, pero clips, keys y source ranges
   todavía usan numerosos `f64` en segundos. Migrar a ticks/PTS requiere versionado de schema y no
   debe hacerse como parche aislado.
2. **P0 — evidencia por función:** existe una base de pruebas excepcionalmente amplia, pero no una
   matriz que impida declarar una función terminada sin save/reopen/undo/export. Este documento se
   convierte en la fuente inicial y debe actualizarse por slice.
3. **P1 — scopes de color:** son la ausencia profesional más clara del panel de grading.
4. **P1 — audio profesional:** faltan buses, solo, routing y medición LUFS/true-peak verificable,
   aunque el gain, fades y la cadena de procesamiento básica sí son reales.
5. **P1 — proxy UX:** el proxy automático funciona y exporta con originales, pero no ofrece la
   elección Completa/1/2/1/4 exigida.
6. **P1 — operaciones avanzadas de timeline:** slip/slide/rolling y sync-lock necesitan diseño y
   comandos propios antes de exponerse.
7. **P1 — controles de forma:** el degradado era funcional, pero el grupo no tenía reset atómico;
   eso incumplía el contrato universal de controles.

## Slice vertical 1 — degradado de formas verificable

Esta entrega cierra el hallazgo 7 sin crear una función decorativa:

- `ShapeStyle` conserva sólido o degradado lineal/radial de 2–4 stops con alpha.
- El inspector ofrece modo, colores, posiciones, ángulo/centro/radio, contorno y tamaño.
- El encabezado **Relleno** ahora tiene reset atómico a sólido blanco, sin contorno y tamaño base;
  todo el gesto es un solo elemento de undo.
- La prueba de proyecto confirma undo/redo y round-trip del documento.
- La prueba del host muestrea píxeles distintos del degradado en el frame usado por el visor.
- La prueba end-to-end exporta H.264 y lee el archivo resultante para demostrar que el lado
  izquierdo sigue rojo y el derecho azul después del encoder.

### Criterios de aceptación del slice

| Criterio | Evidencia |
|---|---|
| Cambio perceptible | Muestreo rojo→azul en `concat-host::titles` |
| Undo/redo | `shape_clips_survive_without_media_and_carry_their_figure` |
| Save/reopen | Round-trip del documento en la misma prueba |
| Preview/export match | Raster del host + export H.264 leído de vuelta |
| Compatibilidad | `gradient: None` mantiene proyectos anteriores como sólidos |
| Reset | `ResetButton` del grupo Relleno, un solo `commit()` |

## Siguientes slices recomendados

1. **QA contractual automatizado:** tabla ejecutable que enlace funciones con pruebas reales.
2. **Proxy quality:** selector Completa/1/2/1/4, persistencia y prueba export-original.
3. **Audio metering:** LUFS/true peak antes/después para normalización, sin añadir otro toggle.
4. **Scopes P1:** histograma y waveform Luma primero; RGB Parade y vectorscope después.
5. **Timeline profesional:** implementar rolling edit como comando transaccional antes de UI.
6. **Timing P0:** diseñar migración gradual a ticks racionales sin romper documentos 0.2.x.

Cada slice debe actualizar esta matriz y aportar las cuatro pruebas mínimas: cambio medible,
undo/redo, save/reopen y coincidencia de exportación.
