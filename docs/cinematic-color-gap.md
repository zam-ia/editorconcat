# Brecha de color cinematográfico después de P1.2

## Verificado hoy

- Ajustes, ruedas Lift/Gamma/Gain, curvas RGB/Luma, LUT `.cube`, efectos y
  scopes recorren la cadena compartida de preview/export.
- Histogram, waveform, RGB parade y vectorscope analizan la imagen procesada
  display-referred.
- Rec.709 se identifica directamente; HLG/PQ se etiquetan como vista SDR
  tone-mapped y no como medición HDR absoluta.

## Brechas para un flujo Log/Rec.709 completo

1. Transformación de entrada gestionada por cámara/gamma y metadata verificable.
2. Espacio de trabajo lineal o scene-referred explícito antes del look creativo.
3. Transformación de salida versionada por destino, incluidas nits HDR.
4. Halation, bloom y grano evaluados en un espacio documentado, no solo como
   efectos visualmente agradables.
5. Chart tests y material Log real que comparen preview/export por píxel y por
   scopes.

No se declara gestión de color completa hasta cerrar esos contratos. P1.3 no
cambia el renderizador ni introduce una conversión Log automática silenciosa.
