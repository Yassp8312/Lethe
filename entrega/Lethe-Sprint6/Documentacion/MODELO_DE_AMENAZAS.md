# Modelo de amenazas

## Activos protegidos

- Confidencialidad de los archivos del volumen oculto.
- Separación entre el contenido señuelo y el contenido real.
- Contraseñas y claves derivadas.
- Integridad operacional del contenedor durante el uso normal.
- Identidad del dispositivo durante preparación y restauración.

## Adversarios contemplados

1. Persona que encuentra o roba la memoria apagada y desconectada.
2. Analista con una única imagen del contenedor cifrado y sin contraseña real.
3. Persona que conoce la contraseña señuelo y examina el volumen exterior.
4. Usuario o programa que introduce contraseñas incorrectas repetidamente.
5. Error humano al seleccionar un dispositivo durante la restauración.

## Garantías buscadas

- Sin una contraseña válida no se monta ningún contenido.
- La contraseña señuelo abre un conjunto coherente de archivos exteriores.
- La contraseña real abre el volumen oculto.
- Lethe no distingue ni registra cuál de las dos contraseñas válidas se utilizó.
- Un intento inválido no cambia el contenedor.
- La selección incorrecta de un disco se detiene antes de cualquier escritura.
- El cifrado y la derivación de claves corresponden al formato mantenido por VeraCrypt.

## Amenazas fuera de alcance

- Equipo anfitrión comprometido, keylogger o captura de pantalla.
- Extracción de claves desde RAM mientras el volumen está montado.
- Ataques de arranque en frío, DMA o hardware especializado sobre el equipo anfitrión.
- Adversario que obtiene imágenes del dispositivo en distintos momentos y compara cambios.
- Adversario que obliga a escribir grandes cantidades de datos en el volumen señuelo.
- Destrucción física, borrado o corrupción deliberada del dispositivo.
- Copias previas, respaldos o imágenes forenses creadas antes de una restauración.
- Consecuencias legales o físicas de negarse a revelar información.
- Vulnerabilidades desconocidas de Windows, VeraCrypt o el firmware de la memoria.

## Límites de la negación plausible

Lethe no promete negar la existencia de cifrado: el programa y `vault.hc` son visibles. La propiedad pretendida es más limitada: revelar la contraseña exterior no entrega automáticamente la contraseña ni el contenido del volumen oculto.

La negación plausible depende del comportamiento del usuario, del estado del equipo anfitrión y de que no existan observaciones históricas contradictorias. No se describe como garantía matemática de persuasión frente a un adversario.

## Riesgos residuales

| Riesgo | Tratamiento |
|---|---|
| Escritura exterior sobrescribe el volumen oculto. | Solo lectura por defecto y advertencia antes de habilitar escritura. |
| Pérdida de ambas contraseñas. | No existe puerta trasera; se documentará la necesidad de respaldo personal seguro. |
| Corrupción o desconexión inesperada. | Desmontaje explícito, pruebas de interrupción y copia de cabeceras recomendada. |
| Manipulación de VeraCrypt. | Validación de firma y versión antes de invocarlo. |
| Selección del disco equivocado. | Identidad compuesta, doble validación y modo `Plan`. |
| Rastros en el host. | Se minimizan los rastros propios, sin prometer eliminar los generados por Windows o aplicaciones. |

## Supuestos

- Windows y VeraCrypt no están comprometidos.
- Las contraseñas son largas, diferentes y no relacionadas.
- El usuario desmonta el volumen antes de retirar la memoria.
- La contraseña real no se introduce bajo observación del adversario.
- Los documentos sensibles se abren con aplicaciones que el usuario considera confiables.
