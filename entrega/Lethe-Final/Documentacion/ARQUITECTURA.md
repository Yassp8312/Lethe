# Arquitectura objetivo de Lethe

## Alcance inicial

Lethe 0.1 funciona exclusivamente en Windows. Usa VeraCrypt como motor criptográfico y de montaje. Lethe no implementa algoritmos criptográficos ni recibe contraseñas: solicita a VeraCrypt que abra el contenedor y deja que el motor muestre su propio diálogo seguro.

El sistema se divide en cuatro componentes lógicos:

- `lethe-core`: configuración, estados, validaciones y reglas de seguridad.
- `lethe-veracrypt`: localización, verificación e invocación de VeraCrypt.
- `lethe-ui`: panel de apertura, cierre, diagnóstico y restauración.
- `lethe-restore`: ayudante temporal que ejecuta la restauración después de cerrar el programa principal.

La implementación actual del núcleo distribuye esas responsabilidades en:

```text
src/
├── lib.rs       Biblioteca de Lethe
├── domain.rs    Tipos seguros e identidad de dispositivos
├── config.rs    Configuración validada
├── error.rs     Errores y códigos públicos
├── state.rs     Máquina de estados
├── storage.rs   Abstracción del almacenamiento
├── provision.rs Preparación transaccional de paquetes locales
├── restore.rs   Planificación, identidad y restauración virtual
├── audit.rs     Eventos técnicos sin secretos
├── usb.rs       Inspección dirigida y no destructiva de una USB física
└── main.rs      Interfaz de consola existente
```

Durante el desarrollo, el almacenamiento se representó mediante objetivos intercambiables: contenedor local, disco virtual y USB física. La USB física permaneció deshabilitada hasta superar las pruebas con discos virtuales; el Sprint 7 habilitó su ruta específica con revalidación elevada.

## Preparación transaccional

El preparador solo publica un paquete cuando toda su estructura ha sido copiada y validada. Primero trabaja en una carpeta hermana identificada como staging y finalmente la renombra al destino. Ante un fallo elimina únicamente esa carpeta temporal previamente verificada. Un destino existente nunca se mezcla ni se sobrescribe.

La versión de desarrollo restringe el destino al árbol local del proyecto. El paquete incluye Lethe, el panel, la copia validada de VeraCrypt, configuración y documentación, pero no crea contraseñas ni incluye `vault.hc`. La creación del volumen se delega al asistente oficial de VeraCrypt mediante `provision wizard`.

## Tres resultados de acceso

Un solo contenedor contiene un volumen exterior y un volumen oculto:

1. Una contraseña no válida no monta ninguna unidad.
2. La contraseña señuelo abre el volumen exterior.
3. La contraseña real abre el volumen oculto.

Lethe no conoce cuál de los dos volúmenes fue abierto. Esto reduce las diferencias observables entre los flujos de acceso.

## Modos de montaje

- Solo lectura: opción predeterminada para consultar información y abrir el volumen señuelo sin modificar el contenedor.
- Lectura y escritura: requiere una confirmación explícita en el panel. Debe reservarse para editar el volumen real.

Un adversario con control físico puede corromper o borrar el contenedor. El cifrado protege la confidencialidad, no la disponibilidad.

## Recuperación de la memoria

La restauración a una memoria convencional se inicia desde el panel de Lethe. Como el programa puede estar ejecutándose desde la propia memoria, el panel copia un ayudante firmado a una carpeta temporal local, transmite únicamente la identidad del dispositivo y cierra el proceso principal antes de modificar el volumen.

La restauración dispone de dos modos internos:

- `Plan`: enumera las operaciones sin modificar el dispositivo.
- `Execute`: ejecuta el plan después de validar nuevamente la identidad y recibir confirmación explícita.

Antes de modificar particiones se comprobarán letra, etiqueta, número de disco, tamaño, bus USB y que no sea disco de arranque o sistema. Cualquier cambio entre la planificación y la ejecución cancela la operación. La restauración lógica no se describirá como borrado forense garantizado en memorias flash.

