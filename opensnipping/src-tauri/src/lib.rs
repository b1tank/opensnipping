pub mod capture;
pub mod config;
pub mod events;
pub mod state;

use std::sync::Mutex;
use tauri::{AppHandle, Emitter};

use capture::{CaptureBackend, CaptureBackendError, RecordingResult, ScreenshotResult, SelectionResult};
use config::CaptureConfig;
use events::{event_names, ErrorEvent, RecordingStartedEvent, RecordingStoppedEvent, ScreenshotCompleteEvent, SelectionCompleteEvent, StateChangedEvent};
use state::{CaptureError, CaptureState, ErrorCode, StateMachine};
use std::path::PathBuf;
use tracing::info;

/// Generate a unique temporary file path for screenshots.
/// Returns a path in /tmp with format: opensnipping-{uuid}.png
pub fn generate_screenshot_temp_path() -> PathBuf {
    PathBuf::from(format!(
        "/tmp/opensnipping-{}.png",
        uuid::Uuid::new_v4()
    ))
}

/// Application state managed by Tauri
pub struct AppState {
    pub state_machine: Mutex<StateMachine>,
    pub config: Mutex<Option<CaptureConfig>>,
    /// Result from portal selection (PipeWire node info)
    pub selection: Mutex<Option<SelectionResult>>,
    /// Active backend instance for recording
    #[cfg(target_os = "linux")]
    pub backend: tokio::sync::Mutex<Option<capture::linux::LinuxCaptureBackend>>,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            state_machine: Mutex::new(StateMachine::new()),
            config: Mutex::new(None),
            selection: Mutex::new(None),
            #[cfg(target_os = "linux")]
            backend: tokio::sync::Mutex::new(None),
        }
    }
}

/// Helper to emit state change event
fn emit_state_change(app: &AppHandle, previous: CaptureState, current: CaptureState) {
    let _ = app.emit(
        event_names::STATE_CHANGED,
        StateChangedEvent {
            state: current,
            previous,
        },
    );
}

/// Helper to emit error event
fn emit_error(app: &AppHandle, error: &CaptureError) {
    let _ = app.emit(
        event_names::ERROR,
        ErrorEvent {
            error: error.clone(),
        },
    );
}

#[tauri::command]
fn ping() -> String {
    "Pong from Rust!".to_string()
}

#[tauri::command]
fn get_state(state: tauri::State<AppState>) -> CaptureState {
    state.state_machine.lock().unwrap().state()
}

/// Convert capture backend error to CaptureError
fn backend_error_to_capture_error(err: &CaptureBackendError) -> CaptureError {
    match err {
        CaptureBackendError::PermissionDenied(msg) => CaptureError {
            code: ErrorCode::PermissionDenied,
            message: msg.clone(),
        },
        CaptureBackendError::PortalError(msg) => CaptureError {
            code: ErrorCode::PortalError,
            message: msg.clone(),
        },
        CaptureBackendError::NoSourceAvailable(msg) => CaptureError {
            code: ErrorCode::PortalError,
            message: msg.clone(),
        },
        CaptureBackendError::NotSupported(msg) => CaptureError {
            code: ErrorCode::Unknown,
            message: msg.clone(),
        },
        CaptureBackendError::Internal(msg) => CaptureError {
            code: ErrorCode::Unknown,
            message: msg.clone(),
        },
    }
}

