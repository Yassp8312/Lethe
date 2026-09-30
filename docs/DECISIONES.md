# Registro inicial de decisiones

## ADR-001: Windows como única plataforma inicial

Reduce la superficie de pruebas y permite terminar un producto funcional antes de abordar diferencias entre controladores y sistemas de archivos.

## ADR-002: VeraCrypt como motor criptográfico

Un formato criptográfico nuevo sería el mayor riesgo del proyecto. VeraCrypt es gratuito, de código abierto y ya implementa volúmenes ocultos, cifrado sectorial, derivación de claves y montaje en Windows.

## ADR-003: La contraseña no atraviesa Lethe

Lethe no usa el parámetro `/p` de VeraCrypt. Pasar una contraseña en argumentos podría exponerla en listados de procesos o registros. El diálogo de VeraCrypt obtiene la contraseña directamente.

## ADR-004: Un contenedor con volumen exterior y oculto

La contraseña señuelo y la real se aplican al mismo objeto cifrado. No se mantienen rutas o cabeceras separadas elegidas por Lethe.

## ADR-005: Escritura deshabilitada por defecto

Escribir en un volumen exterior sin protección puede dañar el volumen oculto. El panel abre en solo lectura por defecto y exige una acción adicional para habilitar escritura.

## ADR-006: Sin borrado por contraseña incorrecta

Los errores de contraseña no cambian el dispositivo, no crean contadores persistentes y no destruyen información. El resultado es simplemente que no aparece ninguna unidad virtual.

## ADR-007: Restauración iniciada desde Lethe

La restauración forma parte del panel. Un ayudante local separado existe únicamente porque Windows puede impedir formatear la unidad desde la que se ejecuta el proceso principal. El usuario no tendrá que localizar ni ejecutar manualmente ese ayudante.

## ADR-008: La USB física se utiliza al final

El desarrollo y las pruebas destructivas se realizan primero con contenedores locales y discos virtuales. Las operaciones sobre USB físicas permanecen bloqueadas hasta completar las pruebas de seguridad y restauración.

## ADR-009: Contenedor fijo alojado en un sistema de archivos normal

La primera versión utiliza un archivo `vault.hc` de tamaño fijo. Este diseño permite desarrollar, respaldar y restaurar el dispositivo con menos riesgo que un formato de particiones propio. La existencia de cifrado no se intenta ocultar; la propiedad buscada es que no pueda demostrarse el contenido del volumen oculto a partir del contenedor por sí solo.

## ADR-010: Dependencias gratuitas

El proyecto usa Rust, componentes incluidos con Windows y VeraCrypt. No se aceptan servicios, licencias o bibliotecas que requieran pago para compilar o utilizar la versión básica.

## ADR-011: Seguridad antes que escritura del señuelo

El montaje predeterminado es de solo lectura. Permitir escritura en el volumen exterior sin protección puede dañar el volumen oculto. La interfaz exige una confirmación adicional para habilitar escritura y documenta que un adversario con control físico aún puede destruir la disponibilidad de los datos.

## ADR-012: Preparación transaccional y sin sobrescritura

El paquete se construye en una carpeta temporal hermana, se valida y se publica mediante un renombrado local. Un destino existente se rechaza. Esto evita mezclar una ejecución fallida con datos anteriores y permite revertir únicamente los archivos creados por la operación actual.

## ADR-013: Contraseñas exclusivamente en VeraCrypt

El preparador no acepta contraseñas por consola, archivos, variables de entorno ni configuración. `provision wizard` ejecuta el asistente gráfico oficial y solo comprueba después que `vault.hc` exista. Así se evita exponer secretos en argumentos, historial o registros de Lethe.

## ADR-014: Destino local restringido durante el desarrollo

Hasta el sprint de integración física, la preparación solo admite carpetas descendientes del área local de trabajo. Las rutas raíz, destinos existentes, enlaces simbólicos de las fuentes y rutas externas se rechazan. Esta restricción reduce el riesgo de modificar por accidente la memoria USB o una carpeta ajena.

## ADR-015: Restauración virtual antes de USB física

El primer backend destructivo acepta únicamente archivos VHD/VHDX locales. La interfaz no recibe letras ni números de disco. El objeto que se limpia se obtiene directamente desde la imagen montada y debe ser clasificado por Windows como virtual, no arrancable y no perteneciente al sistema.

## ADR-016: Identidad ligada al plan

El plan captura ruta canónica, tamaño y marcas de creación/modificación. La ejecución compara esos valores antes de elevarse y el ayudante vuelve a comparar tamaño y modificación inmediatamente antes del montaje. Cualquier diferencia cancela la operación.

## ADR-017: Punto de montaje sin letra durante restauración

La partición restaurada se formatea y verifica mediante un punto de montaje temporal en una carpeta. No se asigna una letra de unidad, evitando ventanas automáticas del Explorador y reduciendo estados transitorios visibles.