### Implementación del Sprint 5

La primera implementación solo admite archivos VHD/VHDX dentro del área local de pruebas. El plan captura ruta canónica, tamaño, creación y última modificación sin montar ni escribir la imagen. La ejecución exige la frase `RESTORE-VIRTUAL-DISK` y compara nuevamente todos esos campos.

Tras elevarse, el ayudante vuelve a comprobar la identidad del archivo, monta exactamente esa imagen y obtiene el objeto de disco desde `Mount-DiskImage`. Solo continúa si Windows informa un único disco `File Backed Virtual` o `Virtual`, no arrancable y no perteneciente al sistema. Después crea GPT, una partición exFAT con etiqueta `JORGITO`, verifica el resultado y desmonta la imagen en un bloque `finally`.

El formateo se realiza sin asignar letra: se usa temporalmente un punto de montaje en una carpeta local. Esto evita que el Explorador abra una unidad destinada a desaparecer al finalizar.

### Preparación física del Sprint 6

`usb inspect` recibe una letra concreta y consulta únicamente el volumen, la partición y el disco derivados de esa letra. No enumera ni selecciona otros discos. La identidad se acepta solo cuando hay una partición, bus USB, capacidad no nula, número de serie, etiqueta esperada y ausencia de las banderas de arranque y sistema.

El resultado de inspección declara `PHYSICAL_EXECUTION=REQUIRES_EXPLICIT_RESTORE_PLAN` y `CHANGES=NONE`. La inspección crea solamente un script temporal local y un registro técnico en `%LOCALAPPDATA%\Lethe`; no escribe en la memoria inspeccionada.

### Restauración física dedicada del Sprint 7

La restauración física no reutiliza el permiso genérico de objetivos destructivos, que continúa rechazando USB. Solo una ruta dedicada puede continuar después de crear un plan completo. El panel exige una primera aceptación visual y después la frase exacta `ERASE-PHYSICAL-USB-JORGITO`.

El panel copia `Lethe.exe` a `%TEMP%` antes de elevarlo. El proceso local recibe letra, etiqueta, sistema de archivos, disco, partición, capacidad y serial capturados por el plan. Rust vuelve a inspeccionar la letra y compara todos los campos. El ayudante PowerShell resuelve nuevamente volumen → partición → disco y repite todas las comprobaciones antes de su primera instrucción destructiva.

Solo después de esas barreras el ayudante limpia el disco exacto, crea GPT, una partición exFAT y la etiqueta `JORGITO`. Usa un punto de montaje temporal sin letra y comprueba sistema de archivos, etiqueta, estilo de partición y número de disco. El panel se cierra al terminar para permitir retirar y reconectar la memoria.

Después de limpiar, el ayudante distingue tres estados: inicializa un disco RAW, convierte un disco vacío MBR mediante `Set-Disk` o conserva GPT. Esta rama se añadió tras observar que `Clear-Disk` puede eliminar particiones sin producir RAW. Al terminar elimina el punto de montaje temporal, cierra el panel y requiere reconectar el medio para recibir una letra nueva.

Existe una recuperación estrecha para el estado intermedio sin particiones. Solo opera sobre el número, serial y capacidad aprobados, exige bus USB y cero particiones, y no ejecuta una segunda limpieza. Su función es completar GPT, exFAT y etiqueta después de una interrupción ya identificada.

## Registro técnico

Lethe registra una marca de tiempo, el inicio, el cierre y los códigos estructurados de operaciones rechazadas. El archivo se guarda en `%LOCALAPPDATA%\Lethe\audit.log`, fuera del paquete extraíble. No contiene contraseñas, argumentos, rutas, letras, etiquetas, números de serie ni contenido del usuario. Un fallo del registro no impide abrir o cerrar un volumen cifrado.

## Flujo de dependencias

```text
lethe-ui
  ├── lethe-core
  ├── lethe-veracrypt
  └── lethe-restore (solo al restaurar)

lethe-veracrypt
  └── VeraCrypt 1.26.29 o versión compatible validada
```