#[tauri::command]
async fn start_capture(
    app: AppHandle,
    state: tauri::State<'_, AppState>,
    config: CaptureConfig,
) -> Result<CaptureState, String> {
    // Validate config
    if let Err(err) = config.validate() {
        let mut sm = state.state_machine.lock().unwrap();
        let error = CaptureError {
            code: ErrorCode::InvalidConfig,
            message: format!("{}: {}", err.field, err.message),
        };
        sm.set_error(error.clone());
        emit_error(&app, &error);
        return Err(error.message);
    }

    // Store config
    let stored_config = config.clone();
    *state.config.lock().unwrap() = Some(stored_config);

    // Transition to Selecting
    let _selecting_state = {
        let mut sm = state.state_machine.lock().unwrap();
        let previous = sm.state();
        match sm.start_selecting() {
            Ok(new_state) => {
                emit_state_change(&app, previous, new_state);
                new_state
            }
            Err(e) => return Err(e.message),
        }
    };

    info!("Starting portal selection...");

    // Now call the portal (this shows the picker dialog)
    let backend = capture::get_backend();
    let selection_result = backend.request_selection(&config).await;

    match selection_result {
        Ok(selection) => {
            info!(
                "Portal selection successful: node_id={}",
                selection.node_id
            );

            // Store selection result
            *state.selection.lock().unwrap() = Some(selection.clone());

            // Emit selection complete event
            let _ = app.emit(
                event_names::SELECTION_COMPLETE,
                SelectionCompleteEvent { selection },
            );

            // Transition to Recording
            let mut sm = state.state_machine.lock().unwrap();
            let previous = sm.state();
            match sm.begin_recording() {
                Ok(new_state) => {
                    emit_state_change(&app, previous, new_state);
                    Ok(new_state)
                }
                Err(e) => Err(e.message),
            }
        }
        Err(backend_err) => {
            info!("Portal selection failed: {:?}", backend_err);

            let error = backend_error_to_capture_error(&backend_err);

            // Transition to Error
            let mut sm = state.state_machine.lock().unwrap();
            sm.set_error(error.clone());
            emit_error(&app, &error);

            // Clear config since we failed
            *state.config.lock().unwrap() = None;

            Err(error.message)
        }
    }
}

#[tauri::command]
fn cancel_capture(app: AppHandle, state: tauri::State<AppState>) -> Result<CaptureState, String> {
    let mut sm = state.state_machine.lock().unwrap();
    let previous = sm.state();
    match sm.cancel_selection() {
        Ok(new_state) => {
            emit_state_change(&app, previous, new_state);
            *state.config.lock().unwrap() = None;
            *state.selection.lock().unwrap() = None;
            Ok(new_state)
        }
        Err(e) => Err(e.message),
    }
}

#[tauri::command]
fn begin_recording(app: AppHandle, state: tauri::State<AppState>) -> Result<CaptureState, String> {
    let mut sm = state.state_machine.lock().unwrap();
    let previous = sm.state();
    match sm.begin_recording() {
        Ok(new_state) => {
            emit_state_change(&app, previous, new_state);
            Ok(new_state)
        }
        Err(e) => Err(e.message),
    }
}

#[tauri::command]
fn pause_recording(app: AppHandle, state: tauri::State<AppState>) -> Result<CaptureState, String> {
    let mut sm = state.state_machine.lock().unwrap();
    let previous = sm.state();
    match sm.pause() {
        Ok(new_state) => {
            emit_state_change(&app, previous, new_state);
            Ok(new_state)
        }
        Err(e) => Err(e.message),
    }
}

#[tauri::command]
fn resume_recording(app: AppHandle, state: tauri::State<AppState>) -> Result<CaptureState, String> {
    let mut sm = state.state_machine.lock().unwrap();
    let previous = sm.state();
    match sm.resume() {
        Ok(new_state) => {
            emit_state_change(&app, previous, new_state);
            Ok(new_state)
        }
        Err(e) => Err(e.message),
    }
}

/// Pause the GStreamer recording pipeline
#[tauri::command]
#[cfg(target_os = "linux")]
async fn pause_recording_video(
    state: tauri::State<'_, AppState>,
) -> Result<(), String> {
    info!("Pausing video recording...");

    let backend_lock = state.backend.lock().await;
    let backend = backend_lock.as_ref().ok_or_else(|| {
        "No recording in progress".to_string()
    })?;

    backend.pause_recording().await.map_err(|e| {
        format!("Failed to pause recording: {}", e)
    })
}

/// Resume the GStreamer recording pipeline
#[tauri::command]
#[cfg(target_os = "linux")]
async fn resume_recording_video(
    state: tauri::State<'_, AppState>,
) -> Result<(), String> {
    info!("Resuming video recording...");

    let backend_lock = state.backend.lock().await;
    let backend = backend_lock.as_ref().ok_or_else(|| {
        "No recording in progress".to_string()
    })?;

    backend.resume_recording().await.map_err(|e| {
        format!("Failed to resume recording: {}", e)
    })
}

/// Stub for non-Linux platforms
#[tauri::command]
#[cfg(not(target_os = "linux"))]
async fn pause_recording_video(
    _state: tauri::State<'_, AppState>,
) -> Result<(), String> {
    Err("Recording not implemented for this platform".to_string())
}

/// Stub for non-Linux platforms
#[tauri::command]
#[cfg(not(target_os = "linux"))]
async fn resume_recording_video(
    _state: tauri::State<'_, AppState>,
) -> Result<(), String> {
    Err("Recording not implemented for this platform".to_string())
}

