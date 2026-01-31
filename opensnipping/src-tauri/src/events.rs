use serde::{Deserialize, Serialize};
use crate::state::{CaptureState, CaptureError};
use crate::capture::SelectionResult;

/// Event emitted when capture state changes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateChangedEvent {
    pub state: CaptureState,
    pub previous: CaptureState,
}

/// Event emitted when permission is needed
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PermissionKind {
    Screen,
    Microphone,
    SystemAudio,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PermissionNeededEvent {
    pub kind: PermissionKind,
}

/// Event emitted for recording progress
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProgressEvent {
    pub duration_ms: u64,
}

/// Event emitted on error
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorEvent {
    pub error: CaptureError,
}

/// Event emitted when portal selection completes successfully
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SelectionCompleteEvent {
    pub selection: SelectionResult,
}

/// Event names for Tauri event system
pub mod event_names {
    pub const STATE_CHANGED: &str = "capture:state_changed";
    pub const PERMISSION_NEEDED: &str = "capture:permission_needed";
    pub const PROGRESS: &str = "capture:progress";
    pub const ERROR: &str = "capture:error";
    pub const SELECTION_COMPLETE: &str = "capture:selection_complete";
}
