# Auditoría de seguridad P0.1

**Fecha:** 13 de noviembre de 2026

**Rama:** `feature/concat-p0-1-security-qa`

**Base:** `d24cb305f1f593e078636a93dd87bcf6b01d4166`

**Alcance:** `src/Cargo.lock`, rutas del ejecutable Windows y bibliotecas nativas empaquetadas.

Esta auditoría separa presencia en el árbol, alcance en el producto y evidencia de
ejecución. Una dependencia vulnerable no se considera explotable solamente por
aparecer en `Cargo.lock`.

## Siete alertas de GitHub

| Dependencia | Instalada en la base | Identificador | Severidad GitHub | Corregida en | Clase | Ruta | Evidencia de ejecución en Windows | Estado P0.1 |
|---|---:|---|---|---:|---|---|---|---|
| `rustls` | 0.23.45 | GHSA-h2v8-394m-p7vv | Baja | 0.23.46 | Runtime y build | `concat-host -> ureq -> rustls`; además `sherpa-onnx-sys [build] -> ureq -> rustls` | Sí: `concat-host` usa HTTP para actualizaciones/descargas. La ruta TLS forma parte del ejecutable. | **Resuelta: 0.23.46** |
| `rustls` | 0.23.45 | GHSA-c8m5-c952-6cm4 | Media | 0.23.46 | Runtime y build | Misma ruta | Sí | **Resuelta: 0.23.46** |
| `rustls` | 0.23.45 | GHSA-w44r-93j7-qv28 | Baja | 0.23.46 | Runtime y build | Misma ruta | Sí | **Resuelta: 0.23.46** |
| `rustls` | 0.23.45 | GHSA-vp88-798v-h67f | Baja | 0.23.46 | Runtime y build | Misma ruta | Sí | **Resuelta: 0.23.46** |
| `zip` | 2.4.2 | GHSA-m665-v7pm-7grj / CVE-2026-115905 | Crítica | 9.0.5 | Build nativo | `concat-speech -> sherpa-onnx -> sherpa-onnx-sys [build] -> zip` | No en el producto Windows: el build de Sherpa usa `.tar.bz2` en Windows y el checkout local fija `SHERPA_LIB_PATH`; `ZipArchive::extract` se reserva al paquete iOS `.xcframework.zip`. | **Pendiente aislada** |
| `zip` | 2.4.2 | GHSA-q7v9-x93c-xqv3 / CVE-2026-115909 | Alta | 9.0.0 | Build nativo | Misma ruta | No en Windows, por la misma condición | **Pendiente aislada** |
| `zip` | 2.4.2 | GHSA-8hxf-qr8p-62x4 / CVE-2026-115906 | Media | 9.0.0 | Build nativo | Misma ruta | No en Windows, por la misma condición | **Pendiente aislada** |

`sherpa-onnx-sys 1.13.7` declara `zip = "2"`. Forzar `zip 9` no es una
actualización mínima compatible; actualizar Sherpa a otra línea mayor también
cambia API y ABI nativa. Por eso P0.1 no oculta la alerta con un override no
soportado. La corrección debe hacerse en una rama de dependencia nativa, con
pruebas de voz y del bundle móvil. Mientras tanto, Windows no descarga ni extrae
ZIP en esa ruta.

## Auditoría completa de Cargo

Comando reproducible:

```powershell
cd src
cargo audit
cargo tree -i rustls@0.23.46
cargo tree -i zip@2.4.2
```

Resultado de base: **17 vulnerabilidades** más avisos de mantenimiento/yank.

Resultado P0.1: **4 vulnerabilidades**, todas en `zip 2.4.2` de build; **10
avisos permitidos** sin corrección compatible inmediata.

Actualizaciones mínimas del lockfile:

