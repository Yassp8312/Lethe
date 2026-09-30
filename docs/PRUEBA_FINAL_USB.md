# Protocolo de prueba final USB

## Estado actual

La prueba física del Sprint 7 fue ejecutada con autorización y completada. Crear un plan sigue siendo una operación sin cambios; repetir la restauración requiere una autorización nueva.

## Objetivo autorizado

La candidata observada es `F:`, etiqueta `JORGITO`, disco 2, tamaño 8 011 120 640 bytes, bus USB y número de serie `057907B76060`. La letra y el número de disco pueden cambiar; los datos deben revalidarse y nunca deben sustituirse por aproximaciones.

## Comprobación no destructiva

```powershell
.\target\release\lethe.exe usb inspect --letter F --expected-label JORGITO
```

Solo se continúa con la preparación si aparecen `USB_STATE=READY_FOR_FINAL_TEST`, `PHYSICAL_EXECUTION=REQUIRES_EXPLICIT_RESTORE_PLAN` y `CHANGES=NONE`, además de todos los campos de identidad esperados.

```powershell
.\target\release\lethe.exe usb restore-plan --letter F --expected-label JORGITO
```

El plan debe declarar `DATA_LOSS=ALL_CURRENT_CONTENT`, `RESULT_FILESYSTEM=exFAT`, `RESULT_LABEL=JORGITO` y `CHANGES=NONE`. La frase de confirmación no debe ejecutarse desde consola como una prueba rutinaria.

## Condiciones antes de habilitar una prueba destructiva

- Autorización explícita del usuario en el mismo momento de la prueba.
- Copia de seguridad confirmada.
- Ningún volumen Lethe o VeraCrypt montado.
- Revalidación inmediata de letra, etiqueta, disco, partición, tamaño, bus, serial y banderas de sistema.
- Coincidencia exacta con la identidad aprobada.
- Plan visible que describa la eliminación de particiones y el resultado exFAT/JORGITO.
- Frase escrita `ERASE-PHYSICAL-USB-JORGITO`, distinta de la usada para discos virtuales.
- Segunda revalidación dentro del ayudante elevado.
- Obtención del disco desde la partición correspondiente a la letra; nunca mediante un número escrito manualmente.
- Cancelación ante cualquier discrepancia, desconexión o salida ambigua de Windows.

## Criterios de aceptación posteriores

- GPT con una sola partición.
- exFAT con etiqueta `JORGITO`.
- Estado saludable.
- El medio se puede retirar y volver a conectar.
- No quedan puntos de montaje temporales.
- El panel y el ejecutable no permanecen en la memoria restaurada.
- Se documenta que la restauración lógica no equivale a borrado forense.
- Tras retirar y reconectar, Windows asigna una letra y conserva GPT/exFAT/JORGITO.

## Regla de parada

Si aparece otro disco, otra etiqueta, otra capacidad, otro serial o una bandera de arranque o sistema, la prueba se cancela sin intentar corregir automáticamente el objetivo.

## Resultado del 29 de septiembre de 2026

La candidata aprobada fue restaurada por última vez y reconectada como `E:`. Windows informó GPT, una partición, exFAT, etiqueta `JORGITO`, estado saludable y el mismo serial `057907B76060`. Las incidencias MBR vacía y reasignación de letra están documentadas en `SPRINT_7.md`.
