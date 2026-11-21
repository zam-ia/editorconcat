# P1.4.2 — aceptación

| Criterio | Resultado |
|---|---|
| Proyecto G&A completo | 66.263 s exportados |
| Aceleración real | QSV 189.884 s; CPU 252.832 s |
| GPU de composición | Intel UHD/DX12 verificada |
| Duración/resolución/fps | Verificado con ffprobe |
| Audio y sincronía | Métricas iguales; delta menor a un frame |
| Cuadros negros | Ninguno inesperado |
| Similitud visual | VMAF 98.602 |
| Regresión de compilación | `cargo check -p concat` correcto |
| Instalación nueva | Pendiente de build release; se conserva P1.4.1 optimizado |

No se declara instalada la UI P1.4.2 hasta completar build release y smoke.
La corrección operacional de GPU sí quedó aplicada al instalado.
