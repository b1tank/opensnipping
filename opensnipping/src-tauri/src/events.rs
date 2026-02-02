use crate::capture::SelectionResult;
use crate::state::{CaptureError, CaptureState};
use serde::{Deserialize, Serialize};

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

/// Event emitted when screenshot capture completes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScreenshotCompleteEvent {
    pub path: String,
    pub width: u32,
    pub height: u32,
}

/// Event emitted when recording starts
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecordingStartedEvent {
    pub output_path: String,
}

/// Event emitted when recording stops and finalization completes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecordingStoppedEvent {
    pub path: String,
    pub duration_ms: u64,
    pub width: u32,
    pub height: u32,
}

/// Event names for Tauri event system
pub mod event_names {
    pub const STATE_CHANGED: &str = "capture:state_changed";
    pub const PERMISSION_NEEDED: &str = "capture:permission_needed";
    pub const PROGRESS: &str = "capture:progress";
    pub const ERROR: &str = "capture:error";
    pub const SELECTION_COMPLETE: &str = "capture:selection_complete";
    pub const SCREENSHOT_COMPLETE: &str = "capture:screenshot_complete";
    pub const RECORDING_STARTED: &str = "capture:recording_started";
    pub const RECORDING_STOPPED: &str = "capture:recording_stopped";
}
