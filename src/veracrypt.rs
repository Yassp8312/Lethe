use crate::{domain::DriveLetter, error::LetheError};
use std::{
    env,
    ffi::OsString,
    path::{Path, PathBuf},
    process::{Child, Command},
};

pub const MINIMUM_VERSION: &str = "1.26.29";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MountMode {
    ReadOnly,
    ReadWrite,
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct Version {
    pub major: u32,
    pub minor: u32,
    pub patch: u32,
}

impl Version {
    pub fn parse(value: &str) -> Result<Self, LetheError> {
        let mut parts = value.trim().split('.');
        let major = parse_numeric_component(parts.next())?;
        let minor = parse_numeric_component(parts.next())?;
        let patch = parse_numeric_component(parts.next())?;
        Ok(Self {
            major,
            minor,
            patch,
        })
    }
}

impl std::fmt::Display for Version {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "{}.{}.{}", self.major, self.minor, self.patch)
    }
}

fn parse_numeric_component(value: Option<&str>) -> Result<u32, LetheError> {
    let digits: String = value
        .ok_or(LetheError::DependencyInspectionFailed("VeraCrypt"))?
        .chars()
        .take_while(char::is_ascii_digit)
        .collect();
    if digits.is_empty() {
        return Err(LetheError::DependencyInspectionFailed("VeraCrypt"));
    }
    digits
        .parse()
        .map_err(|_| LetheError::DependencyInspectionFailed("VeraCrypt"))
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct VeraCryptInstallation {
    executable: PathBuf,
    format_executable: PathBuf,
    version: Version,
    signer: String,
}

impl VeraCryptInstallation {
    pub fn inspect(executable: impl Into<PathBuf>) -> Result<Self, LetheError> {
        let executable = executable.into();
        if !executable.is_file() {
            return Err(LetheError::DependencyMissing("VeraCrypt"));
        }

        let output = Command::new("powershell.exe")
            .args([
                "-NoProfile",
                "-NonInteractive",
                "-Command",
                "$ErrorActionPreference='Stop'; [Console]::OutputEncoding=[System.Text.Encoding]::UTF8; Import-Module (Join-Path $PSHOME 'Modules\\Microsoft.PowerShell.Security\\Microsoft.PowerShell.Security.psd1'); $i=Get-Item -LiteralPath $env:LETHE_INSPECT_TARGET; $s=Get-AuthenticodeSignature -LiteralPath $env:LETHE_INSPECT_TARGET; Write-Output ('VERSION=' + $i.VersionInfo.ProductVersion); Write-Output ('SIGNATURE=' + $s.Status); Write-Output ('SIGNER=' + $s.SignerCertificate.Subject)",
            ])
            .env("LETHE_INSPECT_TARGET", &executable)
            .output()
            .map_err(|error| LetheError::ProcessStartFailed {
                operation: "la inspeccion de VeraCrypt",
                kind: error.kind(),
            })?;
        if !output.status.success() {
            return Err(LetheError::DependencyInspectionFailed("VeraCrypt"));
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let version_text = field(&stdout, "VERSION=")
            .ok_or(LetheError::DependencyInspectionFailed("VeraCrypt"))?;
        let signature = field(&stdout, "SIGNATURE=")
            .ok_or(LetheError::DependencyInspectionFailed("VeraCrypt"))?;
        let signer =
            field(&stdout, "SIGNER=").ok_or(LetheError::DependencyInspectionFailed("VeraCrypt"))?;

        if signature != "Valid" || !signer.contains("IDRIX SARL") {
            return Err(LetheError::SignatureInvalid("VeraCrypt"));
        }
        let version = Version::parse(version_text)?;
        let minimum = Version::parse(MINIMUM_VERSION)?;
        if version < minimum {
            return Err(LetheError::VersionUnsupported {
                found: version.to_string(),
                minimum: MINIMUM_VERSION,
            });
        }

        let format_executable = executable
            .parent()
            .unwrap_or_else(|| Path::new("."))
            .join("VeraCrypt Format.exe");
        if !format_executable.is_file() {
            return Err(LetheError::DependencyMissing("VeraCrypt Format"));
        }

        Ok(Self {
            executable,
            format_executable,
            version,
            signer: signer.to_owned(),
        })
    }

    pub fn executable(&self) -> &Path {
        &self.executable
    }

    pub fn format_executable(&self) -> &Path {
        &self.format_executable
    }

    pub fn version(&self) -> Version {
        self.version
    }

    pub fn signer(&self) -> &str {
        &self.signer
    }

    pub fn mount_spec(
        &self,
        container: &Path,
        letter: DriveLetter,
        mode: MountMode,
    ) -> CommandSpec {
        let mut arguments = vec![
            "/q".into(),
            "/v".into(),
            container.as_os_str().to_owned(),
            "/l".into(),
            letter.argument().into(),
            "/a".into(),
            "/e".into(),
            "/c".into(),
            "n".into(),
            "/h".into(),
            "n".into(),
            "/secureDesktop".into(),
            "y".into(),
            "/protectMemory".into(),
            "y".into(),
            "/protectScreen".into(),
            "y".into(),
        ];
        if mode == MountMode::ReadOnly {
            arguments.extend([OsString::from("/m"), OsString::from("ro")]);
        }
        CommandSpec::new(self.executable.clone(), arguments)
    }

    pub fn unmount_spec(&self, letter: DriveLetter, force: bool) -> CommandSpec {
        let mut arguments = vec!["/q".into(), "/u".into(), letter.argument().into()];
        if force {
            arguments.push("/f".into());
        }
        CommandSpec::new(self.executable.clone(), arguments)
    }

    pub fn wipe_cache_spec(&self) -> CommandSpec {
        CommandSpec::new(self.executable.clone(), vec!["/q".into(), "/w".into()])
    }
}

fn field<'a>(output: &'a str, prefix: &str) -> Option<&'a str> {
    output
        .lines()
        .find_map(|line| line.strip_prefix(prefix).map(str::trim))
}

