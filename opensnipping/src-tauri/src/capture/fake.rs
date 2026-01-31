// Fake capture backend for testing
//
// This module provides a mock implementation of CaptureBackend
// for use in tests without requiring actual portal/PipeWire integration.

use crate::capture::{CaptureBackend, CaptureBackendError, ScreenshotResult, SelectionResult};
use crate::config::CaptureConfig;
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
        _selection: &SelectionResult,
        output_path: &Path,
    ) -> Result<ScreenshotResult, CaptureBackendError> {
        // Return a fake successful result for testing
        // Actual placeholder image generation will be added in task 13f
        Ok(ScreenshotResult {
            path: output_path.to_string_lossy().to_string(),
            width: 1920,
            height: 1080,
        })
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
}
