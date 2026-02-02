pub mod capture;
pub mod config;
pub mod events;
pub mod state;

mod ipc;

use std::path::PathBuf;
use std::sync::Mutex;

use capture::SelectionResult;
use config::CaptureConfig;
use ipc::commands::{
    begin_recording, cancel_capture, finalize_complete, get_state, pause_recording,
    pause_recording_video, ping, reset_error, resume_recording, resume_recording_video,
    start_capture, start_recording_video, stop_recording, stop_recording_video, take_screenshot,
};
use state::StateMachine;

/// Generate a unique temporary file path for screenshots.
/// Returns a path in /tmp with format: opensnipping-{uuid}.png
pub fn generate_screenshot_temp_path() -> PathBuf {
    PathBuf::from(format!("/tmp/opensnipping-{}.png", uuid::Uuid::new_v4()))
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

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let mut builder = tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(AppState::default());

    #[cfg(debug_assertions)]
    {
        builder = builder.plugin(tauri_plugin_mcp_bridge::init());
    }

    builder
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
