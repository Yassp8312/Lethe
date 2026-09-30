# Manual de recuperación

## Objetivo

Este procedimiento devuelve un medio de laboratorio a un estado convencional GPT con una partición exFAT y etiqueta `JORGITO`. La versión actual admite VHD/VHDX locales y una USB física después de plan, confirmación y revalidación exacta.

## Antes de restaurar

1. Cierra cualquier volumen VeraCrypt desde el panel.
2. Comprueba que no haya archivos abiertos en el volumen.
3. Conserva una copia separada de los datos necesarios. La restauración elimina la estructura lógica del objetivo.
4. Para laboratorio, selecciona solo una imagen VHD o VHDX dentro de la carpeta del proyecto.
5. Lee el plan y confirma que la ruta y el tamaño correspondan al archivo de prueba.

## Restauración virtual desde el panel

1. Abre `panel\Lethe.ps1`.
2. Pulsa `Restaurar disco virtual de prueba`.
3. Selecciona la imagen de laboratorio.
4. Revisa el tamaño y el resultado esperado.
5. Acepta la confirmación de Windows cuando corresponda.
6. Espera el mensaje de restauración completada.

El ayudante vuelve a comprobar la identidad, verifica que Windows clasifique el disco como virtual, crea GPT y exFAT, comprueba la etiqueta y desmonta la imagen en todos los casos.

## Si una contraseña es incorrecta

VeraCrypt no monta ninguna unidad. Lethe no incrementa contadores, no borra información y no modifica el contenedor. Cierra el diálogo o vuelve a intentarlo. No existe recuperación de una contraseña perdida.

## Si el volumen no se cierra

Guarda y cierra los archivos abiertos. En el panel pulsa `Actualizar estado`: si indica `MOUNTED`, utiliza `Desmontar volumen`, aunque la ventana principal de VeraCrypt no muestre la unidad. El cierre forzado solo debe utilizarse después de cerrar aplicaciones, porque puede interrumpir escrituras pendientes. Lethe limpia la caché de contraseñas de VeraCrypt después de desmontar.

## Si Windows cambia la letra de la memoria

Mantén conectada una sola memoria preparada con Lethe y pulsa `Actualizar estado`. El panel busca una única carpeta `Lethe` válida entre `D:` y `Z:` y actualiza la ruta de trabajo. Si aparecen varias copias válidas, no elige una automáticamente: desconecta las demás y vuelve a actualizar.

## Si la restauración falla

- Un rechazo de identidad significa que el objetivo cambió después del plan. No continúes; crea un plan nuevo.
- Un rechazo de destino protege contra archivos externos, enlaces o discos no virtuales.
- Una cancelación de UAC deja la operación sin ejecutar.
- El ayudante temporal se elimina al finalizar. Si Windows conserva una carpeta `LetheRestore-*` en `%TEMP%`, verifica que no haya un proceso activo antes de eliminarla.
- Consulta `%LOCALAPPDATA%\Lethe\audit.log` para el código técnico. Ese archivo no contiene contraseñas ni identidad de la memoria.

## Límites de recuperación

Restaurar GPT y exFAT hace que el medio vuelva a ser utilizable, pero no garantiza el borrado forense de una memoria flash. La nivelación de desgaste puede conservar bloques fuera del alcance del sistema operativo. La confidencialidad depende del cifrado previo y de contraseñas adecuadas, no de presentar el formateo como borrado seguro.

## Recuperación de una interrupción física

Si el disco aprobado queda saludable pero sin particiones después de una restauración interrumpida, no se debe repetir una selección genérica. El reparador de contingencia exige el mismo número de disco, serial y capacidad, confirma bus USB y cero particiones, y completa GPT/exFAT sin volver a limpiar el disco. Esta ruta se creó y probó durante el Sprint 7.
