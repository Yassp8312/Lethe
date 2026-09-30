# Lethe

**Almacenamiento cifrado extraíble con negación plausible para Windows.**

[![Windows](https://img.shields.io/badge/plataforma-Windows-0078D4?logo=windows)](https://www.microsoft.com/windows)
[![Rust](https://img.shields.io/badge/nucleo-Rust-000000?logo=rust)](https://www.rust-lang.org/)
[![VeraCrypt](https://img.shields.io/badge/cifrado-VeraCrypt-0B5CAD)](https://www.veracrypt.fr/)
[![Version](https://img.shields.io/badge/version-0.1.0-315B7D)](Cargo.toml)
[![License](https://img.shields.io/badge/licencia-MIT-green)](Cargo.toml)

Lethe es una aplicación gratuita para Windows que simplifica el uso de un contenedor VeraCrypt con un **volumen exterior señuelo** y un **volumen oculto real**. Proporciona un panel pequeño para abrir, consultar y desmontar el almacenamiento, además de una restauración controlada para devolver la memoria USB a un formato convencional.

Lethe no implementa criptografía propia ni recibe contraseñas: el cuadro oficial de VeraCrypt solicita la clave y decide qué volumen puede abrirse.

## Tres resultados con un mismo contenedor

| Entrada | Resultado observable | Contenido |
|---|---|---|
| Contraseña incorrecta | No aparece ninguna unidad | Ninguno |
| Contraseña señuelo | Se monta el volumen exterior | Archivos normales preparados para mostrar |
| Contraseña real | Se monta el volumen oculto | Archivos privados |

Lethe no intenta identificar si VeraCrypt abrió el volumen exterior o el oculto. Esta indistinguibilidad es deliberada y evita crear información adicional que debilite la negación plausible.

## Funciones principales

- Apertura en **solo lectura por defecto**.
- Escritura disponible únicamente después de habilitarla y confirmar una advertencia.
- Contraseñas introducidas exclusivamente en VeraCrypt.
- Detección del estado real del montaje y de cambios en la letra de la memoria.
- Acceso directo a la unidad cifrada cuando está montada.
- Desmontaje normal o forzado y limpieza posterior de la caché de VeraCrypt.
- Diagnóstico y registro técnico sin secretos.
- Preparación transaccional de paquetes locales.
- Restauración controlada de VHD/VHDX y de la memoria USB autorizada.

> [!CAUTION]
> **Restaurar la memoria física elimina todo su contenido.** La operación exige identificar exactamente el dispositivo, aceptar dos confirmaciones y conceder permisos de administrador. No debe utilizarse para abrir archivos ni solucionar una contraseña olvidada.

## Estado del proyecto

La versión `0.1.0` completó los sprints 0–7 y fue validada en Windows con la memoria de prueba `JORGITO`:

- 45 pruebas unitarias aprobadas.
- Compilación `Release` correcta y Clippy sin advertencias.
- Contraseña incorrecta, volumen exterior y volumen oculto comprobados.
- Montaje, acceso, desmontaje, limpieza de caché y reconexión comprobados.
- Restauración física a GPT/exFAT y preparación posterior del contenedor comprobadas.

El proyecto es funcional como prototipo de investigación e ingeniería de seguridad. El ejecutable de desarrollo todavía no posee firma de código propia ni ha recibido una auditoría de seguridad externa.

## Requisitos

- Windows 10 versión 1809 o posterior, o Windows 11.
- [VeraCrypt](https://www.veracrypt.fr/en/Downloads.html) 1.26.29 o una versión compatible y firmada.
- Rust estable, únicamente para compilar desde el código fuente.
- PowerShell y .NET incluidos con Windows para ejecutar el panel.

## Inicio rápido para desarrollo

```powershell
git clone https://github.com/Yassp8312/Lethe.git
cd Lethe
cargo test
cargo build --release
```

Si VeraCrypt no está instalado en su ubicación convencional, indica su ruta para la sesión actual:

```powershell
$env:LETHE_VERACRYPT = 'D:\Trabajo\Programas\VeraCrypt\VeraCrypt.exe'
```

Para probar el panel desde el repositorio necesitas colocar `lethe.conf` y el contenedor `vault.hc` junto a `target\release\lethe.exe`. Después ejecuta:

```powershell
Copy-Item .\lethe.example.conf .\target\release\lethe.conf
powershell -ExecutionPolicy Bypass -File .\panel\Lethe.ps1
```

La creación del contenedor y de sus contraseñas se realiza con el asistente oficial de VeraCrypt. Consulta el [manual básico de usuario](entrega/Lethe-Final/Manual_de_usuario_Lethe.docx) antes de preparar o restaurar una memoria física.

## Uso cotidiano

1. Conecta la memoria que contiene la carpeta `Lethe`.
2. Ejecuta `Abrir Lethe.cmd` desde esa carpeta.
3. Pulsa **Abrir en modo seguro**.
4. Escribe la contraseña únicamente en el diálogo de VeraCrypt.
5. Utiliza la unidad virtual configurada —por defecto `L:`— para acceder a los archivos.
6. Cierra los archivos y pulsa **Desmontar volumen** antes de retirar la memoria.

Para modificar archivos privados, desmonta primero cualquier volumen abierto, habilita expresamente la escritura y utiliza la contraseña real. No se recomienda escribir en el volumen exterior sin la protección de volumen oculto apropiada de VeraCrypt.

## Interfaz de consola

| Comando | Función |
|---|---|
| `lethe doctor` | Comprueba configuración, contenedor y VeraCrypt. |
| `lethe status` | Informa el estado de máquina consumido por el panel. |
| `lethe drive-status` | Consulta la letra virtual configurada. |
| `lethe open-ro` | Abre el diálogo de VeraCrypt en solo lectura. |
| `lethe open-rw` | Abre el diálogo con escritura habilitada. |
| `lethe lock` | Desmonta y limpia la caché. |
| `lethe force-lock` | Fuerza el desmontaje; debe usarse solo tras cerrar los archivos. |
| `lethe provision ...` | Planifica o prepara un paquete local. |
| `lethe restore ...` | Planifica o ejecuta la restauración de un VHD/VHDX de prueba. |
| `lethe usb ...` | Inspecciona o restaura una USB física identificada expresamente. |

Las operaciones de restauración requieren reproducir la identidad capturada en un plan previo. No copies comandos destructivos entre dispositivos ni reutilices planes antiguos.

## Arquitectura

```text
Panel de Windows (PowerShell)
            │
            ▼
Núcleo de Lethe (Rust)
  ├── configuración y estados
  ├── validaciones de seguridad
  ├── auditoría sin secretos
  ├── preparación transaccional
  └── restauración controlada
            │
            ▼
VeraCrypt ──► contenedor vault.hc ──► unidad virtual L:
```

La contraseña no atraviesa el panel ni el núcleo de Lethe. VeraCrypt se ocupa de solicitarla, derivar las claves, descifrar y montar el volumen correspondiente.

## Estructura del repositorio

```text
src/                 Núcleo, CLI, integración con VeraCrypt y restauración
panel/               Panel gráfico de Windows
scripts/             Ayudantes de inspección y restauración
tests/               Pruebas de integración
test-data/            Configuración y datos de laboratorio no sensibles
docs/                 Diseño, amenazas, decisiones y cierres de sprint
entrega/Lethe-Final/  Manual, informe técnico y presentación final
tools/                Generadores de la documentación de entrega
```

## Modelo de seguridad

Lethe está diseñado para reducir errores operativos alrededor de VeraCrypt:

- No almacena ni registra contraseñas.
- No pasa secretos mediante argumentos, archivos o variables persistentes.
- Una contraseña incorrecta no modifica el contenedor ni activa borrado automático.
- El montaje seguro es de solo lectura.
- La restauración rechaza discos internos, de arranque o del sistema.
- La identidad del objetivo se comprueba antes y después de elevar privilegios.
- El registro se guarda fuera de la memoria y contiene únicamente eventos técnicos mínimos.

### Límites importantes

- Lethe no recupera contraseñas olvidadas.
- La existencia del contenedor y del software de cifrado puede ser observable.
- El cifrado protege la confidencialidad, no evita que alguien borre o corrompa el archivo.
- No protege frente a malware, keyloggers o un equipo anfitrión comprometido.
- La negación plausible no garantiza que un adversario acepte una explicación.
- Formatear una memoria flash no equivale a un borrado forense garantizado.

Consulta el [modelo de amenazas](docs/MODELO_DE_AMENAZAS.md) y las [decisiones de diseño](docs/DECISIONES.md) para conocer el alcance completo.

## Pruebas

```powershell
cargo test
cargo clippy --all-targets --all-features -- -D warnings
cargo build --release
```

Las pruebas destructivas se realizaron primero sobre discos virtuales. La ruta USB utiliza un flujo separado, requiere autorización explícita y revalida el dispositivo inmediatamente antes de escribir.

## Documentación

| Documento | Contenido |
|---|---|
| [Manual de usuario](entrega/Lethe-Final/Manual_de_usuario_Lethe.docx) | Uso básico desde conectar la memoria hasta desmontarla o restaurarla. |
| [Informe técnico](entrega/Lethe-Final/Informe_tecnico_Lethe_Final.docx) | Arquitectura, decisiones, incidentes, pruebas y justificación. |
| [Presentación final](entrega/Lethe-Final/Presentacion_Lethe_Final.pptx) | Resumen del proyecto en 9 diapositivas. |
| [Requisitos](docs/REQUISITOS.md) | Requisitos funcionales, de seguridad y no funcionales. |
| [Arquitectura](docs/ARQUITECTURA.md) | Componentes, dependencias y flujos críticos. |
| [Modelo de amenazas](docs/MODELO_DE_AMENAZAS.md) | Activos, adversarios, supuestos y exclusiones. |
| [Máquina de estados](docs/ESTADOS.md) | Estados válidos y transiciones del sistema. |
| [Recuperación](docs/RECUPERACION.md) | Procedimientos ante fallos e interrupciones. |
| [Prueba final USB](docs/PRUEBA_FINAL_USB.md) | Protocolo y criterios de la validación física. |

El historial de implementación está documentado en [`docs/SPRINT_0.md`](docs/SPRINT_0.md) hasta [`docs/SPRINT_7.md`](docs/SPRINT_7.md).

## Licencia

El paquete Rust declara licencia [MIT](Cargo.toml). VeraCrypt es un proyecto independiente y conserva sus propios términos de licencia.
