use crate::{
    domain::{BusType, DeviceIdentity, DriveLetter},
    error::LetheError,
    storage::PhysicalUsbTarget,
};
use std::{
    collections::HashMap,
    fs,
    path::PathBuf,
    process::Command,
    time::{SystemTime, UNIX_EPOCH},
};

const INSPECT_SCRIPT: &str = include_str!("../scripts/inspect-usb.ps1");
const RESTORE_SCRIPT: &str = include_str!("../scripts/restore-physical-usb.ps1");
pub const PHYSICAL_RESTORE_CONFIRMATION: &str = "ERASE-PHYSICAL-USB-JORGITO";

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UsbInspection {
    pub drive_letter: DriveLetter,
    pub filesystem: String,
    pub partition_number: u32,
    pub partition_count: u32,
    pub is_read_only: bool,
    pub is_offline: bool,
    pub identity: DeviceIdentity,
}

impl UsbInspection {
    pub fn inspect(drive_letter: DriveLetter, expected_label: &str) -> Result<Self, LetheError> {
        if !cfg!(windows) {
            return Err(LetheError::UnsupportedPlatform);
        }
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos();
        let helper = std::env::temp_dir().join(format!(
            "LetheUsbInspect-{}-{nonce}.ps1",
            std::process::id()
        ));
        fs::write(&helper, INSPECT_SCRIPT)
            .map_err(|error| LetheError::io("crear el inspector USB temporal", &error))?;
        let result = Command::new("powershell.exe")
            .args([
                "-NoProfile",
                "-NonInteractive",
                "-ExecutionPolicy",
                "Bypass",
                "-File",
            ])
            .arg(&helper)
            .arg("-DriveLetter")
            .arg(drive_letter.argument())
            .output();
        let _ = fs::remove_file(&helper);
        let output = result.map_err(|error| LetheError::ProcessStartFailed {
            operation: "el inspector USB",
            kind: error.kind(),
        })?;
        if !output.status.success() {
            return Err(LetheError::ProcessFailed {
                operation: "la inspeccion USB",
                exit_code: output.status.code(),
            });
        }
        let values = parse_output(&String::from_utf8_lossy(&output.stdout))?;
        Self::from_values(drive_letter, expected_label, &values)
    }

    fn from_values(
        drive_letter: DriveLetter,
        expected_label: &str,
        values: &HashMap<String, String>,
    ) -> Result<Self, LetheError> {
        let label = required(values, "LABEL")?.to_owned();
        if label != expected_label {
            return Err(LetheError::UnsafeDevice(
                "la etiqueta no coincide con la identidad esperada",
            ));
        }
        let bus_type = match required(values, "BUS_TYPE")? {
            "USB" => BusType::Usb,
            "Virtual" | "File Backed Virtual" => BusType::Virtual,
            _ => BusType::Other,
        };
        let partition_count = number(values, "PARTITION_COUNT")?;
        if partition_count != 1 {
            return Err(LetheError::UnsafeDevice(
                "la unidad no tiene exactamente una particion",
            ));
        }
        let identity = DeviceIdentity {
            disk_number: number(values, "DISK_NUMBER")?,
            label,
            size_bytes: number(values, "SIZE_BYTES")?,
            bus_type,
            is_boot: boolean(values, "IS_BOOT")?,
            is_system: boolean(values, "IS_SYSTEM")?,
            serial: Some(required(values, "SERIAL")?.trim().to_owned()),
        };
        let is_read_only = boolean(values, "IS_READ_ONLY")?;
        let is_offline = boolean(values, "IS_OFFLINE")?;
        if is_read_only {
            return Err(LetheError::UnsafeDevice(
                "el disco USB esta protegido contra escritura",
            ));
        }
        if is_offline {
            return Err(LetheError::UnsafeDevice("el disco USB esta fuera de linea"));
        }
        let target = PhysicalUsbTarget::new(identity.clone())?;
        debug_assert_eq!(target.identity(), &identity);
        Ok(Self {
            drive_letter,
            filesystem: required(values, "FILESYSTEM")?.to_owned(),
            partition_number: number(values, "PARTITION_NUMBER")?,
            partition_count,
            is_read_only,
            is_offline,
            identity,
        })
    }

