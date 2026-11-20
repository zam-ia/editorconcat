# Benchmarks de exportación P1.4.1

Equipo local, 2026-11-20. Escenario A: 300 frames sintéticos, 1440×2560,
30 fps, H.264, 12 Mb/s. Tres ejecuciones de capacidad no se sustituyen por una
afirmación sobre el proyecto G&A completo.

| Encoder | Segundos | FPS | Factor vs CPU | Archivo |
|---|---:|---:|---:|---:|
| libx264 veryfast | 9.240 | 32.48 | 1.00× | 15,344,301 B |
| Intel Quick Sync | 3.990 | 75.18 | 2.31× | 16,288,643 B |
| Media Foundation | 2.850 | 105.30 | 3.24× | 15,609,013 B |

Reproducir con:

```powershell
scripts/ci/export-encoder-benchmark.ps1 -Frames 300 -Width 1440 -Height 2560 -Fps 30
```

El caso G&A de 66 segundos y la comparación VMAF/SSIM permanecen pendientes;
no se inventa una mejora de extremo a extremo a partir del encoder aislado.
