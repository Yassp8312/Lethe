use lethe::{
    audit::{self, AuditEvent},
    config::{self, LetheConfig},
    error::LetheError,
    mount::{DriveStatus, inspect_drive},
    provision::{ProvisionRequest, ProvisionSources, run_creation_wizard},
    restore::{RESTORE_CONFIRMATION, RestorePlan},
    usb::{PHYSICAL_RESTORE_CONFIRMATION, PhysicalRestorePlan, UsbInspection},
    veracrypt::{self, MountMode, VeraCryptInstallation},
};
use std::{env, ffi::OsString, path::PathBuf, process::ExitCode};

#[derive(Debug)]
struct Runtime {
    config: LetheConfig,
    base_dir: PathBuf,
    veracrypt: Option<VeraCryptInstallation>,
}

impl Runtime {
    fn discover() -> Result<Self, LetheError> {
        let executable_dir = env::var_os("LETHE_HOME")
            .map(PathBuf::from)
            .unwrap_or_else(|| {
                env::current_exe()
                    .ok()
                    .and_then(|path| path.parent().map(|parent| parent.to_path_buf()))
                    .unwrap_or_else(|| PathBuf::from("."))
            });

        let configured = if let Some(config) = config::load_from(&executable_dir)? {
            Some((config, executable_dir.clone()))
        } else {
            discover_drive_config()?
        };
        let (config, config_dir) =
            configured.unwrap_or_else(|| (LetheConfig::default(), executable_dir.clone()));

        let veracrypt = veracrypt::locate(&config_dir)
            .map(VeraCryptInstallation::inspect)
            .transpose()?;

        Ok(Self {
            config,
            base_dir: config_dir.clone(),
            veracrypt,
        })
    }

    fn container_path(&self) -> PathBuf {
        self.config.container.resolve_under(&self.base_dir)
    }

    fn validate(&self) -> Result<(), LetheError> {
        if !cfg!(target_os = "windows") {
            return Err(LetheError::UnsupportedPlatform);
        }
        if self.veracrypt.is_none() {
            return Err(LetheError::DependencyMissing("VeraCrypt"));
        }
        let container = self.container_path();
        if !container.is_file() {
            return Err(LetheError::ContainerMissing(container));
        }
        Ok(())
    }

    fn mount(&self, mode: MountMode) -> Result<String, LetheError> {
        self.validate()?;
        let veracrypt = self.veracrypt.as_ref().expect("runtime validated");
        let container = self.container_path();
        let letter = self.config.mount_letter;
        match inspect_drive(letter)? {
            DriveStatus::Available => {}
            DriveStatus::OccupiedByVeraCrypt { .. } | DriveStatus::OccupiedByOther { .. } => {
                return Err(LetheError::DriveOccupied(letter.as_char()));
            }
        }

        let spec = veracrypt.mount_spec(&container, letter, mode);
        debug_assert!(!spec.contains_forbidden_secret_switch());
        spec.spawn("VeraCrypt")?;

        Ok(match mode {
            MountMode::ReadOnly => "Dialogo abierto en modo de solo lectura.".to_owned(),
            MountMode::ReadWrite => "Dialogo abierto con escritura habilitada.".to_owned(),
        })
    }

    fn unmount(&self, force: bool) -> Result<String, LetheError> {
        let veracrypt = self
            .veracrypt
            .as_ref()
            .ok_or(LetheError::DependencyMissing("VeraCrypt"))?;
        let letter = self.config.mount_letter;
        match inspect_drive(letter)? {
            DriveStatus::Available => {
                return Err(LetheError::VolumeNotMounted(letter.as_char()));
            }
            DriveStatus::OccupiedByOther { .. } => {
                return Err(LetheError::DriveOccupied(letter.as_char()));
            }
            DriveStatus::OccupiedByVeraCrypt { .. } => {}
        }
        veracrypt
            .unmount_spec(letter, force)
            .run("el desmontaje de VeraCrypt")?;
        veracrypt
            .wipe_cache_spec()
            .run("la limpieza de cache de VeraCrypt")?;
        if !matches!(inspect_drive(letter)?, DriveStatus::Available) {
            return Err(LetheError::DriveStillMounted(letter.as_char()));
        }
        Ok(format!(
            "Unidad {letter}: desmontada y cache de VeraCrypt limpiada."
        ))
    }

