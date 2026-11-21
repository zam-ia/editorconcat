# Benchmarks de exportación P1.4.2

Equipo local, 2026-11-20. Copia del proyecto real G&A: 66.263 s, 1.988
fotogramas, 1440×2560, 30 fps, H.264 a 12 Mb/s y audio AAC estéreo 48 kHz.

| Pipeline | Total | FPS de vídeo | Frente a CPU | Tamaño |
|---|---:|---:|---:|---:|
| Intel UHD + QSV | 189.884 s | 11.250 | 1.33× | 98.928.064 B |
| Intel UHD + libx264 veryfast | 252.832 s | 8.642 | 1.00× | 99.293.753 B |

El resultado real es 24.9 % menos tiempo con QSV. No se extrapola el 2.31×
del encoder aislado: preparación y composición/readback dominan el pipeline.

La ejecución reportada por el usuario a 88 min usaba `h264_qsv`, pero el
compositor era `Microsoft Basic Render Driver` (WARP). Al restaurar
`softwareGpu=false`, el mismo proyecto completo terminó en 3:10 sobre Intel
UHD/DX12. El ajuste y la marca de arranque obsoleta se respaldaron/corrigieron.

Calidad: 1.988 frames, 66.27 s, audio/vídeo con diferencia final menor a un
frame, sin intervalos negros inesperados, VMAF QSV vs CPU 98.602. Audio en
ambos: −21.5 LUFS, LRA 9.5 LU y pico −4.5 dBFS.

Los logs reproducibles están en `C:\Users\USER\Videos\GA-P142-full-*.log` y
las salidas de prueba en `C:\Users\USER\Videos\GA-P142-full-*.mp4`.
