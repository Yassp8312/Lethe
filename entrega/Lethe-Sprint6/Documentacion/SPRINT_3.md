# Sprint 3 — Panel de acceso

## Estado

Completado.

## Implementación entregada

- Panel WinForms compacto para Windows.
- Detección automática de binarios de desarrollo o distribución.
- Soporte para un directorio Lethe explícito mediante `-HomePath`.
- Estado derivado del mapeo real de Windows, no de una suposición de la interfaz.
- Estados visibles: no preparado, bloqueado, esperando contraseña, montado y letra ocupada.
- Botón de apertura segura en solo lectura.
- Apertura con escritura bloqueada por casilla y confirmación adicional.
- Botón de cierre habilitado únicamente cuando existe un volumen VeraCrypt.
- Desmontaje normal verificado.
- Oferta de desmontaje forzado solo después de fallar el cierre normal.
- Limpieza de caché posterior al desmontaje.
- Confirmación al cerrar el panel con un volumen abierto.
- Mensajes públicos basados en códigos, sin mostrar contraseñas ni detalles innecesarios.
- Diagnóstico de estado, versión, letra y ubicación del contenedor.
- Botón de restauración visible pero deshabilitado hasta el Sprint 5.

## Cambios del ejecutable

- Comando `status` con salida estable `clave=valor` para la interfaz.
- Comando `drive-status` para diagnóstico humano.
- Comando `force-lock` para recuperación controlada.
- Rechazo de cierre cuando no existe un volumen montado.
- Rechazo de cierre si la letra pertenece a otro dispositivo.
- Verificación de que la letra quede libre después del desmontaje.
- Código específico `DriveStillMounted` si la verificación posterior falla.

## Prueba visual y funcional

- [x] El panel inicia contra el contenedor local.
- [x] Estado inicial bloqueado y disponible.
- [x] Escritura deshabilitada hasta marcar la casilla.
- [x] Apertura en modo seguro desde el panel.
- [x] Contraseña introducida exclusivamente en VeraCrypt.
- [x] Mapeo real creado en `L:`.
- [x] Panel detecta el estado montado.
- [x] Cierre solicitado desde el panel.
- [x] Caché limpiada después del cierre.
- [x] `L:` vuelve a quedar disponible.
- [x] El panel cierra sin dejar el volumen montado.

## Validación técnica

- [x] 24 pruebas unitarias aprobadas.
- [x] Análisis `clippy` sin advertencias.
- [x] Compilación Release correcta.
- [x] Sintaxis PowerShell validada.
- [x] Contenedor de pruebas local; ninguna operación sobre USB física.

## Entrada del Sprint 4

El Sprint 4 implementará el preparador por consola con planificación, estructura de destino, empaquetado local, ejecución guiada del asistente y reversión de archivos parciales. Continuará trabajando únicamente en carpetas y discos virtuales.
