use crate::{error::ErrorCode, state::AppState, storage::StorageKind};
use std::{
    env, fmt, fs,
    io::Write,
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AuditEvent {
    Started,
    ConfigurationLoaded,
    TargetDetected(StorageKind),
    StateChanged { from: AppState, to: AppState },
    OperationRejected(ErrorCode),
    Stopped,
}

impl fmt::Display for AuditEvent {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Started => write!(formatter, "app_started"),
            Self::ConfigurationLoaded => write!(formatter, "configuration_loaded"),
            Self::TargetDetected(kind) => write!(formatter, "target_detected kind={kind:?}"),
            Self::StateChanged { from, to } => {
                write!(
                    formatter,
                    "state_changed from={} to={}",
                    from.name(),
                    to.name()
                )
            }
            Self::OperationRejected(code) => write!(formatter, "operation_rejected code={code:?}"),
            Self::Stopped => write!(formatter, "app_stopped"),
        }
    }
}

pub fn default_log_path() -> Option<PathBuf> {
    env::var_os("LOCALAPPDATA")
        .filter(|value| !value.is_empty())
        .map(PathBuf::from)
        .map(|root| root.join("Lethe").join("audit.log"))
}

pub fn append_default(event: AuditEvent) {
    if let Some(path) = default_log_path() {
        let _ = append_to(&path, event);
    }
}

pub fn append_to(path: &Path, event: AuditEvent) -> std::io::Result<()> {
    if let Ok(metadata) = fs::symlink_metadata(path)
        && metadata.file_type().is_symlink()
    {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "audit log cannot be a symbolic link",
        ));
    }
    let parent = path.parent().ok_or_else(|| {
        std::io::Error::new(std::io::ErrorKind::InvalidInput, "audit path has no parent")
    })?;
    fs::create_dir_all(parent)?;
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    let mut file = fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)?;
    writeln!(file, "time={timestamp} event={event}")
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn audit_events_only_accept_structured_non_secret_data() {
        let event = AuditEvent::OperationRejected(ErrorCode::ConfigInvalid).to_string();
        assert_eq!(event, "operation_rejected code=ConfigInvalid");
    }

    #[test]
    fn audit_file_contains_only_timestamp_and_structured_event() {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let root = env::temp_dir().join(format!("lethe-audit-{unique}"));
        let path = root.join("audit.log");
        append_to(&path, AuditEvent::Started).unwrap();
        let contents = fs::read_to_string(&path).unwrap();
        assert!(contents.contains("event=app_started"));
        assert!(!contents.contains("password"));
        fs::remove_dir_all(root).unwrap();
    }
}
