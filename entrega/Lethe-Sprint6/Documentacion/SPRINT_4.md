# Sprint 4 — Preparador por consola

## Estado

Completado en almacenamiento local. La memoria USB `JORGITO` (`E:`) no fue utilizada.

## Implementación entregada

- Comando `provision plan` que valida y muestra ocho pasos sin escribir archivos.
- Comando `provision execute` que genera un paquete autocontenido.
- Comando `provision wizard` que abre `VeraCrypt Format.exe` desde el paquete.
- Copia de Lethe, panel, runtime validado de VeraCrypt y documentación.
- Generación de `lethe.conf`, manifiesto, lanzador y guía local.
- Construcción transaccional en una carpeta temporal hermana.
- Publicación mediante renombrado solo después de validar la estructura completa.
- Reversión de la carpeta temporal ante un fallo.
- Rechazo de destinos existentes, raíces de disco y rutas fuera del proyecto.
- Rechazo de enlaces simbólicos y tipos de archivo no regulares en las fuentes.
- Ausencia intencional de contraseñas y del contenedor `vault.hc` en el paquete inicial.

## Flujo de uso local

```powershell
$env:LETHE_VERACRYPT='D:\Trabajo\Programas\VeraCrypt\VeraCrypt.exe'
$target='D:\Proyectos\Lethe\staging\Mi-Lethe'
.\target\release\lethe.exe provision plan --target $target
.\target\release\lethe.exe provision execute --target $target
.\target\release\lethe.exe provision wizard --target $target
```

En el asistente oficial de VeraCrypt se selecciona:

1. Crear un contenedor de archivo cifrado.
2. Crear un volumen VeraCrypt oculto en modo normal.
3. Usar exactamente `<destino>\vault.hc` como ubicación.
4. Crear primero el volumen exterior con su contraseña señuelo.
5. Añadir únicamente archivos señuelo no sensibles cuando el asistente lo solicite.
6. Crear después el volumen oculto con una contraseña distinta y fuerte.

Las contraseñas se escriben exclusivamente en VeraCrypt. Si el asistente se cierra sin crear `vault.hc`, Lethe informa del fallo sin alterar el paquete.

## Comportamiento de las tres contraseñas

- Contraseña incorrecta: VeraCrypt no monta ninguna unidad; no hay borrado ni contador de intentos.
- Contraseña señuelo: se abre el volumen exterior.
- Contraseña real: se abre el volumen oculto.

Lethe no puede ni debe distinguir cuál de los dos volúmenes válidos fue abierto.

## Validación técnica

- [x] 30 pruebas unitarias aprobadas.
- [x] Análisis `clippy` sin advertencias.
- [x] Compilación Release correcta.
- [x] Plan local sin efectos laterales.
- [x] Paquete final publicado con 720 archivos, incluida esta documentación.
- [x] VeraCrypt portátil reconocido con versión 1.26.29 y firma válida.
- [x] Estado del paquete sin contenedor: `NOT_READY`.
- [x] Segundo intento rechazado sin cambiar el manifiesto existente.
- [x] `vault.hc` ausente y cero secretos guardados.
- [x] Ninguna operación sobre la USB física.

## Prueba manual pendiente del propietario

La apertura del asistente está implementada, pero la creación real del volumen requiere que el propietario elija y escriba ambas contraseñas directamente en VeraCrypt. Esa prueba no debe automatizarse ni delegarse a Lethe. Se realizará sobre el paquete local antes de cualquier integración con `E:`.

## Entrada del Sprint 5

El Sprint 5 implementará la restauración controlada, primero sobre un disco virtual: planificación, revalidación de identidad, ayudante temporal, desmontaje y formateo. Las operaciones destructivas sobre USB física continuarán bloqueadas hasta superar esas pruebas.
