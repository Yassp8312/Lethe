use std::{fmt, io, path::PathBuf};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ErrorCode {
    ConfigInvalid,
    CommandUnknown,
    CommandArgumentsInvalid,
    PathUnsafe,
    DriveLetterInvalid,
    PlatformUnsupported,
    DependencyMissing,
    DependencyInspectionFailed,
    SignatureInvalid,
    VersionUnsupported,
    ContainerMissing,
    DriveOccupied,
    DriveInspectionFailed,
    VolumeNotMounted,
    DriveStillMounted,
    ProcessStartFailed,
    ProcessFailed,
    TransitionInvalid,
    PhysicalUsbDisabled,
    DeviceUnsafe,
    ProvisionTargetUnsafe,
    ProvisionTargetExists,
    ProvisionSourceMissing,
    ProvisionFailed,
    RestoreTargetUnsafe,
    RestoreIdentityChanged,
    RestoreConfirmationInvalid,
    RestoreFailed,
    PhysicalRestoreIdentityChanged,
    PhysicalRestoreConfirmationInvalid,
    PhysicalRestoreFailed,
    IoFailed,
}

#[derive(Debug, Eq, PartialEq)]
pub enum LetheError {
    InvalidConfig {
        line: usize,
        reason: &'static str,
    },
    UnknownConfigKey {
        line: usize,
        key: String,
    },
    UnknownCommand,
    InvalidCommandArguments(&'static str),
    UnsafeRelativePath,
    InvalidDriveLetter,
    UnsupportedPlatform,
    DependencyMissing(&'static str),
    DependencyInspectionFailed(&'static str),
    SignatureInvalid(&'static str),
    VersionUnsupported {
        found: String,
        minimum: &'static str,
    },
    ContainerMissing(PathBuf),
    DriveOccupied(char),
    DriveInspectionFailed(u32),
    VolumeNotMounted(char),
    DriveStillMounted(char),
    ProcessStartFailed {
        operation: &'static str,
        kind: io::ErrorKind,
    },
    ProcessFailed {
        operation: &'static str,
        exit_code: Option<i32>,
    },
    InvalidTransition {
        from: &'static str,
        event: &'static str,
    },
    PhysicalUsbDisabled,
    UnsafeDevice(&'static str),
    ProvisionTargetUnsafe(&'static str),
    ProvisionTargetExists(PathBuf),
    ProvisionSourceMissing(PathBuf),
    ProvisionFailed(&'static str),
    RestoreTargetUnsafe(&'static str),
    RestoreIdentityChanged,
    RestoreConfirmationInvalid,
    RestoreFailed(&'static str),
    PhysicalRestoreIdentityChanged,
    PhysicalRestoreConfirmationInvalid,
    PhysicalRestoreFailed(&'static str),
    IoFailed {
        operation: &'static str,
        kind: io::ErrorKind,
    },
}

impl LetheError {
    pub fn code(&self) -> ErrorCode {
        match self {
            Self::InvalidConfig { .. } | Self::UnknownConfigKey { .. } => ErrorCode::ConfigInvalid,
            Self::UnknownCommand => ErrorCode::CommandUnknown,
            Self::InvalidCommandArguments(_) => ErrorCode::CommandArgumentsInvalid,
            Self::UnsafeRelativePath => ErrorCode::PathUnsafe,
            Self::InvalidDriveLetter => ErrorCode::DriveLetterInvalid,
            Self::UnsupportedPlatform => ErrorCode::PlatformUnsupported,
            Self::DependencyMissing(_) => ErrorCode::DependencyMissing,
            Self::DependencyInspectionFailed(_) => ErrorCode::DependencyInspectionFailed,
            Self::SignatureInvalid(_) => ErrorCode::SignatureInvalid,
            Self::VersionUnsupported { .. } => ErrorCode::VersionUnsupported,
            Self::ContainerMissing(_) => ErrorCode::ContainerMissing,
            Self::DriveOccupied(_) => ErrorCode::DriveOccupied,
            Self::DriveInspectionFailed(_) => ErrorCode::DriveInspectionFailed,
            Self::VolumeNotMounted(_) => ErrorCode::VolumeNotMounted,
            Self::DriveStillMounted(_) => ErrorCode::DriveStillMounted,
            Self::ProcessStartFailed { .. } => ErrorCode::ProcessStartFailed,
            Self::ProcessFailed { .. } => ErrorCode::ProcessFailed,
            Self::InvalidTransition { .. } => ErrorCode::TransitionInvalid,
            Self::PhysicalUsbDisabled => ErrorCode::PhysicalUsbDisabled,
            Self::UnsafeDevice(_) => ErrorCode::DeviceUnsafe,
            Self::ProvisionTargetUnsafe(_) => ErrorCode::ProvisionTargetUnsafe,
            Self::ProvisionTargetExists(_) => ErrorCode::ProvisionTargetExists,
            Self::ProvisionSourceMissing(_) => ErrorCode::ProvisionSourceMissing,
            Self::ProvisionFailed(_) => ErrorCode::ProvisionFailed,
            Self::RestoreTargetUnsafe(_) => ErrorCode::RestoreTargetUnsafe,
            Self::RestoreIdentityChanged => ErrorCode::RestoreIdentityChanged,
            Self::RestoreConfirmationInvalid => ErrorCode::RestoreConfirmationInvalid,
            Self::RestoreFailed(_) => ErrorCode::RestoreFailed,
            Self::PhysicalRestoreIdentityChanged => ErrorCode::PhysicalRestoreIdentityChanged,
            Self::PhysicalRestoreConfirmationInvalid => {
                ErrorCode::PhysicalRestoreConfirmationInvalid
            }
            Self::PhysicalRestoreFailed(_) => ErrorCode::PhysicalRestoreFailed,
            Self::IoFailed { .. } => ErrorCode::IoFailed,
        }
    }

    pub fn io(operation: &'static str, error: &io::Error) -> Self {
        Self::IoFailed {
            operation,
            kind: error.kind(),
        }
    }
}

impl fmt::Display for LetheError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidConfig { line, reason } => {
                write!(
                    formatter,
                    "Configuracion invalida en la linea {line}: {reason}."
                )
            }
            Self::UnknownConfigKey { line, key } => {
                write!(formatter, "Opcion desconocida en la linea {line}: {key}.")
            }
            Self::UnknownCommand => write!(formatter, "El comando solicitado no existe."),
            Self::InvalidCommandArguments(reason) => {
                write!(
                    formatter,
                    "Los argumentos del comando no son validos: {reason}."
                )
            }
            Self::UnsafeRelativePath => write!(
                formatter,
                "La ruta del contenedor debe ser relativa y permanecer dentro de Lethe."
            ),
            Self::InvalidDriveLetter => {
                write!(formatter, "La letra de montaje configurada no es valida.")
            }
            Self::UnsupportedPlatform => {
                write!(formatter, "Esta version de Lethe solo funciona en Windows.")
            }
            Self::DependencyMissing(name) => write!(formatter, "No se encontro {name}."),
            Self::DependencyInspectionFailed(name) => {
                write!(formatter, "No se pudo validar la instalacion de {name}.")
            }
            Self::SignatureInvalid(name) => {
                write!(formatter, "La firma digital de {name} no es valida.")
            }
            Self::VersionUnsupported { found, minimum } => write!(
                formatter,
                "Version de VeraCrypt no compatible: {found}; se requiere {minimum} o posterior."
            ),
            Self::ContainerMissing(path) => write!(
                formatter,
                "No se encontro el contenedor cifrado: {}",
                path.display()
            ),
            Self::DriveOccupied(letter) => {
                write!(formatter, "La unidad {letter}: ya esta ocupada.")
            }
            Self::DriveInspectionFailed(code) => write!(
                formatter,
                "No se pudo consultar la letra de unidad (error de Windows {code})."
            ),
            Self::VolumeNotMounted(letter) => {
                write!(
                    formatter,
                    "No hay un volumen VeraCrypt montado en {letter}:."
                )
            }
            Self::DriveStillMounted(letter) => {
                write!(formatter, "La unidad {letter}: continua montada.")
            }
            Self::ProcessStartFailed { operation, kind } => write!(
                formatter,
                "No se pudo iniciar {operation} (error {kind:?})."
            ),
            Self::ProcessFailed {
                operation,
                exit_code,
            } => write!(
                formatter,
                "La operacion {operation} fallo (codigo {}).",
                exit_code
                    .map(|code| code.to_string())
                    .unwrap_or_else(|| "desconocido".to_owned())
            ),
            Self::InvalidTransition { from, event } => write!(
                formatter,
                "Operacion no permitida: {event} desde el estado {from}."
            ),
            Self::PhysicalUsbDisabled => write!(
                formatter,
                "Las operaciones destructivas sobre USB fisicas estan deshabilitadas."
            ),
            Self::UnsafeDevice(reason) => {
                write!(formatter, "El dispositivo fue rechazado: {reason}.")
            }
            Self::ProvisionTargetUnsafe(reason) => {
                write!(
                    formatter,
                    "El destino de preparacion fue rechazado: {reason}."
                )
            }
            Self::ProvisionTargetExists(path) => write!(
                formatter,
                "El destino ya existe y no sera sobrescrito: {}",
                path.display()
            ),
            Self::ProvisionSourceMissing(path) => write!(
                formatter,
                "Falta un componente necesario para preparar el paquete: {}",
                path.display()
            ),
            Self::ProvisionFailed(reason) => {
                write!(formatter, "La preparacion no pudo completarse: {reason}.")
            }
            Self::RestoreTargetUnsafe(reason) => {
                write!(
                    formatter,
                    "El destino de restauracion fue rechazado: {reason}."
                )
            }
            Self::RestoreIdentityChanged => write!(
                formatter,
                "La identidad del disco virtual cambio desde la planificacion."
            ),
            Self::RestoreConfirmationInvalid => write!(
                formatter,
                "La confirmacion de restauracion no coincide con la requerida."
            ),
            Self::RestoreFailed(reason) => {
                write!(formatter, "La restauracion no pudo completarse: {reason}.")
            }
            Self::PhysicalRestoreIdentityChanged => write!(
                formatter,
                "La identidad de la memoria USB cambio desde la planificacion."
            ),
            Self::PhysicalRestoreConfirmationInvalid => write!(
                formatter,
                "La confirmacion de restauracion fisica no coincide con la requerida."
            ),
            Self::PhysicalRestoreFailed(reason) => write!(
                formatter,
                "La restauracion de la memoria USB no pudo completarse: {reason}."
            ),
            Self::IoFailed { operation, kind } => {
                write!(formatter, "Fallo de E/S durante {operation} ({kind:?}).")
            }
        }
    }
}

impl std::error::Error for LetheError {}
