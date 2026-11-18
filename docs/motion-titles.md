# Motion Titles P1.3

## Arquitectura reutilizada

Concat conserva el sistema no destructivo existente: las animaciones del
inspector se convierten en keyframes absolutos de escala, posición, rotación y
opacidad con easing. Por eso la posición final elegida por el usuario no cambia
al aplicar una entrada o salida, y preview/export consumen el mismo modelo.

La galería ofrece Entrada, Salida y Bucle con tarjetas animadas, duración
editable y presets de desvanecer, desplazamientos, zoom/pop, giro, difuso,
rastro, uno por uno y letras binarias. No se añadieron botones sin procesamiento.

## Colección original representativa

P1.3 incorpora cuatro looks editables, construidos con fuentes ya licenciadas y
embebidas en Concat:

| Colección | Preset | Uso |
|---|---|---|
| Wedding | Wedding Cinema | Apertura serif, tracking amplio, sombra y halo dorado sutil |
| Documentary | Documentary Lower Third | Nombre/cargo legible con placa discreta |
| Commercial | Commercial Reveal | Titular Montserrat de alto contraste |
| Social | Social Hook | Hook vertical con trazo, sombra y resplandor rojo |

Los presets son TOML, editables después de aplicarlos, y no copian plantillas,
logos ni recursos propietarios de las imágenes de referencia.

## Limitaciones

Estas cuatro entregas son títulos de un elemento. Un contenedor multi-elemento
con agrupación interna, reemplazo de medios y edición jerárquica requiere un
modelo persistente versionado y no se simuló con una capa visual.