| Paquete | Base | P0.1 | Motivo |
|---|---:|---:|---|
| `rustls` | 0.23.45 | 0.23.46 | Cierra las cuatro alertas GitHub de runtime |
| `rustls-webpki` | 0.103.15 | 0.103.16 | Corrección de validación asociada al stack TLS |
| `pulldown-cmark` | 0.13.4 | 0.13.6 | Corrige RUSTSEC-2026-0414 y RUSTSEC-2026-0433 |
| `webpki-root-certs` / `webpki-roots` | 1.0.9 | 1.0.10 | Corrige la cadena de raíces WebPKI |
| `zstd` / `zstd-safe` / `zstd-sys` | 0.13.3 / 7.2.4 / 2.0.16 | 0.13.4 / 7.4.1 / 2.1.3 | Corrige la familia de avisos zstd detectada por RustSec |
| `rustix` | 1.1.4 | 1.1.6 | Corrige aviso de plataforma sin salto mayor |
| `tokio` | 1.53.1 | 1.53.3 | Corrección mínima del aviso vigente |

Avisos residuales no explotables demostrados en esta entrega: crates marcados
como no mantenidos (`bincode`, `paste`, `pin-utils`, `rustybuzz`, `ttf-parser`,
`zbus_names`, `zvariant`, `zvariant_derive`) y crates yanked
(`async-executor`, `clipboard-win`). Requieren sustitución o actualización de
sus dependientes, no un cambio indiscriminado del lockfile.

## Bibliotecas nativas del bundle Windows

La inspección del bundle instalado se hizo por versión, procedencia en
`.build-deps`, hash SHA-256 y avisos del proyecto upstream.

| Componente | Versión/identidad observada | Hash representativo | Evaluación |
|---|---|---|---|
| FFmpeg/libav | `n8.1.3-9-g29e619e767`, `avcodec 62.28.103` | `avcodec-62.dll`: `915E8F94CB77855D6EE5DFF77BF2CBD1E484E767CBB8E9622E7FDAD80A1DB0FF` | Runtime, procesa medios no confiables. No se cambia el motor en P0.1; queda riesgo residual de parsers y obligación de probar una actualización nativa separada. |
| ONNX Runtime | 1.28.0, paquete oficial Win x64 | `18370C375F07357FA5874344A9D9AC17E6B6FE1EB18B1DD209D79483B4470257` | Runtime opcional de visión/voz. Sin alertas publicadas por el upstream consultado en esta fecha. |
| sherpa-onnx | 1.13.7, shared MT | `sherpa-onnx-c-api.dll`: `F80A93E69FDA12AAAB23581F71097010C70FB8B8C042FEC7C0EFEAD04BE5C441` | GHSA-f5q5-vp8g-wh4x, baja: modelo transducer manipulado. Los paquetes oficiales fijados reducen la ruta, pero modelos locales manipulados siguen siendo riesgo residual. |
| whisper.cpp | 1.8.3, vía `whisper-rs-sys` parcheado | `whisper.dll`: `4FE133C7AE8C5285C308354D78556DA7CCE69ED05AE9FD82BF55A79E6C893297` | GHSA-v488-xqq5-cf9f, baja, corregida upstream en 1.9.5. Debe actualizarse en una rama aislada con regresión de transcripción/CPU. |
| Microsoft VC runtime | 14.44.35211 en el bundle | Inventario comprobado por el smoke test | Dependencia nativa de distribución; debe viajar junto al ejecutable. |

No se copiaron binarios ni código de los repositorios del anexo open source en
P0.1. Sus licencias, commits y prototipos se evaluarán por separado después de
consolidar esta rama.

## Pruebas y evidencia

