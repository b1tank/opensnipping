// Linux capture backend using xdg-desktop-portal and PipeWire
//
// This module integrates with the Freedesktop portal for screen capture
// on Linux (Wayland and X11).

use crate::capture::{CaptureBackendError, RecordingResult, ScreenshotResult, SelectionResult};
use crate::config::{CaptureConfig, CaptureSource};
use ashpd::desktop::screencast::{CursorMode, Screencast, SourceType};
use ashpd::desktop::PersistMode;
use gstreamer::prelude::*;
use std::path::Path;
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::{debug, error, info, warn};

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
        selection: &SelectionResult,
        output_path: &Path,
    ) -> Result<ScreenshotResult, CaptureBackendError> {
        info!(
            "Capturing screenshot from node {} to {:?}",
            selection.node_id, output_path
        );

        // Initialize GStreamer (safe to call multiple times)
        gstreamer::init().map_err(|e| {
            CaptureBackendError::Internal(format!("Failed to initialize GStreamer: {}", e))
        })?;

        // Variables to capture frame dimensions
        let width = Arc::new(AtomicU32::new(0));
        let height = Arc::new(AtomicU32::new(0));
        let got_frame = Arc::new(AtomicBool::new(false));

        // Build the pipeline: pipewiresrc ! videoconvert ! pngenc ! filesink
        let pipeline_str = format!(
            "pipewiresrc path={} num-buffers=1 ! videoconvert ! pngenc ! filesink location={}",
            selection.node_id,
            output_path.display()
        );

        debug!("Creating GStreamer pipeline: {}", pipeline_str);

        let pipeline = gstreamer::parse::launch(&pipeline_str)
            .map_err(|e| CaptureBackendError::Internal(format!("Failed to create pipeline: {}", e)))?;

        let pipeline = pipeline.downcast::<gstreamer::Pipeline>().map_err(|_| {
            CaptureBackendError::Internal("Failed to downcast to Pipeline".to_string())
        })?;

        // Add a pad probe to capture frame dimensions from videoconvert's sink pad
        let width_clone = Arc::clone(&width);
        let height_clone = Arc::clone(&height);
        let got_frame_clone = Arc::clone(&got_frame);

        // Get the videoconvert element to add a probe
        // We iterate over elements to find videoconvert
        for element in pipeline.iterate_elements() {
            if let Ok(elem) = element {
                let factory = elem.factory();
                if let Some(factory) = factory {
                    if factory.name() == "videoconvert" {
                        // Add probe to the sink pad
                        if let Some(pad) = elem.static_pad("sink") {
                            pad.add_probe(gstreamer::PadProbeType::BUFFER, move |_pad, info| {
                                if got_frame_clone.load(Ordering::SeqCst) {
                                    return gstreamer::PadProbeReturn::Ok;
                                }

                                // Try to get caps from the pad
                                if let Some(caps) = _pad.current_caps() {
                                    if let Some(s) = caps.structure(0) {
                                        if let (Ok(w), Ok(h)) = (s.get::<i32>("width"), s.get::<i32>("height")) {
                                            width_clone.store(w as u32, Ordering::SeqCst);
                                            height_clone.store(h as u32, Ordering::SeqCst);
                                            got_frame_clone.store(true, Ordering::SeqCst);
                                            debug!("Captured frame dimensions: {}x{}", w, h);
                                        }
                                    }
                                }

                                // Also try from probe info buffer
                                if let gstreamer::PadProbeInfo { data: Some(gstreamer::PadProbeData::Buffer(_)), .. } = info {
                                    got_frame_clone.store(true, Ordering::SeqCst);
                                }

                                gstreamer::PadProbeReturn::Ok
                            });
                        }
                        break;
                    }
                }
            }
        }

        // Start the pipeline
        pipeline.set_state(gstreamer::State::Playing).map_err(|e| {
            CaptureBackendError::Internal(format!("Failed to start pipeline: {}", e))
        })?;

        // Wait for EOS or error
        let bus = pipeline.bus().ok_or_else(|| {
            CaptureBackendError::Internal("Failed to get pipeline bus".to_string())
        })?;

        let result = loop {
            match bus.timed_pop(gstreamer::ClockTime::from_seconds(10)) {
                Some(msg) => {
                    use gstreamer::MessageView;
                    match msg.view() {
                        MessageView::Eos(..) => {
                            debug!("Pipeline reached EOS");
                            break Ok(());
                        }
                        MessageView::Error(err) => {
                            let debug_info = err.debug().map(|d| format!(" ({:?})", d)).unwrap_or_default();
                            error!(
                                "Pipeline error: {}{}",
                                err.error(),
                                debug_info
                            );
                            break Err(CaptureBackendError::Internal(format!(
                                "Pipeline error: {}{}",
                                err.error(),
                                debug_info
                            )));
                        }
                        MessageView::StateChanged(state_changed) => {
                            // Only log if from the pipeline itself
                            if state_changed.src().map(|s| s == pipeline.upcast_ref::<gstreamer::Object>()).unwrap_or(false) {
                                debug!(
                                    "Pipeline state: {:?} -> {:?}",
                                    state_changed.old(),
                                    state_changed.current()
                                );
                            }
                        }
                        _ => {}
                    }
                }
                None => {
                    warn!("Pipeline timed out waiting for EOS");
                    break Err(CaptureBackendError::Internal(
                        "Pipeline timed out".to_string(),
                    ));
                }
            }
        };

        // Cleanup: stop the pipeline
        let _ = pipeline.set_state(gstreamer::State::Null);

        // Check result
        result?;

        // Get final dimensions
        let final_width = width.load(Ordering::SeqCst);
        let final_height = height.load(Ordering::SeqCst);

        // If we couldn't get dimensions from the probe, try from selection
        let (final_width, final_height) = if final_width == 0 || final_height == 0 {
            selection
                .width
                .zip(selection.height)
                .unwrap_or((1920, 1080)) // fallback defaults
        } else {
            (final_width, final_height)
        };

        // Verify the output file was created
        if !output_path.exists() {
            return Err(CaptureBackendError::Internal(
                "Screenshot file was not created".to_string(),
            ));
        }

        info!(
            "Screenshot captured: {}x{} at {:?}",
            final_width, final_height, output_path
        );

        Ok(ScreenshotResult {
            path: output_path.to_string_lossy().to_string(),
            width: final_width,
            height: final_height,
        })
    }

    async fn start_recording(
        &self,
        _selection: &SelectionResult,
        _config: &CaptureConfig,
    ) -> Result<(), CaptureBackendError> {
        // TODO: Implement in 16f
        Err(CaptureBackendError::NotSupported(
            "Recording not yet implemented".to_string(),
        ))
    }

    async fn stop_recording(&self) -> Result<RecordingResult, CaptureBackendError> {
        // TODO: Implement in 16g
        Err(CaptureBackendError::NotSupported(
            "Recording not yet implemented".to_string(),
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
