// Fake capture backend for testing
//
// This module provides a mock implementation of CaptureBackend
// for use in tests without requiring actual portal/PipeWire integration.

use crate::capture::{CaptureBackend, CaptureBackendError, RecordingResult, ScreenshotResult, SelectionResult};
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
                FakeError::PermissionDenied => {
                    CaptureBackendError::PermissionDenied("Screenshot permission denied".to_string())
                }
                FakeError::PortalError => {
                    CaptureBackendError::PortalError("Portal unavailable for screenshot".to_string())
                }
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
        _selection: &SelectionResult,
        _config: &CaptureConfig,
    ) -> Result<(), CaptureBackendError> {
        // TODO: Implement in 16i
        Err(CaptureBackendError::NotSupported(
            "Recording not yet implemented in fake backend".to_string(),
        ))
    }

    async fn stop_recording(&self) -> Result<RecordingResult, CaptureBackendError> {
        // TODO: Implement in 16i
        Err(CaptureBackendError::NotSupported(
            "Recording not yet implemented in fake backend".to_string(),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{AudioConfig, CaptureSource, ContainerFormat};

    fn test_config() -> CaptureConfig {
        CaptureConfig {
            source: CaptureSource::Screen,
            fps: 30,
            include_cursor: true,
            audio: AudioConfig {
                system: false,
                mic: false,
            },
            container: ContainerFormat::Mp4,
            output_path: "/tmp/test.mp4".to_string(),
        }
    }

    #[tokio::test]
    async fn test_fake_backend_succeeds() {
        let backend = FakeCaptureBackend::succeeding();
        let config = test_config();

        let result = backend.request_selection(&config).await;
        assert!(result.is_ok());

        let selection = result.unwrap();
        assert_eq!(selection.node_id, 42);
        assert_eq!(backend.selection_count(), 1);
    }

    #[tokio::test]
    async fn test_fake_backend_permission_denied() {
        let backend = FakeCaptureBackend::permission_denied();
        let config = test_config();

        let result = backend.request_selection(&config).await;
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            CaptureBackendError::PermissionDenied(_)
        ));
    }

    #[tokio::test]
    async fn test_fake_backend_portal_error() {
        let backend = FakeCaptureBackend::portal_error();
        let config = test_config();

        let result = backend.request_selection(&config).await;
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            CaptureBackendError::PortalError(_)
        ));
    }

    #[tokio::test]
    async fn test_fake_backend_cancel() {
        let backend = FakeCaptureBackend::new();

        let result = backend.cancel_selection().await;
        assert!(result.is_ok());
        assert_eq!(backend.cancel_count(), 1);
    }

    #[tokio::test]
    async fn test_fake_backend_custom_node_id() {
        let backend = FakeCaptureBackend::succeeding();
        backend.set_node_id(123);

        let config = test_config();
        let result = backend.request_selection(&config).await.unwrap();
        assert_eq!(result.node_id, 123);
    }

    #[tokio::test]
    async fn test_fake_backend_screenshot_creates_file() {
        let backend = FakeCaptureBackend::succeeding();
        let selection = SelectionResult {
            node_id: 42,
            stream_fd: None,
            width: Some(64),
            height: Some(48),
        };

        let temp_dir = std::env::temp_dir();
        let output_path = temp_dir.join(format!("test_screenshot_{}.png", uuid::Uuid::new_v4()));

        let result = backend.capture_screenshot(&selection, &output_path).await;
        assert!(result.is_ok());

        let screenshot = result.unwrap();
        assert_eq!(screenshot.width, 64);
        assert_eq!(screenshot.height, 48);
        assert!(std::path::Path::new(&screenshot.path).exists());

        // Cleanup
        let _ = std::fs::remove_file(&output_path);
    }

    #[tokio::test]
    async fn test_fake_backend_screenshot_uses_default_dimensions() {
        let backend = FakeCaptureBackend::succeeding();
        let selection = SelectionResult {
            node_id: 42,
            stream_fd: None,
            width: None,
            height: None,
        };

        let temp_dir = std::env::temp_dir();
        let output_path = temp_dir.join(format!("test_screenshot_{}.png", uuid::Uuid::new_v4()));

        let result = backend.capture_screenshot(&selection, &output_path).await;
        assert!(result.is_ok());

        let screenshot = result.unwrap();
        assert_eq!(screenshot.width, 100); // default
        assert_eq!(screenshot.height, 100); // default

        // Cleanup
        let _ = std::fs::remove_file(&output_path);
    }

    #[tokio::test]
    async fn test_fake_backend_screenshot_fails_when_configured() {
        let backend = FakeCaptureBackend::permission_denied();
        let selection = SelectionResult {
            node_id: 42,
            stream_fd: None,
            width: Some(100),
            height: Some(100),
        };

        let temp_dir = std::env::temp_dir();
        let output_path = temp_dir.join(format!("test_screenshot_{}.png", uuid::Uuid::new_v4()));

        let result = backend.capture_screenshot(&selection, &output_path).await;
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            CaptureBackendError::PermissionDenied(_)
        ));

        // File should not exist
        assert!(!output_path.exists());
    }

    /// Test that ScreenshotResult has all fields required for ScreenshotCompleteEvent emission
    #[tokio::test]
    async fn test_screenshot_result_has_all_event_fields() {
        use crate::events::ScreenshotCompleteEvent;

        let backend = FakeCaptureBackend::succeeding();
        let selection = SelectionResult {
            node_id: 42,
            stream_fd: None,
            width: Some(800),
            height: Some(600),
        };

        let temp_dir = std::env::temp_dir();
        let output_path = temp_dir.join(format!("test_screenshot_{}.png", uuid::Uuid::new_v4()));

        let result = backend
            .capture_screenshot(&selection, &output_path)
            .await
            .unwrap();

        // Verify we can construct a ScreenshotCompleteEvent from the result
        let event = ScreenshotCompleteEvent {
            path: result.path.clone(),
            width: result.width,
            height: result.height,
        };

        // Verify event has expected values
        assert!(!event.path.is_empty(), "Event path should not be empty");
        assert!(
            event.path.ends_with(".png"),
            "Event path should end with .png"
        );
        assert_eq!(event.width, 800);
        assert_eq!(event.height, 600);

        // Cleanup
        let _ = std::fs::remove_file(&output_path);
    }

    /// Test the full selection → screenshot flow (mirrors take_screenshot command logic)
    #[tokio::test]
    async fn test_full_screenshot_flow_selection_to_capture() {
        let backend = FakeCaptureBackend::succeeding();
        backend.set_node_id(99);
        let config = test_config();

        // Step 1: Request selection (like portal picker)
        let selection = backend.request_selection(&config).await.unwrap();
        assert_eq!(selection.node_id, 99);
        assert_eq!(backend.selection_count(), 1);

        // Step 2: Capture screenshot using selection result
        let temp_dir = std::env::temp_dir();
        let output_path = temp_dir.join(format!("test_screenshot_{}.png", uuid::Uuid::new_v4()));

        let screenshot = backend
            .capture_screenshot(&selection, &output_path)
            .await
            .unwrap();

        // Verify screenshot uses selection dimensions
        assert_eq!(screenshot.width, selection.width.unwrap());
        assert_eq!(screenshot.height, selection.height.unwrap());
        assert!(std::path::Path::new(&screenshot.path).exists());

        // Cleanup
        let _ = std::fs::remove_file(&output_path);
    }

    /// Test that screenshot failure doesn't affect subsequent selection requests
    #[tokio::test]
    async fn test_screenshot_failure_is_isolated() {
        let backend = FakeCaptureBackend::new();
        let config = test_config();

        // First selection succeeds
        let selection1 = backend.request_selection(&config).await.unwrap();

        // Configure to fail
        backend.set_should_succeed(false);

        // Screenshot fails
        let temp_dir = std::env::temp_dir();
        let output_path = temp_dir.join(format!("test_screenshot_{}.png", uuid::Uuid::new_v4()));
        let screenshot_result = backend.capture_screenshot(&selection1, &output_path).await;
        assert!(screenshot_result.is_err());

        // Configure to succeed again
        backend.set_should_succeed(true);

        // New selection should succeed
        let selection2 = backend.request_selection(&config).await.unwrap();
        assert_eq!(selection2.node_id, 42);
        assert_eq!(backend.selection_count(), 2);
    }
}
