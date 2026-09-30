use crate::{
    domain::DriveLetter,
    error::LetheError,
    storage::{LocalContainerTarget, StorageTarget},
};
use std::{
    fs, io,
    path::{Path, PathBuf},
    process::Command,
};

const PACKAGE_FORMAT_VERSION: u32 = 1;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProvisionSources {
    pub lethe_executable: PathBuf,
    pub panel_script: PathBuf,
    pub veracrypt_directory: PathBuf,
    pub documentation_directory: PathBuf,
}

pub fn run_creation_wizard(target: &Path, sandbox_root: &Path) -> Result<String, LetheError> {
    let target = validate_existing_package(target, sandbox_root)?;
    validate_package(&target)?;
    let container = target.join("vault.hc");
    if container.exists() {
        return Err(LetheError::ProvisionFailed(
            "vault.hc ya existe; el asistente no lo sobrescribira",
        ));
    }

    let format_executable = target
        .join("tools")
        .join("VeraCrypt")
        .join("VeraCrypt Format.exe");
    let status = Command::new(&format_executable)
        .current_dir(&target)
        .status()
        .map_err(|error| LetheError::ProcessStartFailed {
            operation: "el asistente oficial de creacion de VeraCrypt",
            kind: error.kind(),
        })?;
    if !status.success() {
        return Err(LetheError::ProcessFailed {
            operation: "el asistente oficial de creacion de VeraCrypt",
            exit_code: status.code(),
        });
    }
    if !container.is_file() {
        return Err(LetheError::ProvisionFailed(
            "el asistente termino sin crear vault.hc en el paquete",
        ));
    }
    Ok(format!(
        "Contenedor creado y detectado correctamente: {}",
        container.display()
    ))
}

