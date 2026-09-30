use lethe::{
    domain::DriveLetter,
    mount::{DriveStatus, inspect_drive},
    veracrypt::{MINIMUM_VERSION, VeraCryptInstallation, Version},
};
use std::{env, path::PathBuf};

#[test]
#[ignore = "requires the locally installed and signed VeraCrypt binary"]
fn validates_real_veracrypt_installation() {
    let executable = env::var_os("LETHE_VERACRYPT")
        .map(PathBuf::from)
        .expect("LETHE_VERACRYPT must point to VeraCrypt.exe");
    let installation = VeraCryptInstallation::inspect(executable).unwrap();

    assert!(installation.version() >= Version::parse(MINIMUM_VERSION).unwrap());
    assert!(installation.signer().contains("IDRIX SARL"));
    assert!(installation.format_executable().is_file());
}

#[test]
#[ignore = "queries the real Windows drive namespace"]
fn integration_mount_letter_is_not_already_veracrypt() {
    let status = inspect_drive(DriveLetter::new('L').unwrap()).unwrap();
    assert!(!matches!(status, DriveStatus::OccupiedByVeraCrypt { .. }));
}
