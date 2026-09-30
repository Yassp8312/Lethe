# Lethe

Lethe es un lanzador para Windows que abre un contenedor VeraCrypt con volumen exterior señuelo y volumen oculto real.

## Estado

Sprint 6 completado: Lethe incorpora registro técnico local sin secretos, inspección de identidad USB de solo lectura, pruebas de fallo y documentación de recuperación. La memoria física queda preparada para la prueba final, pero su restauración destructiva continúa deshabilitada.

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
.\target\release\lethe.exe usb inspect --letter F --expected-label JORGITO
```

## Seguridad

Lethe no recibe la contraseña. El diálogo de VeraCrypt se encarga de solicitarla. Una contraseña incorrecta no monta datos ni modifica el contenedor.

Durante el desarrollo, `provision` solo acepta destinos dentro del área local de trabajo. No debe apuntarse a una memoria USB hasta el sprint final de integración.

La restauración implementada acepta exclusivamente archivos `.vhd` o `.vhdx` dentro del área local de pruebas. `usb inspect` consulta solo la letra indicada y no modifica el dispositivo. No existe todavía un comando que restaure una USB física.
