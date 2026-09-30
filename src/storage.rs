use crate::{domain::DeviceIdentity, error::LetheError};
use std::path::{Path, PathBuf};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum StorageKind {
    LocalContainer,
    VirtualDisk,
    PhysicalUsb,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LocalContainerTarget {
    root: PathBuf,
}

impl LocalContainerTarget {
    pub fn new(root: impl Into<PathBuf>) -> Self {
        Self { root: root.into() }
    }

    pub fn root(&self) -> &Path {
        &self.root
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct VirtualDiskTarget {
    image: PathBuf,
}

impl VirtualDiskTarget {
    pub fn new(image: impl Into<PathBuf>) -> Self {
        Self {
            image: image.into(),
        }
    }

    pub fn image(&self) -> &Path {
        &self.image
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PhysicalUsbTarget {
    identity: DeviceIdentity,
}

impl PhysicalUsbTarget {
    pub fn new(identity: DeviceIdentity) -> Result<Self, LetheError> {
        identity.validate_usb_candidate()?;
        Ok(Self { identity })
    }

    pub fn identity(&self) -> &DeviceIdentity {
        &self.identity
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum StorageTarget {
    Local(LocalContainerTarget),
    Virtual(VirtualDiskTarget),
    Physical(PhysicalUsbTarget),
}

impl StorageTarget {
    pub fn kind(&self) -> StorageKind {
        match self {
            Self::Local(_) => StorageKind::LocalContainer,
            Self::Virtual(_) => StorageKind::VirtualDisk,
            Self::Physical(_) => StorageKind::PhysicalUsb,
        }
    }

    pub fn ensure_destructive_operations_allowed(&self) -> Result<(), LetheError> {
        match self {
            Self::Local(_) | Self::Virtual(_) => Ok(()),
            Self::Physical(_) => Err(LetheError::PhysicalUsbDisabled),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::{BusType, DeviceIdentity};

    fn safe_usb_identity() -> DeviceIdentity {
        DeviceIdentity {
            disk_number: 7,
            label: "TEST".to_owned(),
            size_bytes: 8 * 1024 * 1024,
            bus_type: BusType::Usb,
            is_boot: false,
            is_system: false,
            serial: Some("TEST-ONLY".to_owned()),
        }
    }

    #[test]
    fn local_targets_allow_destructive_tests() {
        let target = StorageTarget::Local(LocalContainerTarget::new("test-data"));
        assert!(target.ensure_destructive_operations_allowed().is_ok());
    }

    #[test]
    fn virtual_targets_allow_destructive_tests() {
        let target = StorageTarget::Virtual(VirtualDiskTarget::new("test.vhdx"));
        assert!(target.ensure_destructive_operations_allowed().is_ok());
    }

    #[test]
    fn physical_usb_is_blocked_even_when_identity_is_safe() {
        let target = StorageTarget::Physical(PhysicalUsbTarget::new(safe_usb_identity()).unwrap());
        assert_eq!(
            target.ensure_destructive_operations_allowed().unwrap_err(),
            LetheError::PhysicalUsbDisabled
        );
    }

    #[test]
    fn system_usb_is_rejected_at_construction() {
        let mut identity = safe_usb_identity();
        identity.is_system = true;
        assert!(PhysicalUsbTarget::new(identity).is_err());
    }
}