    fn report(&self) -> String {
        format!(
            "Motor: {}\nContenedor: {}\nUnidad virtual: {}:",
            self.veracrypt
                .as_ref()
                .map(|installation| {
                    format!(
                        "{} (version {}, firma valida)",
                        installation.executable().display(),
                        installation.version()
                    )
                })
                .unwrap_or_else(|| "NO ENCONTRADO".to_owned()),
            self.container_path().display(),
            self.config.mount_letter
        )
    }

    fn drive_status(&self) -> Result<String, LetheError> {
        let letter = self.config.mount_letter;
        Ok(match inspect_drive(letter)? {
            DriveStatus::Available => format!("Unidad {letter}: disponible"),
            DriveStatus::OccupiedByVeraCrypt { target } => {
                format!("Unidad {letter}: VeraCrypt ({target})")
            }
            DriveStatus::OccupiedByOther { target } => {
                format!("Unidad {letter}: ocupada por otro dispositivo ({target})")
            }
        })
    }

    fn machine_status(&self) -> Result<String, LetheError> {
        let container = self.container_path();
        let letter = self.config.mount_letter;
        let container_exists = container.is_file();
        let engine_version = self
            .veracrypt
            .as_ref()
            .map(|installation| installation.version().to_string())
            .unwrap_or_default();
        let (state, target) = match inspect_drive(letter)? {
            DriveStatus::Available => {
                if self.veracrypt.is_some() && container_exists {
                    ("AVAILABLE", String::new())
                } else {
                    ("NOT_READY", String::new())
                }
            }
            DriveStatus::OccupiedByVeraCrypt { target } => ("MOUNTED", target),
            DriveStatus::OccupiedByOther { target } => ("OCCUPIED", target),
        };

        Ok(format!(
            "STATE={state}\nLETTER={letter}\nENGINE_VERSION={engine_version}\nCONTAINER_EXISTS={}\nCONTAINER={}\nTARGET={target}",
            u8::from(container_exists),
            container.display()
        ))
    }
}

fn discover_drive_config() -> Result<Option<(LetheConfig, PathBuf)>, LetheError> {
    for letter in b'D'..=b'Z' {
        let directory = PathBuf::from(format!("{}:\\Lethe", letter as char));
        if let Some(config) = config::load_from(&directory)? {
            return Ok(Some((config, directory)));
        }
    }
    Ok(None)
}

fn print_usage() {
    println!("Lethe 0.1");
    println!("Uso: lethe <doctor|status|drive-status|open-ro|open-rw|lock|force-lock>");
    println!("     lethe provision <plan|execute|wizard> --target <carpeta-absoluta>");
    println!("     lethe restore plan --virtual-image <VHD|VHDX>");
    println!(
        "     lethe restore execute --virtual-image <VHD|VHDX> --expected-size <n> --expected-created <n> --expected-write <n> --confirm <frase>"
    );
    println!("     lethe usb inspect --letter <A-Z> --expected-label <etiqueta>");
    println!("     lethe usb restore-plan --letter <A-Z> --expected-label <etiqueta>");
}

fn execute() -> Result<String, LetheError> {
    let arguments: Vec<OsString> = env::args_os().skip(1).collect();
    let runtime = Runtime::discover()?;
    match arguments.first().and_then(|argument| argument.to_str()) {
        Some("doctor") => {
            println!("{}", runtime.report());
            runtime.validate()?;
            Ok("Diagnostico correcto.".to_owned())
        }
        Some("drive-status") => runtime.drive_status(),
        Some("status") => runtime.machine_status(),
        Some("open-ro") => runtime.mount(MountMode::ReadOnly),
        Some("open-rw") => runtime.mount(MountMode::ReadWrite),
        Some("lock") => runtime.unmount(false),
        Some("force-lock") => runtime.unmount(true),
        Some("provision") => execute_provision(&runtime, &arguments[1..]),
        Some("restore") => execute_restore(&arguments[1..]),
        Some("usb") => execute_usb(&arguments[1..]),
        Some(_) => {
            print_usage();
            Err(LetheError::UnknownCommand)
        }
        None => {
            print_usage();
            Ok(String::new())
        }
    }
}

