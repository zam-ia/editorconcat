# P1.4.2 — rendimiento extremo a extremo

La causa del ETA de 88 min no era QSV: el log confirmó QSV para codificar y
WARP para componer. La preferencia `softwareGpu=true` había sido activada por
una marca de arranque. Se corrigió la preferencia y se reparó `--smoke` para
que retire su tripwire antes de terminar mediante `_exit`.

Se instrumentaron preparación, composición/readback, conversión/codificación,
mezcla de audio, mux y total. En QSV: preparación 67.563 s, composición 53.949
s, conversión/codificación 55.197 s, vídeo 185.174 s, audio 4.324 s, mux 0.311
s y total 189.884 s. El cuello restante es previo al encoder.

La UI nueva muestra GPU/compositor, frames, fps, transcurrido, etapa y ETA
suavizada, y mantiene el progreso monotónico durante audio y mux.
