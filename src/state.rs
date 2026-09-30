use crate::error::LetheError;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AppState {
    NoDevice,
    Detected,
    Prompting,
    MountedReadOnly,
    MountedReadWrite,
    Unmounting,
    RestorePlanned,
    Restoring,
    ErrorSafe,
}

impl AppState {
    pub const fn name(self) -> &'static str {
        match self {
            Self::NoDevice => "NoDevice",
            Self::Detected => "Detected",
            Self::Prompting => "Prompting",
            Self::MountedReadOnly => "MountedReadOnly",
            Self::MountedReadWrite => "MountedReadWrite",
            Self::Unmounting => "Unmounting",
            Self::RestorePlanned => "RestorePlanned",
            Self::Restoring => "Restoring",
            Self::ErrorSafe => "ErrorSafe",
        }
    }

    pub fn transition(self, event: AppEvent) -> Result<Self, LetheError> {
        use AppEvent as E;
        use AppState as S;

        let next = match (self, event) {
            (S::NoDevice, E::DeviceDetected) => S::Detected,
            (S::Detected, E::DeviceRemoved)
            | (S::RestorePlanned, E::DeviceRemoved)
            | (S::ErrorSafe, E::DeviceRemoved) => S::NoDevice,
            (S::Detected, E::OpenRequested) => S::Prompting,
            (S::Prompting, E::PasswordRejected | E::PromptCancelled) => S::Detected,
            (S::Prompting, E::MountedReadOnly) => S::MountedReadOnly,
            (S::Prompting, E::MountedReadWrite) => S::MountedReadWrite,
            (S::MountedReadOnly | S::MountedReadWrite, E::UnmountRequested) => S::Unmounting,
            (S::Unmounting, E::UnmountSucceeded) => S::Detected,
            (S::Detected, E::RestorePlanCreated) => S::RestorePlanned,
            (S::RestorePlanned, E::RestoreCancelled) => S::Detected,
            (S::RestorePlanned, E::RestoreConfirmed) => S::Restoring,
            (S::Restoring, E::RestoreSucceeded) => S::NoDevice,
            (
                S::Detected | S::Prompting | S::Unmounting | S::RestorePlanned | S::Restoring,
                E::OperationFailed,
            ) => S::ErrorSafe,
            (S::ErrorSafe, E::RecoverySucceeded) => S::Detected,
            _ => {
                return Err(LetheError::InvalidTransition {
                    from: self.name(),
                    event: event.name(),
                });
            }
        };
        Ok(next)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AppEvent {
    DeviceDetected,
    DeviceRemoved,
    OpenRequested,
    PasswordRejected,
    PromptCancelled,
    MountedReadOnly,
    MountedReadWrite,
    UnmountRequested,
    UnmountSucceeded,
    RestorePlanCreated,
    RestoreCancelled,
    RestoreConfirmed,
    RestoreSucceeded,
    OperationFailed,
    RecoverySucceeded,
}

impl AppEvent {
    pub const fn name(self) -> &'static str {
        match self {
            Self::DeviceDetected => "DeviceDetected",
            Self::DeviceRemoved => "DeviceRemoved",
            Self::OpenRequested => "OpenRequested",
            Self::PasswordRejected => "PasswordRejected",
            Self::PromptCancelled => "PromptCancelled",
            Self::MountedReadOnly => "MountedReadOnly",
            Self::MountedReadWrite => "MountedReadWrite",
            Self::UnmountRequested => "UnmountRequested",
            Self::UnmountSucceeded => "UnmountSucceeded",
            Self::RestorePlanCreated => "RestorePlanCreated",
            Self::RestoreCancelled => "RestoreCancelled",
            Self::RestoreConfirmed => "RestoreConfirmed",
            Self::RestoreSucceeded => "RestoreSucceeded",
            Self::OperationFailed => "OperationFailed",
            Self::RecoverySucceeded => "RecoverySucceeded",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn invalid_password_returns_to_detected() {
        let state = AppState::Detected
            .transition(AppEvent::OpenRequested)
            .unwrap()
            .transition(AppEvent::PasswordRejected)
            .unwrap();
        assert_eq!(state, AppState::Detected);
    }

    #[test]
    fn mounted_volume_must_unmount_before_restore() {
        let state = AppState::MountedReadOnly;
        assert!(state.transition(AppEvent::RestorePlanCreated).is_err());
    }

    #[test]
    fn restore_requires_a_plan() {
        assert!(
            AppState::Detected
                .transition(AppEvent::RestoreConfirmed)
                .is_err()
        );
    }

    #[test]
    fn complete_restore_ends_without_device() {
        let state = AppState::Detected
            .transition(AppEvent::RestorePlanCreated)
            .unwrap()
            .transition(AppEvent::RestoreConfirmed)
            .unwrap()
            .transition(AppEvent::RestoreSucceeded)
            .unwrap();
        assert_eq!(state, AppState::NoDevice);
    }

    #[test]
    fn operational_failure_enters_safe_error() {
        assert_eq!(
            AppState::Prompting
                .transition(AppEvent::OperationFailed)
                .unwrap(),
            AppState::ErrorSafe
        );
    }
}
