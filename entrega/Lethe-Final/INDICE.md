# Entrega final de Lethe

Esta carpeta reúne la documentación y los artefactos de cierre del Sprint 7.

## Archivos principales

- `Informe_tecnico_Lethe_Final.docx`: informe técnico, decisiones de diseño, riesgos, pruebas, incidencias y resultado final.
- `Manual_de_usuario_Lethe.docx`: instrucciones básicas desde conectar la memoria hasta cerrarla, retirarla o restaurarla.
- `Presentacion_Lethe_Final.pptx`: presentación definitiva de 9 diapositivas para exponer el proyecto.
- `Documentacion/`: copia de la documentación Markdown del repositorio, incluidos los cierres de los sprints 0 a 7.

## Resultado verificado

- 45 pruebas unitarias aprobadas.
- Compilación Release correcta y Clippy sin advertencias.
- Volúmenes exterior y oculto creados y montados manualmente.
- Desmontaje y limpieza de caché completados.
- Restauración física completada y comprobada durante las pruebas.
- `JORGITO` quedó preparada nuevamente en `E:` con el release final y el contenedor cifrado `vault.hc` de 6 GiB.
- El montaje, desmontaje, la reconexión y la detección del estado real fueron validados desde el panel final.

## Límites

- Lethe no recupera contraseñas olvidadas.
- Una contraseña incorrecta no borra datos ni incrementa contadores.
- La restauración convencional de una memoria flash no garantiza borrado forense.
- El binario de desarrollo no tiene firma de código propia.
