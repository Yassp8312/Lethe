# Requisitos de Lethe 0.1

## Alcance del producto

Lethe 0.1 es una aplicación gratuita para Windows que administra un contenedor VeraCrypt con un volumen exterior señuelo y un volumen oculto real. Ofrece preparación por consola, acceso mediante un panel pequeño y restauración de la memoria a un volumen convencional desde el propio panel.

## Requisitos funcionales

| ID | Requisito | Criterio observable |
|---|---|---|
| RF-001 | Funcionar en Windows 10 1809 o posterior y Windows 11. | El binario Release inicia y completa `doctor`. |
| RF-002 | Detectar VeraCrypt y validar que el ejecutable exista. | `doctor` informa la ruta y versión encontradas. |
| RF-003 | No solicitar ni almacenar contraseñas dentro de Lethe. | La contraseña se introduce únicamente en el diálogo de VeraCrypt. |
| RF-004 | Preparar el dispositivo mediante comandos de consola. | `provision` crea la estructura y guía la creación de ambos volúmenes. |
| RF-005 | Abrir el contenedor en solo lectura por defecto. | Aparece una unidad virtual sin capacidad de escritura. |
| RF-006 | Permitir apertura con escritura tras confirmación explícita. | La acción no está disponible accidentalmente con un solo clic. |
| RF-007 | Desmontar el volumen desde el panel. | La letra virtual desaparece al cerrar correctamente. |
| RF-008 | Manejar una contraseña inválida sin modificar datos. | No aparece una unidad y el contenedor conserva su hash o bloques de prueba. |
| RF-009 | Abrir el volumen exterior con la contraseña señuelo. | Solo aparecen los archivos señuelo. |
| RF-010 | Abrir el volumen oculto con la contraseña real. | Solo aparecen los archivos privados. |
| RF-011 | Detectar cambios de letra de la memoria. | Lethe localiza su configuración sin depender de `E:`. |
| RF-012 | Mostrar diagnóstico sin incluir secretos. | El informe contiene rutas, estados y versiones, pero no contraseñas. |
| RF-013 | Iniciar la restauración desde el panel. | El usuario no ejecuta manualmente otra herramienta. |
| RF-014 | Mostrar un plan de restauración antes de ejecutarlo. | El modo `Plan` no produce escrituras. |
| RF-015 | Restaurar el dispositivo como almacenamiento convencional. | Tras la operación, Windows reconoce una partición exFAT utilizable. |
| RF-016 | Cancelar la restauración ante cualquier cambio de identidad. | Cambiar disco, tamaño, etiqueta o bus invalida el plan. |

## Requisitos de seguridad

| ID | Requisito |
|---|---|
| RS-001 | Ninguna contraseña se pasa en argumentos de proceso, archivos, variables persistentes o registros de Lethe. |
| RS-002 | Lethe no implementa primitivas criptográficas ni un formato criptográfico propio. |
| RS-003 | Una contraseña incorrecta no incrementa contadores, borra datos ni escribe en el contenedor. |
| RS-004 | El montaje predeterminado es de solo lectura. |
| RS-005 | La restauración exige bus USB y rechaza discos de arranque, sistema o internos. |
| RS-006 | La identidad del dispositivo se valida antes y después de pedir confirmación. |
| RS-007 | Las operaciones destructivas no admiten comodines ni destinos derivados de texto sin validar. |
| RS-008 | Los binarios externos se verifican mediante firma digital o hash conocido antes de usarlos. |
| RS-009 | Las pruebas automatizadas no pueden habilitar operaciones sobre USB físicas. |
| RS-010 | Los errores visibles no revelan si se esperaba una contraseña real, señuelo o distinta. |

## Requisitos no funcionales

- RNF-001: todas las dependencias necesarias para el uso básico son gratuitas.
- RNF-002: la interfaz debe ser utilizable sin conocimientos de consola para abrir, cerrar o restaurar.
- RNF-003: la preparación puede realizarse por consola.
- RNF-004: el proyecto debe compilarse con Rust estable.
- RNF-005: la configuración debe ser legible y no contener secretos.
- RNF-006: cada operación destructiva debe disponer de modo de simulación.
- RNF-007: el paquete final debe incluir manual, arquitectura, decisiones, amenazas, pruebas, informe y presentación.

## Exclusiones de la versión 0.1

- Linux y macOS.
- Cifrado de la partición del sistema operativo.
- Borrado automático por contraseñas fallidas.
- Autodestrucción silenciosa.
- Ocultación de que existe software de cifrado.
- Garantía de ausencia de rastros en el equipo anfitrión.
- Garantía de borrado forense absoluto en memorias flash.
- Protección contra un equipo infectado o un keylogger.
- Sincronización en la nube o recuperación remota de contraseñas.

## Restricción de desarrollo

Hasta completar las pruebas destructivas sobre VHD/VHDX, ningún comando de desarrollo puede particionar, formatear o escribir estructuras de Lethe en una USB física. `JORGITO` se reserva para la prueba final de integración.