fn execute_usb(arguments: &[OsString]) -> Result<String, LetheError> {
    let action = arguments
        .first()
        .and_then(|argument| argument.to_str())
        .ok_or(LetheError::InvalidCommandArguments(
            "use usb inspect o usb restore-plan",
        ))?;
    match action {
        "inspect" | "restore-plan" => {
            if arguments.len() != 5
                || arguments[1] != "--letter"
                || arguments[3] != "--expected-label"
            {
                return Err(LetheError::InvalidCommandArguments(
                    "use usb <inspect|restore-plan> --letter <A-Z> --expected-label <etiqueta>",
                ));
            }
            let letter = parse_usb_letter(&arguments[2])?;
            let expected_label = parse_usb_text(&arguments[4], "la etiqueta esperada")?;
            if action == "inspect" {
                UsbInspection::inspect(letter, expected_label).map(|inspection| inspection.render())
            } else {
                PhysicalRestorePlan::create(letter, expected_label).map(|plan| plan.render())
            }
        }
        "restore-execute" => execute_physical_restore(arguments),
        _ => Err(LetheError::InvalidCommandArguments(
            "la accion USB debe ser inspect, restore-plan o restore-execute",
        )),
    }
}

fn execute_physical_restore(arguments: &[OsString]) -> Result<String, LetheError> {
    if arguments.len() != 17
        || arguments[1] != "--letter"
        || arguments[3] != "--expected-label"
        || arguments[5] != "--expected-disk"
        || arguments[7] != "--expected-partition"
        || arguments[9] != "--expected-size"
        || arguments[11] != "--expected-filesystem"
        || arguments[13] != "--expected-serial"
        || arguments[15] != "--confirm"
    {
        return Err(LetheError::InvalidCommandArguments(
            "la restauracion fisica requiere la identidad completa del plan",
        ));
    }
    let letter = parse_usb_letter(&arguments[2])?;
    let expected_label = parse_usb_text(&arguments[4], "la etiqueta esperada")?;
    let expected_disk = parse_restore_number(&arguments[6])?;
    let expected_partition = parse_restore_number(&arguments[8])?;
    let expected_size = parse_restore_number(&arguments[10])?;
    let expected_filesystem = parse_usb_text(&arguments[12], "el sistema de archivos")?;
    let expected_serial = parse_usb_text(&arguments[14], "el numero de serie")?;
    let confirmation = parse_usb_text(&arguments[16], "la confirmacion fisica")?;
    if confirmation != PHYSICAL_RESTORE_CONFIRMATION {
        return Err(LetheError::PhysicalRestoreConfirmationInvalid);
    }

    let current = UsbInspection::inspect(letter, expected_label)?;
    if u64::from(current.identity.disk_number) != expected_disk
        || u64::from(current.partition_number) != expected_partition
        || current.identity.size_bytes != expected_size
        || current.filesystem != expected_filesystem
        || current.identity.serial.as_deref() != Some(expected_serial)
    {
        return Err(LetheError::PhysicalRestoreIdentityChanged);
    }
    let plan = PhysicalRestorePlan {
        inspection: current.clone(),
        result_filesystem: "exFAT",
        result_label: "JORGITO",
    };
    plan.execute(&current, confirmation)
}

fn parse_usb_letter(value: &OsString) -> Result<lethe::domain::DriveLetter, LetheError> {
    value
        .to_str()
        .ok_or(LetheError::InvalidDriveLetter)
        .and_then(lethe::domain::DriveLetter::try_from)
}

fn parse_usb_text<'a>(value: &'a OsString, field: &'static str) -> Result<&'a str, LetheError> {
    value
        .to_str()
        .filter(|text| !text.trim().is_empty())
        .ok_or(LetheError::InvalidCommandArguments(field))
}

