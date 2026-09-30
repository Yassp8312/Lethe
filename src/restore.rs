use crate::{
    error::LetheError,
    storage::{StorageTarget, VirtualDiskTarget},
};
use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
};

#[cfg(windows)]
use std::os::windows::fs::MetadataExt;

pub const RESTORE_CONFIRMATION: &str = "RESTORE-VIRTUAL-DISK";
pub const RESTORED_LABEL: &str = "JORGITO";
const MINIMUM_IMAGE_SIZE: u64 = 32 * 1024 * 1024;
const MAXIMUM_IMAGE_SIZE: u64 = 1024 * 1024 * 1024 * 1024;
const HELPER_SCRIPT: &str = include_str!("../scripts/restore-virtual-disk.ps1");

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct VirtualImageIdentity {
    pub path: PathBuf,
    pub size_bytes: u64,
    pub creation_time: u64,
    pub last_write_time: u64,
}

impl VirtualImageIdentity {
    pub fn inspect(image: &Path, sandbox_root: &Path) -> Result<Self, LetheError> {
        if !cfg!(windows) {
            return Err(LetheError::UnsupportedPlatform);
        }
        if !image.is_absolute() {
            return Err(LetheError::RestoreTargetUnsafe(
                "la imagen debe tener una ruta absoluta",
            ));
        }
        let extension = image
            .extension()
            .and_then(|value| value.to_str())
            .unwrap_or_default();
        if !extension.eq_ignore_ascii_case("vhd") && !extension.eq_ignore_ascii_case("vhdx") {
            return Err(LetheError::RestoreTargetUnsafe(
                "solo se admiten imagenes VHD o VHDX",
            ));
        }
        let metadata = fs::symlink_metadata(image)
            .map_err(|error| LetheError::io("inspeccionar la imagen virtual", &error))?;
        if metadata.file_type().is_symlink() || !metadata.is_file() {
            return Err(LetheError::RestoreTargetUnsafe(
                "la imagen debe ser un archivo regular, no un enlace",
            ));
        }
        if !(MINIMUM_IMAGE_SIZE..=MAXIMUM_IMAGE_SIZE).contains(&metadata.len()) {
            return Err(LetheError::RestoreTargetUnsafe(
                "el tamano de la imagen esta fuera del rango permitido",
            ));
        }
        let canonical_path = image
            .canonicalize()
            .map_err(|error| LetheError::io("resolver la imagen virtual", &error))?;
        let canonical_sandbox = sandbox_root
            .canonicalize()
            .map_err(|error| LetheError::io("resolver el area de pruebas", &error))?;
        if canonical_path == canonical_sandbox || !canonical_path.starts_with(&canonical_sandbox) {
            return Err(LetheError::RestoreTargetUnsafe(
                "la imagen debe permanecer dentro del area local de pruebas",
            ));
        }

        StorageTarget::Virtual(VirtualDiskTarget::new(&canonical_path))
            .ensure_destructive_operations_allowed()?;

        #[cfg(windows)]
        {
            Ok(Self {
                path: canonical_path,
                size_bytes: metadata.file_size(),
                creation_time: metadata.creation_time(),
                last_write_time: metadata.last_write_time(),
            })
        }
        #[cfg(not(windows))]
        unreachable!()
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RestorePlan {
    pub identity: VirtualImageIdentity,
    pub filesystem: &'static str,
    pub label: &'static str,
}

impl RestorePlan {
    pub fn create(image: &Path, sandbox_root: &Path) -> Result<Self, LetheError> {
        Ok(Self {
            identity: VirtualImageIdentity::inspect(image, sandbox_root)?,
            filesystem: "exFAT",
            label: RESTORED_LABEL,
        })
    }

    pub fn revalidate(&self, sandbox_root: &Path) -> Result<(), LetheError> {
        let current = VirtualImageIdentity::inspect(&self.identity.path, sandbox_root)?;
        if current != self.identity {
            return Err(LetheError::RestoreIdentityChanged);
        }
        Ok(())
    }

    pub fn render(&self) -> String {
        format!(
            "RESTORE_STATE=PLANNED\nTARGET_KIND=VIRTUAL_DISK\nIMAGE={}\nSIZE_BYTES={}\nCREATION_TIME={}\nLAST_WRITE_TIME={}\nFILESYSTEM={}\nLABEL={}\nCONFIRMATION={}\nCHANGES=NONE",
            self.identity.path.display(),
            self.identity.size_bytes,
            self.identity.creation_time,
            self.identity.last_write_time,
            self.filesystem,
            self.label,
            RESTORE_CONFIRMATION
        )
    }

    pub fn execute(&self, sandbox_root: &Path, confirmation: &str) -> Result<String, LetheError> {
        if confirmation != RESTORE_CONFIRMATION {
            return Err(LetheError::RestoreConfirmationInvalid);
        }
        self.revalidate(sandbox_root)?;
        run_helper(self)
    }
}

fn run_helper(plan: &RestorePlan) -> Result<String, LetheError> {
    let helper_root = std::env::temp_dir().join(format!(
        "LetheRestore-{}-{}",
        std::process::id(),
        plan.identity.creation_time
    ));
    if helper_root.exists() {
        return Err(LetheError::RestoreFailed(
            "la carpeta temporal del ayudante ya existe",
        ));
    }
    fs::create_dir(&helper_root)
        .map_err(|error| LetheError::io("crear el ayudante temporal", &error))?;
    let helper_path = helper_root.join("restore-virtual-disk.ps1");
    let result = (|| {
        fs::write(&helper_path, HELPER_SCRIPT)
            .map_err(|error| LetheError::io("escribir el ayudante temporal", &error))?;
        let output = Command::new("powershell.exe")
            .args([
                "-NoProfile",
                "-NonInteractive",
                "-ExecutionPolicy",
                "Bypass",
                "-File",
            ])
            .arg(&helper_path)
            .arg("-ImagePath")
            .arg(&plan.identity.path)
            .arg("-ExpectedLength")
            .arg(plan.identity.size_bytes.to_string())
            .arg("-ExpectedWriteTime")
            .arg(plan.identity.last_write_time.to_string())
            .arg("-Label")
            .arg(plan.label)
            .output()
            .map_err(|error| LetheError::ProcessStartFailed {
                operation: "el ayudante de restauracion virtual",
                kind: error.kind(),
            })?;
        if !output.status.success() {
            return Err(LetheError::ProcessFailed {
                operation: "la restauracion del disco virtual",
                exit_code: output.status.code(),
            });
        }
        let stdout = String::from_utf8_lossy(&output.stdout);
        if !stdout.lines().any(|line| line.trim() == "RESTORE_OK=1") {
            return Err(LetheError::RestoreFailed(
                "el ayudante no confirmo el resultado esperado",
            ));
        }
        Ok(stdout.trim().to_owned())
    })();
    let cleanup = fs::remove_dir_all(&helper_root)
        .map_err(|error| LetheError::io("eliminar el ayudante temporal", &error));
    match (result, cleanup) {
        (Err(error), _) => Err(error),
        (Ok(_), Err(error)) => Err(error),
        (Ok(output), Ok(())) => Ok(output),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn fixture() -> (PathBuf, PathBuf) {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let sandbox = std::env::temp_dir().join(format!("lethe-restore-{unique}"));
        fs::create_dir(&sandbox).unwrap();
        let image = sandbox.join("test.vhdx");
        let file = fs::File::create(&image).unwrap();
        file.set_len(MINIMUM_IMAGE_SIZE).unwrap();
        (sandbox, image)
    }

    #[test]
    fn plan_has_no_side_effects() {
        let (sandbox, image) = fixture();
        let before = fs::metadata(&image).unwrap().modified().unwrap();
        let plan = RestorePlan::create(&image, &sandbox).unwrap();
        let after = fs::metadata(&image).unwrap().modified().unwrap();
        assert_eq!(before, after);
        assert!(plan.render().contains("CHANGES=NONE"));
        fs::remove_dir_all(sandbox).unwrap();
    }

    #[test]
    fn changed_identity_cancels_restore() {
        let (sandbox, image) = fixture();
        let plan = RestorePlan::create(&image, &sandbox).unwrap();
        fs::OpenOptions::new()
            .write(true)
            .open(&image)
            .unwrap()
            .set_len(MINIMUM_IMAGE_SIZE + 1)
            .unwrap();
        assert_eq!(
            plan.revalidate(&sandbox).unwrap_err(),
            LetheError::RestoreIdentityChanged
        );
        fs::remove_dir_all(sandbox).unwrap();
    }

    #[test]
    fn image_outside_sandbox_is_rejected() {
        let (sandbox, _image) = fixture();
        let outside = sandbox.parent().unwrap().join("outside.vhdx");
        let file = fs::File::create(&outside).unwrap();
        file.set_len(MINIMUM_IMAGE_SIZE).unwrap();
        assert!(matches!(
            RestorePlan::create(&outside, &sandbox).unwrap_err(),
            LetheError::RestoreTargetUnsafe(_)
        ));
        fs::remove_file(outside).unwrap();
        fs::remove_dir_all(sandbox).unwrap();
    }

    #[test]
    fn physical_usb_targets_remain_disabled() {
        use crate::domain::{BusType, DeviceIdentity};
        use crate::storage::PhysicalUsbTarget;
        let identity = DeviceIdentity {
            disk_number: 99,
            label: "TEST".to_owned(),
            size_bytes: MINIMUM_IMAGE_SIZE,
            bus_type: BusType::Usb,
            is_boot: false,
            is_system: false,
            serial: Some("TEST".to_owned()),
        };
        let target = StorageTarget::Physical(PhysicalUsbTarget::new(identity).unwrap());
        assert_eq!(
            target.ensure_destructive_operations_allowed().unwrap_err(),
            LetheError::PhysicalUsbDisabled
        );
    }

    #[test]
    fn wrong_confirmation_is_rejected_before_helper() {
        let (sandbox, image) = fixture();
        let plan = RestorePlan::create(&image, &sandbox).unwrap();
        assert_eq!(
            plan.execute(&sandbox, "NO").unwrap_err(),
            LetheError::RestoreConfirmationInvalid
        );
        fs::remove_dir_all(sandbox).unwrap();
    }
}
