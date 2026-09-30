# Lethe

Lethe es un lanzador para Windows que abre un contenedor VeraCrypt con volumen exterior señuelo y volumen oculto real.

## Estado

Sprint 7 completado: se desplegó el paquete, se creó un contenedor VeraCrypt de 6 GiB con volumen exterior y volumen oculto, se probó el montaje y se restauró finalmente la memoria a GPT/exFAT/JORGITO. El código local incorpora las correcciones encontradas durante ambas restauraciones físicas.

## Documentación de diseño

- [Requisitos](docs/REQUISITOS.md)
- [Modelo de amenazas](docs/MODELO_DE_AMENAZAS.md)
- [Máquina de estados](docs/ESTADOS.md)
- [Arquitectura](docs/ARQUITECTURA.md)
- [Decisiones](docs/DECISIONES.md)
- [Cierre del Sprint 0](docs/SPRINT_0.md)
- [Cierre del Sprint 1](docs/SPRINT_1.md)
- [Cierre del Sprint 2](docs/SPRINT_2.md)
- [Cierre del Sprint 3](docs/SPRINT_3.md)
- [Cierre del Sprint 4](docs/SPRINT_4.md)
- [Cierre del Sprint 5](docs/SPRINT_5.md)
- [Cierre del Sprint 6](docs/SPRINT_6.md)
- [Cierre del Sprint 7](docs/SPRINT_7.md)
- [Manual de recuperación](docs/RECUPERACION.md)
- [Protocolo de prueba final USB](docs/PRUEBA_FINAL_USB.md)

## Requisitos de desarrollo

- Windows 10 1809 o posterior
- Rust estable
- VeraCrypt para realizar pruebas de integración

La versión estable debe obtenerse desde la [página oficial de VeraCrypt](https://veracrypt.io/en/Downloads.html). Lethe no redistribuye binarios del motor.

## Comandos

```powershell
cargo run -- doctor
cargo run -- open-ro
cargo run -- open-rw
cargo run -- lock
cargo build --release
powershell -ExecutionPolicy Bypass -File .\panel\Lethe.ps1
$env:LETHE_VERACRYPT='D:\Trabajo\Programas\VeraCrypt\VeraCrypt.exe'
.\target\release\lethe.exe provision plan --target 'D:\Proyectos\Lethe\staging\Mi-Lethe'
.\target\release\lethe.exe provision execute --target 'D:\Proyectos\Lethe\staging\Mi-Lethe'
.\target\release\lethe.exe provision wizard --target 'D:\Proyectos\Lethe\staging\Mi-Lethe'
.\target\release\lethe.exe restore plan --virtual-image 'D:\Proyectos\Lethe\test-data\restore-lab.vhdx'
.\target\release\lethe.exe usb inspect --letter E --expected-label JORGITO
.\target\release\lethe.exe usb restore-plan --letter E --expected-label JORGITO
```

## Seguridad

Lethe no recibe la contraseña. El diálogo de VeraCrypt se encarga de solicitarla. Una contraseña incorrecta no monta datos ni modifica el contenedor.

`provision` solo acepta destinos dentro del área local de trabajo. Para preparar una USB se genera primero el paquete local y luego se copia de forma controlada.

La restauración virtual acepta exclusivamente archivos `.vhd` o `.vhdx` dentro del área local de pruebas. La ruta física solo puede partir de una letra explícita, requiere reproducir toda la identidad del plan y una frase de pérdida de datos, y revalida nuevamente dentro del ayudante elevado. No debe ejecutarse sin autorización inmediata y copia de seguridad.