#[tauri::command]
fn stop_recording(app: AppHandle, state: tauri::State<AppState>) -> Result<CaptureState, String> {
    let mut sm = state.state_machine.lock().unwrap();
    let previous = sm.state();
    match sm.stop() {
        Ok(new_state) => {
            emit_state_change(&app, previous, new_state);
            Ok(new_state)
        }
        Err(e) => Err(e.message),
    }
}

#[tauri::command]
fn finalize_complete(app: AppHandle, state: tauri::State<AppState>) -> Result<CaptureState, String> {
    let mut sm = state.state_machine.lock().unwrap();
    let previous = sm.state();
    match sm.finalize_complete() {
        Ok(new_state) => {
            emit_state_change(&app, previous, new_state);
            *state.config.lock().unwrap() = None;
            *state.selection.lock().unwrap() = None;
            Ok(new_state)
        }
        Err(e) => Err(e.message),
    }
}

#[tauri::command]
fn reset_error(app: AppHandle, state: tauri::State<AppState>) -> Result<CaptureState, String> {
    let mut sm = state.state_machine.lock().unwrap();
    let previous = sm.state();
    match sm.reset() {
        Ok(new_state) => {
            emit_state_change(&app, previous, new_state);
            Ok(new_state)
        }
        Err(e) => Err(e.message),
    }
}

/// Take a screenshot: request portal selection, capture frame, emit event
#[tauri::command]
async fn take_screenshot(
    app: AppHandle,
    _state: tauri::State<'_, AppState>,
    config: CaptureConfig,
) -> Result<ScreenshotResult, String> {
    // Validate config
    if let Err(err) = config.validate() {
        let error = CaptureError {
            code: ErrorCode::InvalidConfig,
            message: format!("{}: {}", err.field, err.message),
        };
        emit_error(&app, &error);
        return Err(error.message);
    }

    info!("Starting screenshot portal selection...");

    // Request selection via portal
    let backend = capture::get_backend();
    let selection_result = backend.request_selection(&config).await;

    let selection = match selection_result {
        Ok(sel) => {
            info!(
                "Screenshot selection successful: node_id={}",
                sel.node_id
            );
            sel
        }
        Err(backend_err) => {
            info!("Screenshot selection failed: {:?}", backend_err);
            let error = backend_error_to_capture_error(&backend_err);
            emit_error(&app, &error);
            return Err(error.message);
        }
    };

    // Generate unique output path
    let output_path = generate_screenshot_temp_path();

    info!("Capturing screenshot to {:?}...", output_path);

    // Capture the screenshot
    let screenshot_result = backend.capture_screenshot(&selection, &output_path).await;

    match screenshot_result {
        Ok(screenshot) => {
            info!(
                "Screenshot captured: {}x{} at {}",
                screenshot.width, screenshot.height, screenshot.path
            );

            // Emit screenshot complete event
            let _ = app.emit(
                event_names::SCREENSHOT_COMPLETE,
                ScreenshotCompleteEvent {
                    path: screenshot.path.clone(),
                    width: screenshot.width,
                    height: screenshot.height,
                },
            );

            Ok(screenshot)
        }
        Err(backend_err) => {
            info!("Screenshot capture failed: {:?}", backend_err);
            let error = backend_error_to_capture_error(&backend_err);
            emit_error(&app, &error);
            Err(error.message)
        }
    }
}

/// Start video recording with the current selection
#[tauri::command]
#[cfg(target_os = "linux")]
async fn start_recording_video(
    app: AppHandle,
    state: tauri::State<'_, AppState>,
) -> Result<(), String> {
    info!("Starting video recording...");

    // Get stored config and selection
    let config = state.config.lock().unwrap().clone().ok_or_else(|| {
        "No capture config set. Call start_capture first.".to_string()
    })?;

    let selection = state.selection.lock().unwrap().clone().ok_or_else(|| {
        "No selection available. Call start_capture first.".to_string()
    })?;

    // Create and store backend instance
    let backend = capture::linux::LinuxCaptureBackend::new();

    // Start recording
    match backend.start_recording(&selection, &config).await {
        Ok(()) => {
            info!("Recording started: {}", config.output_path);

            // Store backend for later stop
            let mut backend_lock = state.backend.lock().await;
            *backend_lock = Some(backend);

            // Emit recording started event
            let _ = app.emit(
                event_names::RECORDING_STARTED,
                RecordingStartedEvent {
                    output_path: config.output_path.clone(),
                },
            );

            Ok(())
        }
        Err(backend_err) => {
            info!("Recording start failed: {:?}", backend_err);
            let error = backend_error_to_capture_error(&backend_err);
            emit_error(&app, &error);
            Err(error.message)
        }
    }
}