pub fn locate(base_dir: &Path) -> Option<PathBuf> {
    let mut candidates = Vec::new();
    if let Some(explicit) = env::var_os("LETHE_VERACRYPT") {
        candidates.push(PathBuf::from(explicit));
    }
    candidates.push(
        base_dir
            .join("tools")
            .join("VeraCrypt")
            .join("VeraCrypt.exe"),
    );
    candidates.push(PathBuf::from(r"C:\Program Files\VeraCrypt\VeraCrypt.exe"));
    candidates.push(PathBuf::from(
        r"C:\Program Files (x86)\VeraCrypt\VeraCrypt.exe",
    ));
    candidates.into_iter().find(|path| path.is_file())
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CommandSpec {
    program: PathBuf,
    arguments: Vec<OsString>,
}

impl CommandSpec {
    fn new(program: PathBuf, arguments: Vec<OsString>) -> Self {
        Self { program, arguments }
    }

    pub fn program(&self) -> &Path {
        &self.program
    }

    pub fn arguments(&self) -> &[OsString] {
        &self.arguments
    }

    pub fn spawn(&self, operation: &'static str) -> Result<Child, LetheError> {
        Command::new(&self.program)
            .args(&self.arguments)
            .spawn()
            .map_err(|error| LetheError::ProcessStartFailed {
                operation,
                kind: error.kind(),
            })
    }

    pub fn run(&self, operation: &'static str) -> Result<(), LetheError> {
        let status = Command::new(&self.program)
            .args(&self.arguments)
            .status()
            .map_err(|error| LetheError::ProcessStartFailed {
                operation,
                kind: error.kind(),
            })?;
        if !status.success() {
            return Err(LetheError::ProcessFailed {
                operation,
                exit_code: status.code(),
            });
        }
        Ok(())
    }

    pub fn contains_forbidden_secret_switch(&self) -> bool {
        self.arguments.iter().any(|argument| {
            let normalized = argument.to_string_lossy().to_ascii_lowercase();
            matches!(
                normalized.as_str(),
                "/p" | "/password" | "/pim" | "/tokenpin"
            )
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fake_installation() -> VeraCryptInstallation {
        VeraCryptInstallation {
            executable: PathBuf::from(r"C:\VeraCrypt\VeraCrypt.exe"),
            format_executable: PathBuf::from(r"C:\VeraCrypt\VeraCrypt Format.exe"),
            version: Version::parse(MINIMUM_VERSION).unwrap(),
            signer: "CN=IDRIX SARL".to_owned(),
        }
    }

    #[test]
    fn versions_are_ordered_numerically() {
        assert!(Version::parse("1.26.29").unwrap() > Version::parse("1.25.9").unwrap());
    }

    #[test]
    fn version_accepts_a_numeric_suffix() {
        assert_eq!(
            Version::parse("1.26.29.0").unwrap(),
            Version {
                major: 1,
                minor: 26,
                patch: 29
            }
        );
    }

    #[test]
    fn mount_specs_never_contain_password_switches() {
        let spec = fake_installation().mount_spec(
            Path::new(r"C:\test\vault.hc"),
            DriveLetter::new('L').unwrap(),
            MountMode::ReadOnly,
        );
        assert!(!spec.contains_forbidden_secret_switch());
        assert!(spec.arguments().iter().any(|item| item == "ro"));
        assert!(spec.arguments().iter().any(|item| item == "/secureDesktop"));
    }

    #[test]
    fn read_write_mount_does_not_add_read_only_option() {
        let spec = fake_installation().mount_spec(
            Path::new(r"C:\test\vault.hc"),
            DriveLetter::new('L').unwrap(),
            MountMode::ReadWrite,
        );
        assert!(!spec.arguments().iter().any(|item| item == "ro"));
    }

    #[test]
    fn unmount_spec_uses_current_switch_and_optional_force() {
        let installation = fake_installation();
        let normal = installation.unmount_spec(DriveLetter::new('L').unwrap(), false);
        let forced = installation.unmount_spec(DriveLetter::new('L').unwrap(), true);
        assert!(normal.arguments().iter().any(|item| item == "/u"));
        assert!(!normal.arguments().iter().any(|item| item == "/f"));
        assert!(forced.arguments().iter().any(|item| item == "/f"));
    }
}
