// Fake capture backend for testing
//
// This module provides a mock implementation of CaptureBackend
// for use in tests without requiring actual portal/PipeWire integration.

use crate::capture::{
    CaptureBackend, CaptureBackendError, RecordingResult, ScreenshotResult, SelectionResult,
};
use crate::config::CaptureConfig;
use image::{ImageBuffer, Rgb};
use std::path::Path;
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::sync::Arc;

/// Configurable fake backend for testing
#[derive(Debug, Clone)]
pub struct FakeCaptureBackend {
    /// Whether selection should succeed
    should_succeed: Arc<AtomicBool>,
    /// Error to return on failure
    error_type: Arc<std::sync::Mutex<FakeError>>,
    /// Fake node ID to return
    fake_node_id: Arc<AtomicU32>,
    /// Count of selection requests
    selection_count: Arc<AtomicU32>,
    /// Count of cancel requests
    cancel_count: Arc<AtomicU32>,
    /// Whether recording is in progress
    is_recording: Arc<AtomicBool>,
    /// Whether recording is paused
    is_paused: Arc<AtomicBool>,
    /// Recording start time (for duration calculation)
    recording_start: Arc<std::sync::Mutex<Option<std::time::Instant>>>,
    /// Output path for fake recording
    recording_output_path: Arc<std::sync::Mutex<Option<String>>>,
    /// Count of start_recording calls
    start_recording_count: Arc<AtomicU32>,
    /// Count of stop_recording calls
    stop_recording_count: Arc<AtomicU32>,
    /// Count of pause_recording calls
    pause_recording_count: Arc<AtomicU32>,
    /// Count of resume_recording calls
    resume_recording_count: Arc<AtomicU32>,
}

#[derive(Debug, Clone)]
pub enum FakeError {
    PermissionDenied,
    PortalError,
    NoSource,
}

impl Default for FakeCaptureBackend {
    fn default() -> Self {
        Self::new()
    }
}

impl FakeCaptureBackend {
    pub fn new() -> Self {
        Self {
            should_succeed: Arc::new(AtomicBool::new(true)),
            error_type: Arc::new(std::sync::Mutex::new(FakeError::PermissionDenied)),
            fake_node_id: Arc::new(AtomicU32::new(42)),
            selection_count: Arc::new(AtomicU32::new(0)),
            cancel_count: Arc::new(AtomicU32::new(0)),
            is_recording: Arc::new(AtomicBool::new(false)),
            is_paused: Arc::new(AtomicBool::new(false)),
            recording_start: Arc::new(std::sync::Mutex::new(None)),
            recording_output_path: Arc::new(std::sync::Mutex::new(None)),
            start_recording_count: Arc::new(AtomicU32::new(0)),
            stop_recording_count: Arc::new(AtomicU32::new(0)),
            pause_recording_count: Arc::new(AtomicU32::new(0)),
            resume_recording_count: Arc::new(AtomicU32::new(0)),
        }
    }

    /// Create a backend that always succeeds
    pub fn succeeding() -> Self {
        let backend = Self::new();
        backend.should_succeed.store(true, Ordering::SeqCst);
        backend
    }

    /// Create a backend that always fails with permission denied
    pub fn permission_denied() -> Self {
        let backend = Self::new();
        backend.should_succeed.store(false, Ordering::SeqCst);
        *backend.error_type.lock().unwrap() = FakeError::PermissionDenied;
        backend
    }

    /// Create a backend that always fails with portal error
    pub fn portal_error() -> Self {
        let backend = Self::new();
        backend.should_succeed.store(false, Ordering::SeqCst);
        *backend.error_type.lock().unwrap() = FakeError::PortalError;
        backend
    }

    /// Set whether selection should succeed
    pub fn set_should_succeed(&self, succeed: bool) {
        self.should_succeed.store(succeed, Ordering::SeqCst);
    }

    /// Set the fake node ID to return
    pub fn set_node_id(&self, node_id: u32) {
        self.fake_node_id.store(node_id, Ordering::SeqCst);
    }

    /// Get count of selection requests
    pub fn selection_count(&self) -> u32 {
        self.selection_count.load(Ordering::SeqCst)
    }

    /// Get count of cancel requests
    pub fn cancel_count(&self) -> u32 {
        self.cancel_count.load(Ordering::SeqCst)
    }

    /// Get count of start_recording calls
    pub fn start_recording_count(&self) -> u32 {
        self.start_recording_count.load(Ordering::SeqCst)
    }

    /// Get count of stop_recording calls
    pub fn stop_recording_count(&self) -> u32 {
        self.stop_recording_count.load(Ordering::SeqCst)
    }

    /// Get count of pause_recording calls
    pub fn pause_recording_count(&self) -> u32 {
        self.pause_recording_count.load(Ordering::SeqCst)
    }

    /// Get count of resume_recording calls
    pub fn resume_recording_count(&self) -> u32 {
        self.resume_recording_count.load(Ordering::SeqCst)
    }

    /// Check if currently recording
    pub fn is_recording(&self) -> bool {
        self.is_recording.load(Ordering::SeqCst)
    }

    /// Check if recording is paused
    pub fn is_paused(&self) -> bool {
        self.is_paused.load(Ordering::SeqCst)
    }
}