| Comprobación | Resultado |
|---|---|
| `cargo audit` | 4 vulnerabilidades residuales, todas `zip 2.4.2` de build |
| `scripts/ci/qa-contract.ps1 -SkipExport` | 5/5 contratos PASS |
| `scripts/ci/qa-contract.ps1` | 7/7 contratos PASS |
| `cargo test -p concat-project -j 1` | 151/151 PASS |
| `cargo test -p concat-host --lib -j 1` | 108/108 PASS |
| Paridad preview/export | PASS; cinco variantes exportadas y leídas |
| Regresión audiovisual | PASS; 35 estados de edición exportados y leídos |
| Reset atómico de relleno | PASS; una operación de undo restaura forma, colores, gradiente, contorno y tamaño |
| Compatibilidad 0.2.x | PASS; carga de documento plano heredado y save/reopen |
| Ejecutable Windows (`cargo build -p concat -j 1`) | PASS |
| Imports del bundle | PASS; cada DLL resuelve al bundle o a Windows |
| `concat.exe --version` con PATH mínimo | PASS; `concat 0.2.8` |
| `concat.exe --smoke` con PATH mínimo | PASS; ventana, Intel UHD/DX12 y Whisper inicializados |
| `cargo audit bin` sobre el ejecutable nuevo | Sin vulnerabilidades recuperadas; tres avisos permitidos. El binario no usa `cargo auditable`, por lo que la recuperación de 190 dependencias es parcial. |

El primer build encontró dos limitaciones del entorno que quedaron resueltas:
CMake no estaba instalado y Cargo intentó enlazar dos test bins simultáneamente,
agotando RAM. El runner ahora localiza CMake instalado por WinGet, limita Cargo a
un trabajo y compila solamente el test lib de `concat`.

El perfil de distribución `app` también requirió declarar
`SHERPA_ONNX_LIB_DIR`; después alcanzó el LTO, pero el proceso local terminó sin
producir `target/app/concat.exe` ni conservar un diagnóstico final. Para no
confundir esto con éxito se construyó y verificó el ejecutable Windows del perfil
`dev`. El smoke del ejecutable es PASS; el artefacto `app` permanece **no
verificado en este equipo** y debe recompilarse en CI o en un host con margen de
LTO antes de publicar.

## Riesgos residuales y decisión

1. `zip 2.4.2` permanece en build para Sherpa. No entra al ejecutable Windows ni
   se ejecuta en su camino de build local, pero mantiene riesgo para builds iOS.
2. FFmpeg recibe archivos no confiables; su actualización necesita un branch de
   codec con la misma matriz de 35 exportaciones y fuzz/corpus de importación.
3. Sherpa y Whisper cargan modelos binarios. Se deben exigir hashes/manifest y
   validar modelos locales antes de ampliar la importación de pesos externos.
4. Los avisos de mantenimiento no son vulnerabilidades demostradas, pero deben
   convertirse en tareas de reemplazo antes de que bloqueen futuras correcciones.
5. Esta rama no hace merge, no publica una versión y no cambia el motor FFmpeg,
   WGPU, Slint ni el esquema persistente de proyectos.

## Fuentes primarias consultadas

- GitHub Security Advisories: [GHSA-h2v8-394m-p7vv](https://github.com/advisories/GHSA-h2v8-394m-p7vv), [GHSA-c8m5-c952-6cm4](https://github.com/advisories/GHSA-c8m5-c952-6cm4), [rustls security](https://github.com/rustls/rustls/security).
- RustSec para la ruta `zip`: [RUSTSEC-2026-0367](https://rustsec.org/advisories/RUSTSEC-2026-0367.html), [RUSTSEC-2026-0368](https://rustsec.org/advisories/RUSTSEC-2026-0368.html), [RUSTSEC-2026-0369](https://rustsec.org/advisories/RUSTSEC-2026-0369.html), [RUSTSEC-2026-0429](https://rustsec.org/advisories/RUSTSEC-2026-0429.html).
- Bibliotecas nativas: [FFmpeg Security](https://ffmpeg.org/security.html), [ONNX Runtime Security](https://github.com/microsoft/onnxruntime/security/advisories), [sherpa-onnx GHSA-f5q5-vp8g-wh4x](https://github.com/advisories/GHSA-f5q5-vp8g-wh4x), [whisper.cpp GHSA-v488-xqq5-cf9f](https://github.com/advisories/GHSA-v488-xqq5-cf9f).
