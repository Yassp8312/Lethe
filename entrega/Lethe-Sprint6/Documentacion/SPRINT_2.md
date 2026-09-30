# Sprint 2 — Integración segura con VeraCrypt

## Estado

Completado.

## Implementación entregada

- Localización de VeraCrypt mediante variable explícita, copia portátil o rutas estándar.
- Inspección de `VeraCrypt.exe` mediante firma Authenticode.
- Requisito de firmante IDRIX SARL.
- Validación de versión mínima 1.26.29.
- Comprobación de la existencia de `VeraCrypt Format.exe`.
- Construcción tipada de comandos de montaje, desmontaje y limpieza de caché.
- Prohibición comprobable de `/p`, `/password`, `/pim` y `/tokenpin`.
- Escritorio seguro, protección de memoria y protección de pantalla habilitados.
- Caché de contraseña e historial de volúmenes deshabilitados.
- Montaje de solo lectura y lectura-escritura diferenciados.
- Detección nativa de letras libres, VeraCrypt y otros dispositivos mediante `QueryDosDeviceW`.
- Rechazo de una letra de montaje previamente ocupada.
- Configuración local de integración mediante `LETHE_HOME`.

## Verificación automática

- [x] 24 pruebas unitarias.
- [x] Firma real de VeraCrypt validada.
- [x] Versión real 1.26.29 validada.
- [x] Firmante real IDRIX SARL validado.
- [x] `VeraCrypt Format.exe` localizado.
- [x] Letra `L:` consultada mediante la API real de Windows.
- [x] `cargo clippy --all-targets -- -D warnings` sin advertencias.
- [x] Compilación Release correcta.

## Validación manual

- [x] Crear `test-data/local-vault.hc` mediante el asistente gráfico.
- [x] Abrirlo desde Lethe en solo lectura.
- [x] Confirmar que una contraseña incorrecta no monta `L:`.
- [x] Abrirlo con la contraseña temporal correcta.
- [x] Confirmar que `L:` se identifica como `\Device\VeraCryptVolumeL`.
- [x] Confirmar que el montaje de solo lectura rechaza la creación de archivos.
- [x] Desmontarlo desde Lethe.
- [x] Limpiar la caché de VeraCrypt después del desmontaje.
- [x] Confirmar que `L:` vuelve a estar disponible.

La contraseña de integración debe introducirse directamente en VeraCrypt. No se automatiza mediante `/password`, aunque el contenedor sea descartable, para que el procedimiento de prueba coincida con la política del producto.

## Evidencia de la prueba negativa

El SHA-256 de `local-vault.hc` fue `8261FFC90EA1B81BAC091F55A5386B655BE3D393EE2AA729EFFB0F6402B9578C` antes y después de introducir una contraseña incorrecta. No apareció ningún mapeo para `L:`.

## Condición de cierre

La validación manual pasó completamente. La memoria USB física permaneció fuera de alcance y no fue modificada.

## Entrada del Sprint 3

El Sprint 3 integrará estos estados y operaciones en el panel: diagnóstico, apertura segura, apertura con escritura confirmada, desmontaje verificado y presentación coherente de errores.
