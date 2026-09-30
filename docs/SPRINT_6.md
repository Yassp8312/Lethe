# Sprint 6 — Endurecimiento y preparación de entrega

## Estado

Completado. La memoria `JORGITO` fue consultada en `F:` de forma dirigida y no destructiva. No se montó, formateó ni escribió en ella.

## Implementación entregada

- Comando `usb inspect` limitado a una letra explícita.
- Resolución encadenada letra → volumen → partición → disco, sin enumerar otros discos.
- Validación de etiqueta, bus USB, partición única, tamaño, número de serie y banderas de arranque y sistema.
- Resultado mecanizado con `PHYSICAL_EXECUTION=DISABLED` y `CHANGES=NONE`.
- El backend físico destructivo continúa bloqueado incluso con una identidad válida.
- Registro técnico en `%LOCALAPPDATA%\Lethe\audit.log`.
- Eventos de registro estructurados que excluyen contraseñas, argumentos, rutas e identidad del dispositivo.
- Manual de recuperación y protocolo de prueba final.
- Informe técnico y presentación de entrega.

## Identidad observada el 29 de septiembre de 2026

```text
DRIVE_LETTER=F
LABEL=JORGITO
FILESYSTEM=NTFS
DISK_NUMBER=2
PARTITION_NUMBER=1
PARTITION_COUNT=1
SIZE_BYTES=8011120640
BUS_TYPE=USB
IS_BOOT=0
IS_SYSTEM=0
SERIAL=057907B76060
PHYSICAL_EXECUTION=DISABLED
CHANGES=NONE
```

Esta instantánea no se utilizará como permiso permanente. La identidad debe volver a comprobarse inmediatamente antes de cualquier prueba física futura.

## Pruebas realizadas

- 41 pruebas unitarias aprobadas.
- `cargo clippy --all-targets --all-features -- -D warnings` sin advertencias.
- Compilación Release correcta.
- Etiqueta incorrecta rechazada.
- Disco de sistema rechazado.
- Más de una partición rechazada.
- Número de serie vacío rechazado.
- Confirmación virtual incorrecta rechazada antes del ayudante.
- Cambio de identidad virtual cancela la restauración.
- USB física continúa rechazada por el backend destructivo.
- Inspección real de `F:` completada con `CHANGES=NONE`.

## Resultado

El software está listo para empaquetado y para revisar el protocolo de la prueba física final. El Sprint 6 no autoriza esa prueba destructiva y no cambia el contenido de `F:`.
