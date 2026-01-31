// Linux capture backend using xdg-desktop-portal and PipeWire
//
// This module integrates with the Freedesktop portal for screen capture
// on Linux (Wayland and X11).

use crate::capture::{CaptureBackendError, ScreenshotResult, SelectionResult};
use crate::config::{CaptureConfig, CaptureSource};
use ashpd::desktop::screencast::{CursorMode, Screencast, SourceType};
use ashpd::desktop::PersistMode;
use std::path::Path;
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::{debug, error, info};

/// Linux capture backend using xdg-desktop-portal
#[derive(Debug)]
pub struct LinuxCaptureBackend {
    /// Active screencast session (if any)
    session: Arc<Mutex<Option<ActiveSession>>>,
}

/// Holds an active screencast session
#[derive(Debug)]
struct ActiveSession {
    /// PipeWire node ID (stored for future use in recording pipeline)
    _node_id: u32,
    /// Stream file descriptor (if available)
    _stream_fd: Option<std::os::fd::OwnedFd>,
}

impl LinuxCaptureBackend {
    pub fn new() -> Self {
        Self {
            session: Arc::new(Mutex::new(None)),
        }
    }

    /// Convert CaptureSource to portal SourceType
    fn source_type_from_config(source: &CaptureSource) -> SourceType {
        match source {
            CaptureSource::Screen => SourceType::Monitor,
            CaptureSource::Monitor => SourceType::Monitor,
            CaptureSource::Window => SourceType::Window,
            // Region selection is handled via Monitor + UI crop
            CaptureSource::Region => SourceType::Monitor,
        }
    }
}

impl Default for LinuxCaptureBackend {
    fn default() -> Self {
        Self::new()
    }
}

impl super::CaptureBackend for LinuxCaptureBackend {
    async fn request_selection(
        &self,
        config: &CaptureConfig,
    ) -> Result<SelectionResult, CaptureBackendError> {
        info!("Requesting screen selection via portal");

        // Create screencast proxy
        let screencast = Screencast::new()
            .await
            .map_err(|e| CaptureBackendError::PortalError(format!("Failed to connect to screencast portal: {}", e)))?;

        // Create session
        let session = screencast
            .create_session()
            .await
            .map_err(|e| CaptureBackendError::PortalError(format!("Failed to create session: {}", e)))?;

        debug!("Portal session created");

        // Determine source type from config
        let source_type = Self::source_type_from_config(&config.source);

        // Configure cursor mode
        let cursor_mode = if config.include_cursor {
            CursorMode::Embedded
        } else {
            CursorMode::Hidden
        };

        // Select sources - this shows the portal picker dialog
        screencast
            .select_sources(
                &session,
                cursor_mode,
                source_type.into(),
                false, // multiple sources
                None,  // restore token
                PersistMode::DoNot,
            )
            .await
            .map_err(|e| {
                // Portal errors often mean user cancelled
                if e.to_string().contains("cancelled") || e.to_string().contains("denied") {
                    CaptureBackendError::PermissionDenied("User cancelled selection".to_string())
                } else {
                    CaptureBackendError::PortalError(format!("Failed to select sources: {}", e))
                }
            })?;

        debug!("Source selection completed");

        // Start the screencast stream
        let streams = screencast
            .start(&session, None)
            .await
            .map_err(|e| {
                error!("Failed to start screencast: {}", e);
                if e.to_string().contains("cancelled") || e.to_string().contains("denied") {
                    CaptureBackendError::PermissionDenied("User denied screencast permission".to_string())
                } else {
                    CaptureBackendError::PortalError(format!("Failed to start screencast: {}", e))
                }
            })?
            .response()
            .map_err(|e| {
                error!("Failed to get screencast response: {}", e);
                CaptureBackendError::PortalError(format!("Failed to get response: {}", e))
            })?;

        // Get stream info
        if streams.streams().is_empty() {
            return Err(CaptureBackendError::NoSourceAvailable(
                "No streams returned from portal".to_string(),
            ));
        }

        let stream = &streams.streams()[0];
        let node_id = stream.pipe_wire_node_id();

        info!(
            "Got PipeWire node ID: {}, size: {:?}",
            node_id,
            stream.size()
        );

        // Store session info
        let mut session_lock = self.session.lock().await;
        *session_lock = Some(ActiveSession {
            _node_id: node_id,
            _stream_fd: None,
        });

        let (width, height) = stream.size().map(|(w, h)| (Some(w as u32), Some(h as u32))).unwrap_or((None, None));

        Ok(SelectionResult {
            node_id,
            stream_fd: None,
            width,
            height,
        })
    }

    async fn cancel_selection(&self) -> Result<(), CaptureBackendError> {
        info!("Cancelling selection");
        let mut session_lock = self.session.lock().await;
        *session_lock = None;
        Ok(())
    }

    async fn capture_screenshot(
        &self,
        _selection: &SelectionResult,
        _output_path: &Path,
    ) -> Result<ScreenshotResult, CaptureBackendError> {
        // TODO: Implement GStreamer pipeline in task 13e
        Err(CaptureBackendError::NotSupported(
            "Screenshot capture not yet implemented".to_string(),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_source_type_conversion() {
        assert!(matches!(
            LinuxCaptureBackend::source_type_from_config(&CaptureSource::Screen),
            SourceType::Monitor
        ));
        assert!(matches!(
            LinuxCaptureBackend::source_type_from_config(&CaptureSource::Window),
            SourceType::Window
        ));
        assert!(matches!(
            LinuxCaptureBackend::source_type_from_config(&CaptureSource::Region),
            SourceType::Monitor
        ));
    }

    #[test]
    fn test_backend_creation() {
        let backend = LinuxCaptureBackend::new();
        // Just verify it creates without panic
        assert!(backend.session.try_lock().is_ok());
    }
}