fn execute_restore(arguments: &[OsString]) -> Result<String, LetheError> {
    let action = arguments
        .first()
        .and_then(|argument| argument.to_str())
        .ok_or(LetheError::InvalidCommandArguments(
            "use restore plan o restore execute",
        ))?;
    let workspace = env::current_dir()
        .map_err(|error| LetheError::io("consultar el area de pruebas", &error))?;

    match action {
        "plan" => {
            if arguments.len() != 3 || arguments[1] != "--virtual-image" {
                return Err(LetheError::InvalidCommandArguments(
                    "use restore plan --virtual-image <VHD|VHDX>",
                ));
            }
            let image = PathBuf::from(&arguments[2]);
            Ok(RestorePlan::create(&image, &workspace)?.render())
        }
        "execute" => {
            if arguments.len() != 11
                || arguments[1] != "--virtual-image"
                || arguments[3] != "--expected-size"
                || arguments[5] != "--expected-created"
                || arguments[7] != "--expected-write"
                || arguments[9] != "--confirm"
            {
                return Err(LetheError::InvalidCommandArguments(
                    "la ejecucion requiere imagen, identidad esperada y confirmacion",
                ));
            }
            let image = PathBuf::from(&arguments[2]);
            let expected_size = parse_restore_number(&arguments[4])?;
            let expected_created = parse_restore_number(&arguments[6])?;
            let expected_write = parse_restore_number(&arguments[8])?;
            let confirmation = arguments[10]
                .to_str()
                .ok_or(LetheError::RestoreConfirmationInvalid)?;
            if confirmation != RESTORE_CONFIRMATION {
                return Err(LetheError::RestoreConfirmationInvalid);
            }
            let plan = RestorePlan::create(&image, &workspace)?;
            if plan.identity.size_bytes != expected_size
                || plan.identity.creation_time != expected_created
                || plan.identity.last_write_time != expected_write
            {
                return Err(LetheError::RestoreIdentityChanged);
            }
            plan.execute(&workspace, confirmation)
        }
        _ => Err(LetheError::InvalidCommandArguments(
            "la accion debe ser plan o execute",
        )),
    }
}

fn parse_restore_number(value: &OsString) -> Result<u64, LetheError> {
    value.to_str().and_then(|number| number.parse().ok()).ok_or(
        LetheError::InvalidCommandArguments("la identidad numerica del plan no es valida"),
    )
}

fn execute_provision(runtime: &Runtime, arguments: &[OsString]) -> Result<String, LetheError> {
    if arguments.len() != 3 || arguments[1] != "--target" {
        print_usage();
        return Err(LetheError::InvalidCommandArguments(
            "use provision <plan|execute|wizard> --target <carpeta-absoluta>",
        ));
    }
    let action = arguments[0]
        .to_str()
        .ok_or(LetheError::InvalidCommandArguments(
            "la accion no es texto valido",
        ))?;
    let target = PathBuf::from(&arguments[2]);
    let workspace = env::current_dir()
        .map_err(|error| LetheError::io("consultar el area de trabajo", &error))?;

    if action == "wizard" {
        return run_creation_wizard(&target, &workspace);
    }

    let veracrypt = runtime
        .veracrypt
        .as_ref()
        .ok_or(LetheError::DependencyMissing("VeraCrypt"))?;
    let veracrypt_directory = veracrypt
        .executable()
        .parent()
        .ok_or(LetheError::ProvisionSourceMissing(
            veracrypt.executable().to_path_buf(),
        ))?
        .to_path_buf();
    let request = ProvisionRequest {
        target,
        sandbox_root: workspace.clone(),
        sources: ProvisionSources {
            lethe_executable: env::current_exe()
                .map_err(|error| LetheError::io("localizar Lethe.exe", &error))?,
            panel_script: workspace.join("panel").join("Lethe.ps1"),
            veracrypt_directory,
            documentation_directory: workspace.join("docs"),
        },
        mount_letter: runtime.config.mount_letter,
        container_name: "vault.hc",
    };
    match action {
        "plan" => Ok(request.plan()?.render()),
        "execute" => Ok(request.execute()?.render()),
        _ => Err(LetheError::InvalidCommandArguments(
            "la accion debe ser plan, execute o wizard",
        )),
    }
}

fn main() -> ExitCode {
    audit::append_default(AuditEvent::Started);
    let result = execute();
    match result {
        Ok(message) => {
            if !message.is_empty() {
                println!("{message}");
            }
            audit::append_default(AuditEvent::Stopped);
            ExitCode::SUCCESS
        }
        Err(error) => {
            audit::append_default(AuditEvent::OperationRejected(error.code()));
            eprintln!("ERROR [{:?}]: {error}", error.code());
            ExitCode::from(2)
        }
    }
}
