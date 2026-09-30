use crate::{domain::DriveLetter, error::LetheError};

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DriveStatus {
    Available,
    OccupiedByVeraCrypt { target: String },
    OccupiedByOther { target: String },
}

#[cfg(target_os = "windows")]
pub fn inspect_drive(letter: DriveLetter) -> Result<DriveStatus, LetheError> {
    const ERROR_FILE_NOT_FOUND: u32 = 2;
    const ERROR_PATH_NOT_FOUND: u32 = 3;

    #[link(name = "Kernel32")]
    unsafe extern "system" {
        fn QueryDosDeviceW(
            lp_device_name: *const u16,
            lp_target_path: *mut u16,
            ucch_max: u32,
        ) -> u32;
        fn GetLastError() -> u32;
    }

    let name = [letter.as_char() as u16, ':' as u16, 0];
    let mut buffer = vec![0_u16; 32_768];

    // SAFETY: `name` is NUL-terminated and `buffer` is writable for exactly the
    // length passed to the Windows API. Both allocations live through the call.
    let length = unsafe {
        QueryDosDeviceW(
            name.as_ptr(),
            buffer.as_mut_ptr(),
            buffer.len().try_into().expect("buffer length fits u32"),
        )
    };
    if length == 0 {
        // SAFETY: GetLastError has no arguments and is called immediately after
        // the failed Windows API call on the same thread.
        let code = unsafe { GetLastError() };
        return if matches!(code, ERROR_FILE_NOT_FOUND | ERROR_PATH_NOT_FOUND) {
            Ok(DriveStatus::Available)
        } else {
            Err(LetheError::DriveInspectionFailed(code))
        };
    }

    let end = buffer
        .iter()
        .position(|value| *value == 0)
        .unwrap_or(length as usize);
    let target = String::from_utf16_lossy(&buffer[..end]);
    if target.to_ascii_lowercase().contains("veracrypt") {
        Ok(DriveStatus::OccupiedByVeraCrypt { target })
    } else {
        Ok(DriveStatus::OccupiedByOther { target })
    }
}

#[cfg(not(target_os = "windows"))]
pub fn inspect_drive(_letter: DriveLetter) -> Result<DriveStatus, LetheError> {
    Err(LetheError::UnsupportedPlatform)
}

#[cfg(all(test, target_os = "windows"))]
mod tests {
    use super::*;

    #[test]
    fn system_drive_is_reported_as_occupied() {
        let status = inspect_drive(DriveLetter::new('C').unwrap()).unwrap();
        assert!(matches!(status, DriveStatus::OccupiedByOther { .. }));
    }
}
