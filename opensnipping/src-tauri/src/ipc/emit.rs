use tauri::{AppHandle, Emitter};

use crate::events::{event_names, ErrorEvent, StateChangedEvent};
use crate::state::{CaptureError, CaptureState};

pub(crate) fn emit_state_change(app: &AppHandle, previous: CaptureState, current: CaptureState) {
    let _ = app.emit(
        event_names::STATE_CHANGED,
        StateChangedEvent {
            state: current,
            previous,
        },
    );
}

pub(crate) fn emit_error(app: &AppHandle, error: &CaptureError) {
    let _ = app.emit(
        event_names::ERROR,
        ErrorEvent {
            error: error.clone(),
        },
    );
}