fn validate_existing_package(target: &Path, sandbox_root: &Path) -> Result<PathBuf, LetheError> {
    if !target.is_absolute() || !target.is_dir() {
        return Err(LetheError::ProvisionTargetUnsafe(
            "el paquete debe ser una carpeta absoluta existente",
        ));
    }
    let canonical_target = target
        .canonicalize()
        .map_err(|error| LetheError::io("validar el paquete preparado", &error))?;
    let canonical_sandbox = sandbox_root
        .canonicalize()
        .map_err(|error| LetheError::io("validar el area de trabajo", &error))?;
    if canonical_target == canonical_sandbox || !canonical_target.starts_with(&canonical_sandbox) {
        return Err(LetheError::ProvisionTargetUnsafe(
            "el paquete debe permanecer dentro del area local de trabajo",
        ));
    }
    Ok(canonical_target)
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProvisionRequest {
    pub target: PathBuf,
    pub sandbox_root: PathBuf,
    pub sources: ProvisionSources,
    pub mount_letter: DriveLetter,
    pub container_name: &'static str,
}

impl ProvisionRequest {
    pub fn validate(&self) -> Result<(), LetheError> {
        let storage = StorageTarget::Local(LocalContainerTarget::new(&self.target));
        storage.ensure_destructive_operations_allowed()?;

        if !self.target.is_absolute() || self.target.file_name().is_none() {
            return Err(LetheError::ProvisionTargetUnsafe(
                "debe ser una carpeta absoluta, no una raiz de disco",
            ));
        }
        if self.target.exists() {
            return Err(LetheError::ProvisionTargetExists(self.target.clone()));
        }

        let parent = self
            .target
            .parent()
            .ok_or(LetheError::ProvisionTargetUnsafe(
                "el destino no tiene carpeta padre",
            ))?;
        if !parent.is_dir() {
            return Err(LetheError::ProvisionTargetUnsafe(
                "la carpeta padre no existe",
            ));
        }
        let canonical_parent = parent
            .canonicalize()
            .map_err(|error| LetheError::io("validar la carpeta padre", &error))?;
        let canonical_sandbox = self
            .sandbox_root
            .canonicalize()
            .map_err(|error| LetheError::io("validar el area de trabajo", &error))?;
        if !canonical_parent.starts_with(&canonical_sandbox) {
            return Err(LetheError::ProvisionTargetUnsafe(
                "debe permanecer dentro del area local de trabajo",
            ));
        }

        for source in [
            &self.sources.lethe_executable,
            &self.sources.panel_script,
            &self.sources.veracrypt_directory,
            &self.sources.documentation_directory,
        ] {
            if !source.exists() {
                return Err(LetheError::ProvisionSourceMissing(source.clone()));
            }
        }
        for required in ["VeraCrypt.exe", "VeraCrypt Format.exe"] {
            let path = self.sources.veracrypt_directory.join(required);
            if !path.is_file() {
                return Err(LetheError::ProvisionSourceMissing(path));
            }
        }
        Ok(())
    }

    pub fn plan(&self) -> Result<ProvisionPlan, LetheError> {
        self.validate()?;
        Ok(ProvisionPlan {
            target: self.target.clone(),
            steps: vec![
                "Crear una carpeta transaccional dentro del area local".to_owned(),
                "Copiar Lethe.exe y el panel grafico".to_owned(),
                "Copiar el runtime validado de VeraCrypt".to_owned(),
                "Copiar la documentacion del proyecto".to_owned(),
                "Generar configuracion, lanzador y manifiesto sin secretos".to_owned(),
                "Validar la estructura preparada".to_owned(),
                "Publicar el paquete mediante renombrado local".to_owned(),
                "Dejar vault.hc sin crear para usar el asistente seguro".to_owned(),
            ],
        })
    }

    pub fn execute(&self) -> Result<ProvisionReport, LetheError> {
        let plan = self.plan()?;
        let parent = self.target.parent().expect("validated target has a parent");
        let target_name = self
            .target
            .file_name()
            .expect("validated target has a file name")
            .to_string_lossy();
        let staging = parent.join(format!(
            ".{target_name}.lethe-staging-{}",
            std::process::id()
        ));
        if staging.exists() {
            return Err(LetheError::ProvisionTargetExists(staging));
        }

        fs::create_dir(&staging)
            .map_err(|error| LetheError::io("crear el staging de preparacion", &error))?;
        let result = self.build_in(&staging).and_then(|file_count| {
            validate_package(&staging)?;
            fs::rename(&staging, &self.target)
                .map_err(|error| LetheError::io("publicar el paquete", &error))?;
            Ok(ProvisionReport {
                target: self.target.clone(),
                file_count,
                plan_steps: plan.steps.len(),
            })
        });

        if result.is_err() && staging.exists() {
            cleanup_staging(&staging, parent)?;
        }
        result
    }

    fn build_in(&self, staging: &Path) -> Result<usize, LetheError> {
        let mut file_count = 0;
        copy_file(&self.sources.lethe_executable, &staging.join("Lethe.exe"))?;
        file_count += 1;
        copy_file(&self.sources.panel_script, &staging.join("Lethe.ps1"))?;
        file_count += 1;

        let tools = staging.join("tools").join("VeraCrypt");
        file_count += copy_tree(&self.sources.veracrypt_directory, &tools)?;
        let documentation = staging.join("documentation");
        file_count += copy_tree(&self.sources.documentation_directory, &documentation)?;

        write_text(
            &staging.join("lethe.conf"),
            &format!(
                "# Configuracion generada por Lethe\ncontainer={}\nmount_letter={}\n",
                self.container_name, self.mount_letter
            ),
        )?;
        file_count += 1;
        write_text(
            &staging.join("Abrir Lethe.cmd"),
            "@echo off\r\nsetlocal\r\ncd /d \"%TEMP%\"\r\npowershell.exe -NoProfile -ExecutionPolicy Bypass -File \"%~dp0Lethe.ps1\" -HomePath \"%~dp0.\"\r\nendlocal\r\n",
        )?;
        file_count += 1;
        write_text(
            &staging.join("README.txt"),
            "LETHE - PAQUETE DE PREPARACION\r\n\r\nEste paquete todavia no contiene vault.hc.\r\nUse VeraCrypt Format desde tools\\VeraCrypt para crear el contenedor.\r\nNo escriba contrasenas en archivos o argumentos de consola.\r\n",
        )?;
        file_count += 1;
        write_text(
            &staging.join("lethe-package.manifest"),
            &format!(
                "format_version={PACKAGE_FORMAT_VERSION}\ncontainer={}\nmount_letter={}\nengine=VeraCrypt\nsecrets_stored=0\n",
                self.container_name, self.mount_letter
            ),
        )?;
        file_count += 1;
        Ok(file_count)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProvisionPlan {
    pub target: PathBuf,
    pub steps: Vec<String>,
}

impl ProvisionPlan {
    pub fn render(&self) -> String {
        let mut output = format!("PLAN DE PREPARACION\nDestino: {}\n", self.target.display());
        for (index, step) in self.steps.iter().enumerate() {
            output.push_str(&format!("{}. {step}\n", index + 1));
        }
        output.push_str("Cambios realizados: ninguno");
        output
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProvisionReport {
    pub target: PathBuf,
    pub file_count: usize,
    pub plan_steps: usize,
}

impl ProvisionReport {
    pub fn render(&self) -> String {
        format!(
            "Paquete preparado correctamente.\nDestino: {}\nArchivos: {}\nPasos validados: {}\nContenedor: pendiente de creacion interactiva",
            self.target.display(),
            self.file_count,
            self.plan_steps
        )
    }
}

fn copy_file(source: &Path, destination: &Path) -> Result<(), LetheError> {
    let metadata = fs::symlink_metadata(source)
        .map_err(|error| LetheError::io("inspeccionar un archivo fuente", &error))?;
    if metadata.file_type().is_symlink() || !metadata.is_file() {
        return Err(LetheError::ProvisionFailed(
            "un componente fuente no es un archivo regular",
        ));
    }
    if let Some(parent) = destination.parent() {
        fs::create_dir_all(parent)
            .map_err(|error| LetheError::io("crear una carpeta del paquete", &error))?;
    }
    fs::copy(source, destination)
        .map_err(|error| LetheError::io("copiar un archivo del paquete", &error))?;
    Ok(())
}

fn copy_tree(source: &Path, destination: &Path) -> Result<usize, LetheError> {
    let metadata = fs::symlink_metadata(source)
        .map_err(|error| LetheError::io("inspeccionar una carpeta fuente", &error))?;
    if metadata.file_type().is_symlink() || !metadata.is_dir() {
        return Err(LetheError::ProvisionFailed(
            "una fuente recursiva no es una carpeta regular",
        ));
    }
    fs::create_dir_all(destination)
        .map_err(|error| LetheError::io("crear una carpeta del paquete", &error))?;
    let mut count = 0;
    let entries = fs::read_dir(source)
        .map_err(|error| LetheError::io("enumerar una carpeta fuente", &error))?;
    for entry in entries {
        let entry = entry.map_err(|error| LetheError::io("leer una entrada fuente", &error))?;
        let file_type = entry
            .file_type()
            .map_err(|error| LetheError::io("inspeccionar una entrada fuente", &error))?;
        if file_type.is_symlink() {
            return Err(LetheError::ProvisionFailed(
                "los enlaces simbolicos no se copian al paquete",
            ));
        }
        let destination_entry = destination.join(entry.file_name());
        if file_type.is_dir() {
            count += copy_tree(&entry.path(), &destination_entry)?;
        } else if file_type.is_file() {
            copy_file(&entry.path(), &destination_entry)?;
            count += 1;
        } else {
            return Err(LetheError::ProvisionFailed(
                "se encontro un tipo de archivo no compatible",
            ));
        }
    }
    Ok(count)
}

fn write_text(path: &Path, contents: &str) -> Result<(), LetheError> {
    fs::write(path, contents)
        .map_err(|error| LetheError::io("escribir un archivo generado", &error))
}

fn validate_package(root: &Path) -> Result<(), LetheError> {
    for relative in [
        "Lethe.exe",
        "Lethe.ps1",
        "Abrir Lethe.cmd",
        "lethe.conf",
        "lethe-package.manifest",
        "tools\\VeraCrypt\\VeraCrypt.exe",
        "tools\\VeraCrypt\\VeraCrypt Format.exe",
        "documentation\\REQUISITOS.md",
    ] {
        if !root.join(relative).is_file() {
            return Err(LetheError::ProvisionFailed(
                "la estructura generada esta incompleta",
            ));
        }
    }
    if root.join("vault.hc").exists() {
        return Err(LetheError::ProvisionFailed(
            "el preparador no debe generar contrasenas ni contenedores",
        ));
    }
    Ok(())
}

fn cleanup_staging(staging: &Path, expected_parent: &Path) -> Result<(), LetheError> {
    if staging.parent() != Some(expected_parent)
        || !staging
            .file_name()
            .is_some_and(|name| name.to_string_lossy().contains(".lethe-staging-"))
    {
        return Err(LetheError::ProvisionTargetUnsafe(
            "se rechazo limpiar una carpeta temporal ambigua",
        ));
    }
    match fs::remove_dir_all(staging) {
        Ok(()) => Ok(()),
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(LetheError::io("revertir el staging incompleto", &error)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn fixture() -> (PathBuf, ProvisionRequest) {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let sandbox = std::env::temp_dir().join(format!("lethe-provision-{unique}"));
        let sources = sandbox.join("sources");
        let veracrypt = sources.join("veracrypt");
        let docs = sources.join("docs");
        fs::create_dir_all(&veracrypt).unwrap();
        fs::create_dir_all(&docs).unwrap();
        fs::write(sources.join("Lethe.exe"), b"test exe").unwrap();
        fs::write(sources.join("Lethe.ps1"), b"test panel").unwrap();
        fs::write(veracrypt.join("VeraCrypt.exe"), b"test vc").unwrap();
        fs::write(veracrypt.join("VeraCrypt Format.exe"), b"test vc format").unwrap();
        fs::write(docs.join("REQUISITOS.md"), b"test docs").unwrap();
        let request = ProvisionRequest {
            target: sandbox.join("output").join("Lethe"),
            sandbox_root: sandbox.clone(),
            sources: ProvisionSources {
                lethe_executable: sources.join("Lethe.exe"),
                panel_script: sources.join("Lethe.ps1"),
                veracrypt_directory: veracrypt,
                documentation_directory: docs,
            },
            mount_letter: DriveLetter::new('L').unwrap(),
            container_name: "vault.hc",
        };
        fs::create_dir(sandbox.join("output")).unwrap();
        (sandbox, request)
    }

    #[test]
    fn plan_does_not_create_target() {
        let (sandbox, request) = fixture();
        let plan = request.plan().unwrap();
        assert!(!request.target.exists());
        assert!(plan.render().contains("Cambios realizados: ninguno"));
        fs::remove_dir_all(sandbox).unwrap();
    }

    #[test]
    fn execute_publishes_complete_package() {
        let (sandbox, request) = fixture();
        let report = request.execute().unwrap();
        assert!(request.target.join("Lethe.exe").is_file());
        assert!(
            request
                .target
                .join("tools\\VeraCrypt\\VeraCrypt.exe")
                .is_file()
        );
        assert!(!request.target.join("vault.hc").exists());
        let launcher = fs::read_to_string(request.target.join("Abrir Lethe.cmd")).unwrap();
        assert!(launcher.contains("cd /d \"%TEMP%\""));
        assert!(launcher.contains("-HomePath \"%~dp0.\""));
        assert!(report.file_count >= 8);
        fs::remove_dir_all(sandbox).unwrap();
    }

    #[test]
    fn existing_target_is_never_overwritten() {
        let (sandbox, request) = fixture();
        fs::create_dir(&request.target).unwrap();
        assert!(matches!(
            request.execute().unwrap_err(),
            LetheError::ProvisionTargetExists(_)
        ));
        fs::remove_dir_all(sandbox).unwrap();
    }

    #[test]
    fn target_outside_sandbox_is_rejected() {
        let (sandbox, mut request) = fixture();
        request.target = std::env::temp_dir().join("outside-lethe-package");
        assert!(matches!(
            request.validate().unwrap_err(),
            LetheError::ProvisionTargetUnsafe(_)
        ));
        fs::remove_dir_all(sandbox).unwrap();
    }

    #[test]
    fn missing_source_leaves_no_target_or_staging() {
        let (sandbox, request) = fixture();
        fs::remove_file(&request.sources.panel_script).unwrap();
        assert!(request.execute().is_err());
        assert!(!request.target.exists());
        let staging_count = fs::read_dir(request.target.parent().unwrap())
            .unwrap()
            .filter_map(Result::ok)
            .filter(|entry| {
                entry
                    .file_name()
                    .to_string_lossy()
                    .contains("lethe-staging")
            })
            .count();
        assert_eq!(staging_count, 0);
        fs::remove_dir_all(sandbox).unwrap();
    }

    #[test]
    fn wizard_rejects_package_outside_sandbox() {
        let (sandbox, _request) = fixture();
        let outside = sandbox.parent().unwrap();
        assert!(matches!(
            validate_existing_package(outside, &sandbox).unwrap_err(),
            LetheError::ProvisionTargetUnsafe(_)
        ));
        fs::remove_dir_all(sandbox).unwrap();
    }
}
