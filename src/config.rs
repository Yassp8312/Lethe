use crate::{
    domain::{DriveLetter, RelativeContainerPath},
    error::LetheError,
};
use std::{fs, io, path::Path};

pub const CONFIG_FILE: &str = "lethe.conf";

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LetheConfig {
    pub container: RelativeContainerPath,
    pub mount_letter: DriveLetter,
}

impl Default for LetheConfig {
    fn default() -> Self {
        Self {
            container: RelativeContainerPath::new("vault.hc")
                .expect("the built-in container path is valid"),
            mount_letter: DriveLetter::new('L').expect("the built-in drive letter is valid"),
        }
    }
}

pub fn parse(contents: &str) -> Result<LetheConfig, LetheError> {
    let mut config = LetheConfig::default();
    let mut saw_container = false;
    let mut saw_mount_letter = false;

    for (index, original_line) in contents.lines().enumerate() {
        let line_number = index + 1;
        let line = original_line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let (key, value) = line.split_once('=').ok_or(LetheError::InvalidConfig {
            line: line_number,
            reason: "se esperaba clave=valor",
        })?;
        let key = key.trim();
        let value = value.trim();
        match key {
            "container" => {
                if saw_container {
                    return Err(LetheError::InvalidConfig {
                        line: line_number,
                        reason: "container esta repetido",
                    });
                }
                config.container = RelativeContainerPath::new(value)?;
                saw_container = true;
            }
            "mount_letter" => {
                if saw_mount_letter {
                    return Err(LetheError::InvalidConfig {
                        line: line_number,
                        reason: "mount_letter esta repetido",
                    });
                }
                config.mount_letter = DriveLetter::try_from(value)?;
                saw_mount_letter = true;
            }
            _ => {
                return Err(LetheError::UnknownConfigKey {
                    line: line_number,
                    key: key.to_owned(),
                });
            }
        }
    }
    Ok(config)
}

pub fn load_from(directory: &Path) -> Result<Option<LetheConfig>, LetheError> {
    let path = directory.join(CONFIG_FILE);
    let contents = match fs::read_to_string(path) {
        Ok(contents) => contents,
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(None),
        Err(error) => return Err(LetheError::io("leer la configuracion", &error)),
    };
    parse(&contents).map(Some)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_valid_config() {
        let config = parse("container=data\\vault.hc\nmount_letter=l\n").unwrap();
        assert_eq!(config.container.as_path(), Path::new(r"data\vault.hc"));
        assert_eq!(config.mount_letter.as_char(), 'L');
    }

    #[test]
    fn rejects_unknown_keys_without_echoing_values() {
        let error = parse("password=super-secret-value\n").unwrap_err();
        assert_eq!(error.code(), crate::error::ErrorCode::ConfigInvalid);
        assert!(!error.to_string().contains("super-secret-value"));
    }

    #[test]
    fn rejects_duplicate_keys() {
        assert!(parse("container=a.hc\ncontainer=b.hc\n").is_err());
    }

    #[test]
    fn rejects_parent_directory_escape() {
        assert_eq!(
            parse("container=..\\vault.hc\n").unwrap_err(),
            LetheError::UnsafeRelativePath
        );
    }
}
