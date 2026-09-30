# Sprint 5 — Restauración controlada

## Estado

Completado y validado sobre un disco VHDX local. La memoria USB `JORGITO` (`E:`) no fue utilizada.

## Implementación entregada

- Módulo `restore` con identidad inmutable del objetivo virtual.
- Comando `restore plan` sin escrituras ni montaje.
- Comando `restore execute` con identidad esperada y frase de confirmación.
- Revalidación antes de ejecutar y nuevamente dentro del ayudante.
- Restricción a imágenes `.vhd` y `.vhdx` dentro del área local de pruebas.
- Rechazo permanente de objetivos USB físicos en el backend de desarrollo.
- Ayudante PowerShell incorporado dentro de `Lethe.exe`.
- Elevación explícita mediante UAC desde el panel.
- Copia temporal del ejecutable para separar el proceso principal del restaurador.
- Limpieza del staging temporal aun cuando la operación falla.
- Verificación de bus virtual, disco no arrancable y no perteneciente al sistema.
- Creación de GPT, partición única, exFAT y etiqueta `JORGITO`.
- Punto de montaje temporal sin letra de unidad.
- Verificación posterior de sistema de archivos y etiqueta antes de declarar éxito.
- Botón de restauración habilitado en el panel solo cuando no hay un volumen VeraCrypt montado.

## Comandos

```powershell
.\target\release\lethe.exe restore plan `
  --virtual-image 'D:\Proyectos\Lethe\test-data\restore-lab.vhdx'
```

`execute` es un comando interno utilizado por el panel. Requiere reproducir exactamente los campos de identidad emitidos por `plan` y confirmar con `RESTORE-VIRTUAL-DISK`.

## Prueba destructiva aislada

Se creó `test-data\restore-lab.vhdx`, un VHDX fijo de 64 MiB con NTFS y etiqueta inicial `LETHE-BEFORE`. Lethe generó el plan, recibió elevación, limpió únicamente el disco obtenido desde esa imagen, creó la nueva estructura y la desmontó.

La verificación independiente posterior informó:

```text
VERIFY_OK=1
BUS_TYPE=File Backed Virtual
IS_BOOT=0
IS_SYSTEM=0
PARTITION_STYLE=GPT
FILESYSTEM=exFAT
LABEL=JORGITO
HEALTH=Healthy
```

## Incidencia detectada y corregida

La primera etiqueta propuesta, `LETHE-RESTORED`, fue rechazada por Windows en esta ruta de exFAT. El ayudante falló de forma segura antes de declarar éxito. Se sustituyó por `JORGITO`, se añadió verificación explícita y la prueba completa terminó correctamente.

También se eliminó la asignación temporal de letra, que provocaba que el Explorador intentara abrir una ubicación después del desmontaje. El diseño final usa una carpeta temporal como punto de montaje.

## Validación técnica

- [x] 35 pruebas unitarias aprobadas.
- [x] `clippy` sin advertencias.
- [x] Compilación Release correcta.
- [x] Sintaxis del panel PowerShell validada.
- [x] Plan sin cambios laterales.
- [x] Cambio de identidad cancela la ejecución.
- [x] Confirmación incorrecta cancela la ejecución.
- [x] Objetivo fuera del sandbox rechazado.
- [x] Objetivo USB físico rechazado.
- [x] Restauración real de VHDX completada.
- [x] Resultado GPT/exFAT/JORGITO saludable.
- [x] Imagen desmontada al finalizar.
- [x] Ninguna operación sobre `E:`.

## Limitaciones pendientes

- El binario de desarrollo todavía no posee firma de código propia.
- La restauración de USB física continúa deshabilitada.
- La integración física requerirá inventario y revalidación de letra, etiqueta, número de disco, capacidad, bus, serial y banderas de sistema/arranque.
- Restaurar una memoria flash significa volverla utilizable; no garantiza borrado forense absoluto por la nivelación de desgaste.

## Entrada del Sprint 6

El siguiente sprint debe endurecer y empaquetar el producto: registro técnico sin secretos, pruebas de fallos, documentación de recuperación, generación de entrega y preparación de la prueba final sobre la USB física. La habilitación física seguirá requiriendo una decisión explícita y controles adicionales.