    pub fn render(&self) -> String {
        format!(
            "USB_STATE=READY_FOR_FINAL_TEST\nDRIVE_LETTER={}\nLABEL={}\nFILESYSTEM={}\nDISK_NUMBER={}\nPARTITION_NUMBER={}\nPARTITION_COUNT={}\nSIZE_BYTES={}\nBUS_TYPE=USB\nIS_BOOT=0\nIS_SYSTEM=0\nIS_READ_ONLY=0\nIS_OFFLINE=0\nSERIAL={}\nPHYSICAL_EXECUTION=REQUIRES_EXPLICIT_RESTORE_PLAN\nCHANGES=NONE",
            self.drive_letter,
            self.identity.label,
            self.filesystem,
            self.identity.disk_number,
            self.partition_number,
            self.partition_count,
            self.identity.size_bytes,
            self.identity.serial.as_deref().unwrap_or_default()
        )
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PhysicalRestorePlan {
    pub inspection: UsbInspection,
    pub result_filesystem: &'static str,
    pub result_label: &'static str,
}

impl PhysicalRestorePlan {
    pub fn create(drive_letter: DriveLetter, expected_label: &str) -> Result<Self, LetheError> {
        Ok(Self {
            inspection: UsbInspection::inspect(drive_letter, expected_label)?,
            result_filesystem: "exFAT",
            result_label: "JORGITO",
        })
    }

    pub fn render(&self) -> String {
        format!(
            "RESTORE_STATE=PLANNED\nTARGET_KIND=PHYSICAL_USB\nDRIVE_LETTER={}\nCURRENT_LABEL={}\nCURRENT_FILESYSTEM={}\nDISK_NUMBER={}\nPARTITION_NUMBER={}\nPARTITION_COUNT={}\nSIZE_BYTES={}\nBUS_TYPE=USB\nIS_BOOT=0\nIS_SYSTEM=0\nIS_READ_ONLY=0\nIS_OFFLINE=0\nSERIAL={}\nRESULT_FILESYSTEM={}\nRESULT_LABEL={}\nCONFIRMATION={}\nDATA_LOSS=ALL_CURRENT_CONTENT\nCHANGES=NONE",
            self.inspection.drive_letter,
            self.inspection.identity.label,
            self.inspection.filesystem,
            self.inspection.identity.disk_number,
            self.inspection.partition_number,
            self.inspection.partition_count,
            self.inspection.identity.size_bytes,
            self.inspection
                .identity
                .serial
                .as_deref()
                .unwrap_or_default(),
            self.result_filesystem,
            self.result_label,
            PHYSICAL_RESTORE_CONFIRMATION
        )
    }

    pub fn execute(
        &self,
        expected: &UsbInspection,
        confirmation: &str,
    ) -> Result<String, LetheError> {
        if confirmation != PHYSICAL_RESTORE_CONFIRMATION {
            return Err(LetheError::PhysicalRestoreConfirmationInvalid);
        }
        if &self.inspection != expected {
            return Err(LetheError::PhysicalRestoreIdentityChanged);
        }
        run_physical_restore_helper(self)
    }
}

fn run_physical_restore_helper(plan: &PhysicalRestorePlan) -> Result<String, LetheError> {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    let helper_root: PathBuf = std::env::temp_dir().join(format!(
        "LethePhysicalRestore-{}-{nonce}",
        std::process::id()
    ));
    fs::create_dir(&helper_root)
        .map_err(|error| LetheError::io("crear el restaurador fisico temporal", &error))?;
    let helper_path = helper_root.join("restore-physical-usb.ps1");
    let result = (|| {
        fs::write(&helper_path, RESTORE_SCRIPT)
            .map_err(|error| LetheError::io("escribir el restaurador fisico", &error))?;
        let identity = &plan.inspection.identity;
        let output = Command::new("powershell.exe")
            .args([
                "-NoProfile",
                "-NonInteractive",
                "-ExecutionPolicy",
                "Bypass",
                "-File",
            ])
            .arg(&helper_path)
            .arg("-DriveLetter")
            .arg(plan.inspection.drive_letter.argument())
            .arg("-ExpectedLabel")
            .arg(&identity.label)
            .arg("-ExpectedFileSystem")
            .arg(&plan.inspection.filesystem)
            .arg("-ExpectedDiskNumber")
            .arg(identity.disk_number.to_string())
            .arg("-ExpectedPartitionNumber")
            .arg(plan.inspection.partition_number.to_string())
            .arg("-ExpectedSize")
            .arg(identity.size_bytes.to_string())
            .arg("-ExpectedSerial")
            .arg(identity.serial.as_deref().unwrap_or_default())
            .arg("-ResultLabel")
            .arg(plan.result_label)
            .output()
            .map_err(|error| LetheError::ProcessStartFailed {
                operation: "el restaurador fisico",
                kind: error.kind(),
            })?;
        if !output.status.success() {
            return Err(LetheError::ProcessFailed {
                operation: "la restauracion fisica",
                exit_code: output.status.code(),
            });
        }
        let stdout = String::from_utf8_lossy(&output.stdout);
        if !stdout
            .lines()
            .any(|line| line.trim() == "PHYSICAL_RESTORE_OK=1")
        {
            return Err(LetheError::PhysicalRestoreFailed(
                "el ayudante no confirmo el resultado esperado",
            ));
        }
        Ok(stdout.trim().to_owned())
    })();
    let cleanup = fs::remove_dir_all(&helper_root)
        .map_err(|error| LetheError::io("eliminar el restaurador fisico temporal", &error));
    match (result, cleanup) {
        (Err(error), _) => Err(error),
        (Ok(_), Err(error)) => Err(error),
        (Ok(output), Ok(())) => Ok(output),
    }
}

fn parse_output(output: &str) -> Result<HashMap<String, String>, LetheError> {
    let values: HashMap<String, String> = output
        .lines()
        .filter_map(|line| line.trim().split_once('='))
        .map(|(key, value)| (key.to_owned(), value.to_owned()))
        .collect();
    if values.is_empty() {
        return Err(LetheError::UnsafeDevice(
            "Windows no devolvio una identidad de dispositivo",
        ));
    }
    Ok(values)
}

fn required<'a>(
    values: &'a HashMap<String, String>,
    key: &'static str,
) -> Result<&'a str, LetheError> {
    values
        .get(key)
        .map(String::as_str)
        .ok_or(LetheError::UnsafeDevice("la identidad USB esta incompleta"))
}

