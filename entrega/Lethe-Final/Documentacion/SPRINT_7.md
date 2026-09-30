# Sprint 7 — Restauración física final

## Estado

Completado. Se probó el ciclo físico completo: restauración convencional, despliegue de Lethe, creación de un contenedor VeraCrypt con volumen exterior y oculto, prueba manual de montaje, desmontaje con limpieza de caché y restauración final de la memoria.

## Implementación

- `usb restore-plan` produce un plan sin cambios.
- El plan captura letra, etiqueta, sistema de archivos, disco, partición, cantidad de particiones, capacidad, bus, serial y banderas de seguridad.
- El plan declara pérdida total y un resultado GPT/exFAT/JORGITO.
- La frase física es distinta de la usada con VHDX.
- `restore-execute` exige reproducir todos los campos del plan.
- Rust revalida la identidad antes de crear el ayudante.
- El ayudante elevado vuelve a resolver el disco desde la letra y repite todas las comparaciones.
- La primera instrucción destructiva aparece después de todas las comprobaciones.
- El ejecutable y el script trabajan desde `%TEMP%`, no desde el medio objetivo.
- El resultado se verifica mediante un punto de montaje temporal sin letra.
- El panel muestra dos confirmaciones y se cierra después del resultado.
- El backend destructivo genérico continúa rechazando USB físicas.

## Plan observado

```text
DRIVE_LETTER=F
CURRENT_LABEL=JORGITO
CURRENT_FILESYSTEM=NTFS
DISK_NUMBER=2
PARTITION_NUMBER=1
PARTITION_COUNT=1
SIZE_BYTES=8011120640
BUS_TYPE=USB
IS_BOOT=0
IS_SYSTEM=0
IS_READ_ONLY=0
IS_OFFLINE=0
SERIAL=057907B76060
RESULT_FILESYSTEM=exFAT
RESULT_LABEL=JORGITO
DATA_LOSS=ALL_CURRENT_CONTENT
CHANGES=NONE
```

La identidad se volverá a consultar en el momento de la ejecución. Ningún campo de esta instantánea autoriza por sí solo una restauración futura.

## Validación antes de la prueba destructiva

- 45 pruebas unitarias aprobadas.
- Clippy sin advertencias.
- Compilación Release correcta.
- Sintaxis del panel y del ayudante físico válida.
- Plan real de `F:` completado con `CHANGES=NONE`.
- Confirmación incorrecta rechazada con código específico.
- Identidad modificada rechazada antes del ayudante en pruebas unitarias.
- El código genérico continúa devolviendo `PhysicalUsbDisabled`.

## Prueba del almacenamiento cifrado

- Paquete desplegado de forma transaccional en `F:\Lethe`.
- Contenedor `vault.hc` creado por VeraCrypt con tamaño de 6 GiB.
- Volumen exterior creado y poblado con archivos elegidos por el usuario.
- Volumen oculto creado mediante modo directo después de una desconexión accidental del asistente.
- Contraseñas introducidas solo por el usuario en VeraCrypt y nunca comunicadas a Lethe.
- Montaje manual probado desde el panel en la unidad virtual `L:`.
- Desmontaje normal y limpieza de caché completados.
- Memoria restaurada de nuevo a formato convencional al finalizar la prueba.

## Incidencia de la prueba física

La primera ejecución eliminó la partición NTFS, pero `Clear-Disk` conservó el disco vacío con estilo MBR. La llamada posterior a `Initialize-Disk` falló porque Windows no consideraba el disco RAW. Como no se creó la nueva partición, la memoria desapareció del Explorador.

La identidad del disco se comprobó de nuevo mediante número, serial, capacidad, bus y banderas. El disco seguía saludable, en línea, escribible y sin particiones. Un reparador limitado a esa identidad convirtió MBR a GPT con `Set-Disk`, creó la partición, formateó exFAT y verificó la etiqueta. Ningún otro disco fue consultado ni modificado por el reparador.

El ayudante principal quedó corregido: después de `Clear-Disk` utiliza `Initialize-Disk` solo para RAW y `Set-Disk -PartitionStyle GPT` para un disco vacío MBR.

La restauración posterior del contenedor completó GPT, exFAT y `JORGITO`, pero informó error al intentar reasignar `F:` mientras el panel seguía ejecutándose desde esa letra. La estructura ya estaba terminada y saludable. El diseño definitivo no reasigna la letra desde el ayudante: cierra el panel y requiere reconectar la memoria. Así se evita confundir una asignación de letra fallida con un fallo de formateo.

## Verificación final después de reconectar

```text
DRIVE_LETTER=E
LABEL=JORGITO
FILESYSTEM=exFAT
HEALTH=Healthy
DISK_NUMBER=1
PARTITION_NUMBER=1
PARTITION_COUNT=1
PARTITION_STYLE=GPT
SIZE_BYTES=8011120640
BUS_TYPE=USB
SERIAL=057907B76060
IS_BOOT=0
IS_SYSTEM=0
IS_READ_ONLY=0
IS_OFFLINE=0
```

La prueba confirma que Lethe puede devolver la memoria a un formato convencional. El resultado no se presenta como borrado forense de memoria flash.

La letra y el número de disco cambiaron después de reconectar. El serial `057907B76060` confirmó que Windows seguía informando el mismo dispositivo físico.

## Resultado del sprint

El software y la documentación quedan almacenados en el proyecto local. La memoria de prueba terminó vacía y convencional como `E:`, no cifrada. El paquete puede volver a desplegarse y el contenedor puede recrearse cuando se necesite utilizar Lethe.
