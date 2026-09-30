# Sprint 0 — Especificación y seguridad

## Objetivo

Eliminar ambigüedades antes de ampliar la implementación y establecer criterios verificables para acceso, montaje, restauración y pruebas.

## Entregables

- [x] Alcance de Lethe 0.1 definido.
- [x] Requisitos funcionales identificados.
- [x] Requisitos de seguridad identificados.
- [x] Exclusiones explícitas documentadas.
- [x] Modelo de amenazas documentado.
- [x] Estados y transiciones definidos.
- [x] Arquitectura lógica definida.
- [x] Restauración integrada reconciliada con las restricciones de Windows.
- [x] Prohibición temporal de utilizar la USB física documentada.
- [x] Decisiones arquitectónicas registradas.

## Criterios de aceptación

1. Los tres resultados de contraseña tienen un comportamiento inequívoco.
2. Ningún requisito exige que Lethe conozca una contraseña.
3. Una contraseña incorrecta no provoca escrituras ni borrado.
4. La restauración se inicia desde el panel y requiere confirmación explícita.
5. La restauración rechaza discos internos, de arranque o de sistema.
6. Las promesas de seguridad distinguen confidencialidad, disponibilidad y borrado forense.
7. Las pruebas físicas con `JORGITO` están aplazadas hasta completar discos virtuales.
8. No existe una dependencia de pago en el diseño base.

## Definición de terminado

El Sprint 0 se considera terminado cuando los documentos de requisitos, amenazas, estados, arquitectura y decisiones son coherentes entre sí y pueden convertirse directamente en tareas del Sprint 1.

## Entrada del Sprint 1

El Sprint 1 implementará `lethe-core`: configuración tipada, estados, validaciones, errores seguros y una abstracción de almacenamiento que mantendrá deshabilitadas las operaciones sobre USB físicas.