impl CaptureBackend for FakeCaptureBackend {
    async fn request_selection(
        &self,
        _config: &CaptureConfig,
    ) -> Result<SelectionResult, CaptureBackendError> {
        self.selection_count.fetch_add(1, Ordering::SeqCst);

        if self.should_succeed.load(Ordering::SeqCst) {
            Ok(SelectionResult {
                node_id: self.fake_node_id.load(Ordering::SeqCst),
                stream_fd: None,
                width: Some(1920),
                height: Some(1080),
            })
        } else {
            let error = self.error_type.lock().unwrap().clone();
            Err(match error {
                FakeError::PermissionDenied => {
                    CaptureBackendError::PermissionDenied("User cancelled".to_string())
                }
                FakeError::PortalError => {
                    CaptureBackendError::PortalError("Portal unavailable".to_string())
                }
                FakeError::NoSource => {
                    CaptureBackendError::NoSourceAvailable("No display found".to_string())
                }
            })
        }
    }

    async fn cancel_selection(&self) -> Result<(), CaptureBackendError> {
        self.cancel_count.fetch_add(1, Ordering::SeqCst);
        Ok(())
    }

    async fn capture_screenshot(
        &self,
        selection: &SelectionResult,
        output_path: &Path,
    ) -> Result<ScreenshotResult, CaptureBackendError> {
        if !self.should_succeed.load(Ordering::SeqCst) {
            let error = self.error_type.lock().unwrap().clone();
            return Err(match error {
                FakeError::PermissionDenied => CaptureBackendError::PermissionDenied(
                    "Screenshot permission denied".to_string(),
                ),
                FakeError::PortalError => CaptureBackendError::PortalError(
                    "Portal unavailable for screenshot".to_string(),
                ),
                FakeError::NoSource => {
                    CaptureBackendError::NoSourceAvailable("No display for screenshot".to_string())
                }
            });
        }

        // Use dimensions from selection if available, otherwise default
        let width = selection.width.unwrap_or(100);
        let height = selection.height.unwrap_or(100);

        // Generate a solid-color placeholder PNG (cornflower blue)
        let img: ImageBuffer<Rgb<u8>, Vec<u8>> = ImageBuffer::from_fn(width, height, |_, _| {
            Rgb([100u8, 149u8, 237u8]) // cornflower blue
        });

        img.save(output_path).map_err(|e| {
            CaptureBackendError::Internal(format!("Failed to save placeholder PNG: {}", e))
        })?;

        Ok(ScreenshotResult {
            path: output_path.to_string_lossy().to_string(),
            width,
            height,
        })
    }

    async fn start_recording(
        &self,
        selection: &SelectionResult,
        config: &CaptureConfig,
    ) -> Result<(), CaptureBackendError> {
        self.start_recording_count.fetch_add(1, Ordering::SeqCst);

        if !self.should_succeed.load(Ordering::SeqCst) {
            let error = self.error_type.lock().unwrap().clone();
            return Err(match error {
                FakeError::PermissionDenied => {
                    CaptureBackendError::PermissionDenied("Recording permission denied".to_string())
                }
                FakeError::PortalError => {
                    CaptureBackendError::PortalError("Recording portal error".to_string())
                }
                FakeError::NoSource => {
                    CaptureBackendError::NoSourceAvailable("No source for recording".to_string())
                }
            });
        }

        if self.is_recording.load(Ordering::SeqCst) {
            return Err(CaptureBackendError::Internal(
                "Recording already in progress".to_string(),
            ));
        }

        // Store recording state
        self.is_recording.store(true, Ordering::SeqCst);
        *self.recording_start.lock().unwrap() = Some(std::time::Instant::now());
        *self.recording_output_path.lock().unwrap() = Some(config.output_path.clone());

        // Store dimensions for later use (we don't actually record, just track state)
        let _ = selection; // acknowledge we received it

        Ok(())
    }

    async fn stop_recording(&self) -> Result<RecordingResult, CaptureBackendError> {
        self.stop_recording_count.fetch_add(1, Ordering::SeqCst);

        if !self.is_recording.load(Ordering::SeqCst) {
            return Err(CaptureBackendError::Internal(
                "No recording in progress".to_string(),
            ));
        }

        // Calculate duration
        let duration_ms = self
            .recording_start
            .lock()
            .unwrap()
            .map(|t| t.elapsed().as_millis() as u64)
            .unwrap_or(0);

        let output_path = self
            .recording_output_path
            .lock()
            .unwrap()
            .take()
            .unwrap_or_else(|| "/tmp/fake_recording.mp4".to_string());

        // Reset recording state
        self.is_recording.store(false, Ordering::SeqCst);
        self.is_paused.store(false, Ordering::SeqCst);
        *self.recording_start.lock().unwrap() = None;

        Ok(RecordingResult {
            path: output_path,
            duration_ms,
            width: 1920,
            height: 1080,
        })
    }

    async fn pause_recording(&self) -> Result<(), CaptureBackendError> {
        self.pause_recording_count.fetch_add(1, Ordering::SeqCst);

        if !self.is_recording.load(Ordering::SeqCst) {
            return Err(CaptureBackendError::Internal(
                "No recording in progress".to_string(),
            ));
        }

        if self.is_paused.load(Ordering::SeqCst) {
            return Err(CaptureBackendError::Internal(
                "Recording is already paused".to_string(),
            ));
        }

        self.is_paused.store(true, Ordering::SeqCst);
        Ok(())
    }

    async fn resume_recording(&self) -> Result<(), CaptureBackendError> {
        self.resume_recording_count.fetch_add(1, Ordering::SeqCst);

        if !self.is_recording.load(Ordering::SeqCst) {
            return Err(CaptureBackendError::Internal(
                "No recording in progress".to_string(),
            ));
        }

        if !self.is_paused.load(Ordering::SeqCst) {
            return Err(CaptureBackendError::Internal(
                "Recording is not paused".to_string(),
            ));
        }

        self.is_paused.store(false, Ordering::SeqCst);
        Ok(())
    }
}
