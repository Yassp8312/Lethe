# Máquina de estados de Lethe

## Estados

| Estado | Descripción |
|---|---|
| `NoDevice` | No se encontró un objetivo Lethe válido. |
| `Detected` | Se encontró configuración y contenedor, aún sin montar. |
| `Prompting` | VeraCrypt está solicitando una contraseña. |
| `MountedReadOnly` | El volumen está montado sin escritura. |
| `MountedReadWrite` | El volumen está montado con escritura explícitamente habilitada. |
| `Unmounting` | Se solicitó el desmontaje y se espera su resultado. |
| `RestorePlanned` | Existe un plan ligado a una identidad exacta de dispositivo. |
| `Restoring` | El ayudante elevado está restaurando el dispositivo. |
| `ErrorSafe` | Ocurrió un error y no se autoriza una operación destructiva. |

## Transiciones principales

```mermaid
stateDiagram-v2
    [*] --> NoDevice
    NoDevice --> Detected: configuración válida encontrada
    Detected --> Prompting: abrir solo lectura o escritura
    Prompting --> Detected: contraseña inválida o cancelación
    Prompting --> MountedReadOnly: montaje seguro correcto
    Prompting --> MountedReadWrite: montaje con escritura correcto
    MountedReadOnly --> Unmounting: cerrar volumen
    MountedReadWrite --> Unmounting: cerrar volumen
    Unmounting --> Detected: desmontaje correcto
    Detected --> RestorePlanned: generar plan
    RestorePlanned --> Detected: cancelar o cambiar identidad
    RestorePlanned --> Restoring: confirmación y revalidación correctas
    Restoring --> NoDevice: restauración completada
    Detected --> ErrorSafe: validación fallida
    Prompting --> ErrorSafe: fallo de proceso
    Unmounting --> ErrorSafe: desmontaje fallido
    RestorePlanned --> ErrorSafe: identidad no coincide
    ErrorSafe --> NoDevice: objetivo ausente
    ErrorSafe --> Detected: diagnóstico corregido
```

## Invariantes

1. `Prompting` no contiene ni registra la contraseña.
2. Solo `MountedReadWrite` permite escritura deliberada.
3. No se puede pasar directamente de un estado montado a `Restoring`.
4. `Restoring` requiere un `RestorePlanned` válido y una segunda lectura de identidad coincidente.
5. Cualquier ambigüedad en el dispositivo conduce a `ErrorSafe`.
6. Una contraseña inválida vuelve a `Detected`; nunca conduce a restauración o borrado.