fn number<T: std::str::FromStr>(
    values: &HashMap<String, String>,
    key: &'static str,
) -> Result<T, LetheError> {
    required(values, key)?
        .parse()
        .map_err(|_| LetheError::UnsafeDevice("la identidad USB contiene un numero invalido"))
}

fn boolean(values: &HashMap<String, String>, key: &'static str) -> Result<bool, LetheError> {
    match required(values, key)? {
        "0" => Ok(false),
        "1" => Ok(true),
        _ => Err(LetheError::UnsafeDevice(
            "la identidad USB contiene una bandera invalida",
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn safe_values() -> HashMap<String, String> {
        parse_output(
            "LABEL=JORGITO\nFILESYSTEM=NTFS\nDISK_NUMBER=2\nPARTITION_NUMBER=1\nPARTITION_COUNT=1\nSIZE_BYTES=8011120640\nBUS_TYPE=USB\nIS_BOOT=0\nIS_SYSTEM=0\nIS_READ_ONLY=0\nIS_OFFLINE=0\nSERIAL=057907B76060\n",
        )
        .unwrap()
    }

    #[test]
    fn expected_identity_is_accepted_without_enabling_execution() {
        let inspection =
            UsbInspection::from_values(DriveLetter::new('F').unwrap(), "JORGITO", &safe_values())
                .unwrap();
        assert!(
            inspection
                .render()
                .contains("PHYSICAL_EXECUTION=REQUIRES_EXPLICIT_RESTORE_PLAN")
        );
        assert!(inspection.render().contains("CHANGES=NONE"));
    }

    #[test]
    fn wrong_label_is_rejected() {
        assert!(matches!(
            UsbInspection::from_values(DriveLetter::new('F').unwrap(), "OTHER", &safe_values()),
            Err(LetheError::UnsafeDevice(_))
        ));
    }

    #[test]
    fn system_disk_is_rejected() {
        let mut values = safe_values();
        values.insert("IS_SYSTEM".to_owned(), "1".to_owned());
        assert!(
            UsbInspection::from_values(DriveLetter::new('F').unwrap(), "JORGITO", &values).is_err()
        );
    }

    #[test]
    fn multiple_partitions_are_rejected() {
        let mut values = safe_values();
        values.insert("PARTITION_COUNT".to_owned(), "2".to_owned());
        assert!(
            UsbInspection::from_values(DriveLetter::new('F').unwrap(), "JORGITO", &values).is_err()
        );
    }

    #[test]
    fn physical_plan_declares_total_data_loss_but_makes_no_changes() {
        let inspection =
            UsbInspection::from_values(DriveLetter::new('F').unwrap(), "JORGITO", &safe_values())
                .unwrap();
        let plan = PhysicalRestorePlan {
            inspection,
            result_filesystem: "exFAT",
            result_label: "JORGITO",
        };
        let rendered = plan.render();
        assert!(rendered.contains("DATA_LOSS=ALL_CURRENT_CONTENT"));
        assert!(rendered.contains("CHANGES=NONE"));
    }

    #[test]
    fn wrong_physical_confirmation_is_rejected_before_helper() {
        let inspection =
            UsbInspection::from_values(DriveLetter::new('F').unwrap(), "JORGITO", &safe_values())
                .unwrap();
        let plan = PhysicalRestorePlan {
            inspection: inspection.clone(),
            result_filesystem: "exFAT",
            result_label: "JORGITO",
        };
        assert_eq!(
            plan.execute(&inspection, "NO").unwrap_err(),
            LetheError::PhysicalRestoreConfirmationInvalid
        );
    }

    #[test]
    fn changed_physical_identity_is_rejected_before_helper() {
        let inspection =
            UsbInspection::from_values(DriveLetter::new('F').unwrap(), "JORGITO", &safe_values())
                .unwrap();
        let mut changed = inspection.clone();
        changed.identity.serial = Some("OTHER".to_owned());
        let plan = PhysicalRestorePlan {
            inspection,
            result_filesystem: "exFAT",
            result_label: "JORGITO",
        };
        assert_eq!(
            plan.execute(&changed, PHYSICAL_RESTORE_CONFIRMATION)
                .unwrap_err(),
            LetheError::PhysicalRestoreIdentityChanged
        );
    }

    #[test]
    fn physical_helper_resolves_from_letter_before_first_destructive_command() {
        let resolve = RESTORE_SCRIPT
            .find("$target = Resolve-ExactTarget")
            .unwrap();
        let clear = RESTORE_SCRIPT.find("Clear-Disk").unwrap();
        assert!(RESTORE_SCRIPT.contains("$partition | Get-Disk"));
        assert!(resolve < clear);
    }
}
