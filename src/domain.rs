use crate::error::LetheError;
use std::{
    fmt,
    path::{Component, Path, PathBuf},
};

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DriveLetter(char);

impl DriveLetter {
    pub fn new(value: char) -> Result<Self, LetheError> {
        if !value.is_ascii_alphabetic() {
            return Err(LetheError::InvalidDriveLetter);
        }
        Ok(Self(value.to_ascii_uppercase()))
    }

    pub fn as_char(self) -> char {
        self.0
    }

    pub fn argument(self) -> String {
        self.0.to_string()
    }
}

impl TryFrom<&str> for DriveLetter {
    type Error = LetheError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let mut characters = value.trim().chars();
        let letter = characters.next().ok_or(LetheError::InvalidDriveLetter)?;
        if characters.next().is_some() {
            return Err(LetheError::InvalidDriveLetter);
        }
        Self::new(letter)
    }
}

impl fmt::Display for DriveLetter {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}", self.0)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RelativeContainerPath(PathBuf);

impl RelativeContainerPath {
    pub fn new(value: impl AsRef<Path>) -> Result<Self, LetheError> {
        let value = value.as_ref();
        if value.as_os_str().is_empty() || value.is_absolute() {
            return Err(LetheError::UnsafeRelativePath);
        }
        if !value
            .components()
            .all(|component| matches!(component, Component::Normal(_)))
        {
            return Err(LetheError::UnsafeRelativePath);
        }
        Ok(Self(value.to_path_buf()))
    }

    pub fn as_path(&self) -> &Path {
        &self.0
    }

    pub fn resolve_under(&self, base: impl AsRef<Path>) -> PathBuf {
        base.as_ref().join(&self.0)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BusType {
    Usb,
    Virtual,
    Other,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DeviceIdentity {
    pub disk_number: u32,
    pub label: String,
    pub size_bytes: u64,
    pub bus_type: BusType,
    pub is_boot: bool,
    pub is_system: bool,
    pub serial: Option<String>,
}

impl DeviceIdentity {
    pub fn validate_usb_candidate(&self) -> Result<(), LetheError> {
        if self.bus_type != BusType::Usb {
            return Err(LetheError::UnsafeDevice("el bus no es USB"));
        }
        if self.is_boot || self.is_system {
            return Err(LetheError::UnsafeDevice(
                "es un disco de arranque o del sistema",
            ));
        }
        if self.size_bytes == 0 {
            return Err(LetheError::UnsafeDevice("el tamaño informado es cero"));
        }
        if self
            .serial
            .as_deref()
            .is_none_or(|serial| serial.trim().is_empty())
        {
            return Err(LetheError::UnsafeDevice(
                "el dispositivo no informa un numero de serie",
            ));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn drive_letters_are_normalized() {
        assert_eq!(DriveLetter::try_from("l").unwrap().as_char(), 'L');
    }

    #[test]
    fn drive_letters_reject_more_than_one_character() {
        assert_eq!(
            DriveLetter::try_from("LL").unwrap_err(),
            LetheError::InvalidDriveLetter
        );
    }

    #[test]
    fn relative_container_rejects_escape() {
        assert_eq!(
            RelativeContainerPath::new("..\\vault.hc").unwrap_err(),
            LetheError::UnsafeRelativePath
        );
    }

    #[test]
    fn relative_container_resolves_under_base() {
        let path = RelativeContainerPath::new("data\\vault.hc").unwrap();
        assert_eq!(
            path.resolve_under(r"C:\Lethe"),
            PathBuf::from(r"C:\Lethe\data\vault.hc")
        );
    }

    #[test]
    fn usb_candidate_requires_a_serial_number() {
        let identity = DeviceIdentity {
            disk_number: 2,
            label: "JORGITO".to_owned(),
            size_bytes: 8_011_120_640,
            bus_type: BusType::Usb,
            is_boot: false,
            is_system: false,
            serial: Some("   ".to_owned()),
        };
        assert!(matches!(
            identity.validate_usb_candidate(),
            Err(LetheError::UnsafeDevice(_))
        ));
    }
}
