# Pipeline de color cinematográfico P1.4

## Alcance validable

Concat conserva Rec.709 SDR como salida inicial y añade una interpretación explícita por medio:

- Automático: conserva el comportamiento 0.2.x y no presume que un archivo desconocido sea Log.
- Rec.709 SDR: no inserta conversión técnica.
- Sony S-Log3 / S-Gamut3.Cine.
- Sony S-Log3 / S-Gamut3.

La selección vive en `MediaItem.source_color`, se modifica con un comando transaccional y persiste en el documento. Un proyecto antiguo no contiene el campo y, por tanto, abre como `Auto` sin cambiar su imagen.

## Orden de procesamiento

`interpretación de origen → conversión técnica → efectos/correcciones del clip → composición → salida Rec.709 SDR`

Para los dos perfiles Sony, `concat.slog3-to-rec709` se antepone automáticamente a la lista de efectos del clip en el plan compartido por preview y exportación. La curva inversa S-Log3 y las matrices S-Gamut3/S-Gamut3.Cine a Rec.709 están implementadas en un único shader WGPU. No se inserta para Rec.709 ni Auto, evitando una doble conversión.

## Looks editables disponibles

La biblioteca existente aporta combinaciones originales y parametrizables que cubren los seis usos solicitados: `Warm` (Wedding Warm), `Film` (Soft Film), `Clean` (Clean Commercial), `Natural` (Documentary Natural), `Moody` y `Golden Hour`. Sus parámetros siguen siendo efectos ordinarios, con intensidad/bypass, undo y persistencia; no son imágenes precocinadas ni alteran geometría facial.

## Límites honestos

- La detección automática de metadatos Log no se activa cuando el contenedor no es inequívoco; el usuario debe seleccionar la interpretación.
- P1.4 no declara gestión HDR completa ni ACES/OCIO.
- Recuperar altas luces no puede reconstruir canales ya recortados en la fuente.
- La comparación de dos planos permanece basada en los scopes existentes; todavía no existe un visor dividido dedicado.
