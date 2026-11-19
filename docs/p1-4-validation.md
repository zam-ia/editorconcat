# Validación P1.4

**Rama:** `feature/concat-p1-4-cinematic-color-and-timeline`
**Base:** `aec3649d42153ec8c086b65f846677f7e59adb48`

## Evidencia automatizada

| Contrato | Evidencia | Estado |
|---|---|---|
| Compatibilidad del modelo 0.2.x | suite completa `concat-project` | 157/157 PASS, incluidos los 154 contratos anteriores y 3 contratos P1.4 |
| Interpretación de fuente | `source_colour_interpretation_is_undoable_and_persists` | comando, undo/redo y reapertura |
| Roles y crecimiento | `new_timelines_expose_audio_spine_and_overlay_roles_without_a_track_limit` | 20 pistas visuales + 20 de audio adicionales |
| Compuesto persistente | `compound_clip_round_trips_edits_and_decomposes_atomically` | crear, guardar, reabrir, editar, descomponer, undo/redo |
| Shader técnico | fixtures default/min/max de `concat.slog3-to-rec709` | validación de catálogo/compilación WGSL |
| Preview/export | suite completa `concat-export` | 39/39 PASS; orden técnico/creativo y expansión del compuesto incluidos |
| Catálogo de efectos | suite completa `concat-effects` | 72/72 PASS; el shader nuevo carga y sus fixtures pasan |
| Títulos y formas | pruebas `titles::` de `concat-host` | 6/6 PASS; rasterizador y caché sin regresiones |
| Aplicación Windows | `cargo check -p concat --bin concat` | PASS con la interfaz Slint completa |
| Ejecutable Windows | `cargo rustc --profile quick`, hash y smoke | PASS; SHA-256 `78FEB508A25436F7456CBD1B86E3B883AD75B822698186FDFC9EA2F40A545419` |

## Instalación ligera validada

- Instalado en `C:\Users\USER\AppData\Local\Programs\Concat-Ligero\concat.exe`.
- El ejecutable instalado y el artefacto `target/quick` tienen el mismo SHA-256.
- Smoke test Windows: proceso receptivo tras 12 segundos, 182.7 MiB de working set y log de arranque sin errores.
- Respaldo reversible: `concat.exe.pre-p1-4-20261119-082654.bak`.

## Rendimiento

No se añadió runtime, framework ni modelo de IA. La conversión Log usa un único pase GPU y el compuesto se guarda como estructura de proyecto; no crea archivos intermedios. Las pistas siguen usando las cachés existentes de thumbnails y waveform.

## Limitaciones que impiden declarar toda la especificación como cerrada

La prueba de resistencia completa de 1.000 clips con métricas de scroll, el visor dedicado de comparación entre dos planos, la duplicación independiente de compuestos y un caso manual con material Sony de cámara conocido requieren fixtures/proyecto de usuario que no se incluyen en el repositorio. No se presentan como `Probado` sin esa evidencia.
