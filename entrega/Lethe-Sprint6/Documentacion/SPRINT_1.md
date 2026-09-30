# Sprint 1 — Núcleo y arquitectura interna

## Objetivo

Convertir el prototipo monolítico en un núcleo reutilizable y verificable, sin habilitar operaciones destructivas sobre memorias USB físicas.

## Implementación entregada

| Área | Archivo | Resultado |
|---|---|---|
| Tipos de dominio | `src/domain.rs` | Letras normalizadas, rutas confinadas e identidad de dispositivo. |
| Configuración | `src/config.rs` | Parser tipado, rechazo de claves duplicadas/desconocidas y ausencia de secretos. |
| Errores | `src/error.rs` | Códigos estructurados y mensajes públicos sin valores sensibles. |
| Estados | `src/state.rs` | Transiciones explícitas y rechazo de operaciones inválidas. |
| Almacenamiento | `src/storage.rs` | Objetivos locales, virtuales y físicos diferenciados. |
| Auditoría | `src/audit.rs` | Eventos estructurados que no admiten texto secreto arbitrario. |
| Biblioteca | `src/lib.rs` | Superficie modular reutilizable por consola, panel y pruebas. |
| Ejecutable | `src/main.rs` | Adaptado al núcleo nuevo sin perder los comandos existentes. |

## Barreras de seguridad incorporadas

- Las rutas de contenedor son relativas y no pueden usar `..`, raíz o prefijos externos.
- Las letras de unidad solo admiten un carácter ASCII alfabético y se normalizan a mayúscula.
- Una clave de configuración desconocida no imprime su valor.
- Los discos de sistema, arranque, tamaño cero o bus no USB se rechazan como candidatos físicos.
- Incluso una identidad USB válida recibe `PhysicalUsbDisabled` al solicitar una operación destructiva.
- La restauración no puede iniciarse desde un estado montado ni sin un plan previo.
- Una contraseña inválida vuelve al estado `Detected` sin transición destructiva.

## Pruebas

Se añadieron 18 pruebas unitarias que cubren:

- Parseo y validación de configuración.
- Prevención de escapes de ruta.
- Normalización de letras.
- Seguridad de mensajes de error.
- Transiciones normales e inválidas.
- Restauración únicamente después de planificación.
- Bloqueo incondicional de USB físicas.
- Rechazo de dispositivos del sistema.
- Eventos de auditoría estructurados.

## Comprobaciones de cierre

- [x] `cargo fmt --check`
- [x] `cargo test` — 18 aprobadas, 0 fallidas.
- [x] `cargo clippy --all-targets -- -D warnings`
- [x] `cargo build --release`
- [x] Sintaxis del panel PowerShell validada.
- [x] VeraCrypt 1.26.29 detectado en la ruta personalizada.
- [x] Ninguna escritura realizada sobre `JORGITO`.

## Resultado del diagnóstico

El diagnóstico reconoce correctamente VeraCrypt y devuelve `ContainerMissing` porque `vault.hc` todavía no ha sido creado. Este resultado es esperado antes de los sprints de integración y preparación.

## Entrada del Sprint 2

El Sprint 2 implementará un adaptador VeraCrypt aislado, detección y validación de versión/firma, construcción segura de comandos, detección del estado de montaje y pruebas con un contenedor local pequeño.