## ADR-018: Ayudante efímero y elevación explícita

El panel copia el ejecutable a una carpeta temporal única, muestra el plan y solicita confirmación antes de iniciarlo con elevación de Windows. El binario incorpora el script de restauración y elimina sus archivos temporales al terminar. La firma de código del ejecutable de producción queda como requisito de empaquetado; las compilaciones actuales de desarrollo no están firmadas.

## ADR-019: Inspección física dirigida por letra

La preparación de la prueba final no enumera todas las unidades. El usuario indica una letra y Lethe obtiene desde ella el volumen, la única partición y el disco asociado. Después exige etiqueta, bus USB, capacidad, serial y banderas seguras. Esto reduce el riesgo de confundir la memoria con otro disco, sin presentar la inspección como autorización para formatear.

## ADR-020: Ejecución física ausente en el Sprint 6

Una identidad válida no habilita operaciones destructivas. La abstracción `PhysicalUsbTarget` continúa devolviendo `PhysicalUsbDisabled` y la interfaz de consola solo ofrece `usb inspect`. La activación física se reserva para una prueba final separada con confirmación explícita y revalidación inmediata.

## ADR-021: Registro local mínimo y no vinculante

El registro se guarda en `%LOCALAPPDATA%\Lethe`, no en la memoria extraíble. Solo contiene tiempo, ciclo de la aplicación y códigos de error definidos por el programa. No registra secretos ni identidad del dispositivo. La escritura se trata como mejor esfuerzo para que un problema de auditoría no bloquee el acceso o el cierre de datos.

## ADR-022: Ruta física dedicada sin habilitar el backend genérico

La abstracción general de almacenamiento sigue rechazando operaciones destructivas sobre USB. La restauración física se implementa como un flujo separado que solo acepta una identidad producida por su propio plan. Esto evita que otra operación futura herede accidentalmente permiso para borrar dispositivos físicos.

## ADR-023: Confirmación escrita y pérdida total explícita

El plan declara `DATA_LOSS=ALL_CURRENT_CONTENT`. El panel muestra letra, etiqueta, disco, capacidad y serial, solicita una aceptación y después exige escribir `ERASE-PHYSICAL-USB-JORGITO`. La frase virtual no se reutiliza, de modo que una automatización de laboratorio no autoriza una memoria real.

## ADR-024: Disco derivado siempre desde la letra

Ni el panel ni el ayudante seleccionan un disco enumerando todos los dispositivos. La letra indicada resuelve una partición y esa partición resuelve el disco. El número de disco capturado funciona únicamente como una comprobación de igualdad; nunca se usa para sustituir un objetivo que dejó de coincidir.

## ADR-025: Ejecución local antes de borrar el medio

El panel copia el ejecutable a una carpeta temporal local y utiliza esa copia para solicitar elevación. El ayudante trabaja desde la carpeta temporal, no desde la memoria que será restaurada. Después del resultado, la interfaz se cierra para que el medio pueda reconectarse y recibir una letra nueva.

## ADR-026: Conversión explícita del disco vacío a GPT

`Clear-Disk` puede eliminar todas las particiones sin cambiar el estilo MBR a RAW. El ayudante inspecciona el estilo después de limpiar: inicializa GPT si es RAW, convierte con `Set-Disk` si continúa en MBR y no hace conversión si ya es GPT. Esta decisión surge de la prueba física y evita repetir la interrupción observada.

## ADR-027: Recuperación limitada a una restauración interrumpida

El reparador de contingencia solo acepta el número, capacidad y serial aprobados, exige bus USB, disco saludable, no arrancable, no perteneciente al sistema y exactamente cero particiones. No vuelve a ejecutar `Clear-Disk`; únicamente termina GPT, partición y exFAT. Así se recupera el estado intermedio sin ampliar el objetivo autorizado.

## ADR-028: Reconexión después de la restauración física

El ayudante verifica el resultado mediante un punto de montaje temporal y termina sin intentar reutilizar la letra original. El panel puede estar ejecutándose desde esa letra y mantenerla ocupada aun después de eliminar la partición anterior. La asignación produjo un falso fallo después de haber completado GPT y exFAT. El flujo definitivo declara éxito tras verificar el volumen, cierra el panel y pide desconectar y reconectar para que Windows asigne una letra limpia.

## ADR-029: El panel usa el estado del controlador como fuente de verdad

La ventana principal de VeraCrypt puede no refrescar inmediatamente un volumen iniciado por otro proceso. Lethe consulta el espacio de nombres de unidades de Windows mediante su comando `status`, actualiza el panel periódicamente y ofrece actualización manual, acceso a la carpeta cifrada y desmontaje desde la misma interfaz. El panel fija además la misma copia validada de VeraCrypt para abrir, consultar y desmontar. Si la memoria cambia de letra mientras el panel continúa abierto, se admite la redetección únicamente cuando existe una sola carpeta `Lethe` válida entre `D:` y `Z:`.
