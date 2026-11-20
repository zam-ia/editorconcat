# Aceptación P1.4.1

## Aceptado e instalado

- A1–A6 automáticas para seis audios simultáneos.
- Pistas visuales no aceptan audio.
- H.264 Quick Sync abre a 1440×2560 y supera 2× a libx264 en la sonda aislada.
- Fallback a CPU conservado.
- `cargo check`, pruebas focalizadas y build `release` aprobados.
- Arranque temporal e instalado: proceso responsivo, ~176 MB en reposo.
- SHA-256 instalado: `B0ED9BD1068A178E2E5D976BDE1DA4EF9A319593C4E86C3F035E303943755265`.

## Pendiente; no se declara cerrado

- Exportación comparable completa de G&A y validación VMAF/SSIM.
- Perfil por etapa decode/composición/readback/conversión/encode/mux.
- Modos de exportación seleccionables en UI e informe exportable.
- Comparación de color dedicada, compuestos independientes y estrés de 1.000 clips.