/// Stop video recording and finalize the output file
#[tauri::command]
#[cfg(target_os = "linux")]
async fn stop_recording_video(
    app: AppHandle,
    state: tauri::State<'_, AppState>,
) -> Result<RecordingResult, String> {
    info!("Stopping video recording...");

    // Take backend from storage
    let backend = {
        let mut backend_lock = state.backend.lock().await;
        backend_lock.take().ok_or_else(|| {
            "No recording in progress".to_string()
        })?
    };

    // Stop recording
    match backend.stop_recording().await {
        Ok(result) => {
            info!(
                "Recording stopped: {} ({} ms)",
                result.path, result.duration_ms
            );

            // Transition state to Finalizing then Idle
            {
                let mut sm = state.state_machine.lock().unwrap();
                let previous = sm.state();
                if let Ok(finalizing) = sm.stop() {
                    emit_state_change(&app, previous, finalizing);
                    let previous_finalizing = finalizing;
                    if let Ok(idle) = sm.finalize_complete() {
                        emit_state_change(&app, previous_finalizing, idle);
                    }
                }
            }

            // Clear config and selection
            *state.config.lock().unwrap() = None;
            *state.selection.lock().unwrap() = None;

            // Emit recording stopped event
            let _ = app.emit(
                event_names::RECORDING_STOPPED,
                RecordingStoppedEvent {
                    path: result.path.clone(),
                    duration_ms: result.duration_ms,
                    width: result.width,
                    height: result.height,
                },
            );

            Ok(result)
        }
        Err(backend_err) => {
            info!("Recording stop failed: {:?}", backend_err);
            let error = backend_error_to_capture_error(&backend_err);
            emit_error(&app, &error);
            Err(error.message)
        }
    }
}

/// Stub for non-Linux platforms
#[tauri::command]
#[cfg(not(target_os = "linux"))]
async fn start_recording_video(
    _app: AppHandle,
    _state: tauri::State<'_, AppState>,
) -> Result<(), String> {
    Err("Recording not implemented for this platform".to_string())
}

/// Stub for non-Linux platforms
#[tauri::command]
#[cfg(not(target_os = "linux"))]
async fn stop_recording_video(
    _app: AppHandle,
    _state: tauri::State<'_, AppState>,
) -> Result<RecordingResult, String> {
    Err("Recording not implemented for this platform".to_string())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(AppState::default())
        .invoke_handler(tauri::generate_handler![
            ping,
            get_state,
            start_capture,
            cancel_capture,
            begin_recording,
            pause_recording,
            resume_recording,
            stop_recording,
            finalize_complete,
            reset_error,
            take_screenshot,
            start_recording_video,
            stop_recording_video,
            pause_recording_video,
            resume_recording_video,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn test_ping_returns_pong() {
        assert_eq!(ping(), "Pong from Rust!");
    }

    #[test]
    fn test_screenshot_temp_path_is_in_tmp_dir() {
        let path = generate_screenshot_temp_path();
        assert!(
            path.starts_with("/tmp"),
            "Path should be in /tmp directory: {:?}",
            path
        );
    }

    #[test]
    fn test_screenshot_temp_path_has_correct_prefix() {
        let path = generate_screenshot_temp_path();
        let filename = path.file_name().unwrap().to_str().unwrap();
        assert!(
            filename.starts_with("opensnipping-"),
            "Filename should start with 'opensnipping-': {}",
            filename
        );
    }

    #[test]
    fn test_screenshot_temp_path_has_png_extension() {
        let path = generate_screenshot_temp_path();
        assert_eq!(
            path.extension().and_then(|e| e.to_str()),
            Some("png"),
            "Path should have .png extension: {:?}",
            path
        );
    }

    #[test]
    fn test_screenshot_temp_path_is_unique() {
        // Generate multiple paths and ensure they are all unique
        let mut paths = HashSet::new();
        for _ in 0..100 {
            let path = generate_screenshot_temp_path();
            assert!(
                paths.insert(path.clone()),
                "Generated path should be unique: {:?}",
                path
            );
        }
        assert_eq!(paths.len(), 100);
    }
}
