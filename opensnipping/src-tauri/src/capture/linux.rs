// Linux capture backend using xdg-desktop-portal and PipeWire
//
// This module integrates with the Freedesktop portal for screen capture
// on Linux (Wayland and X11).

use crate::capture::{CaptureBackendError, RecordingResult, ScreenshotResult, SelectionResult};
use crate::config::{CaptureConfig, CaptureSource, ContainerFormat};
use ashpd::desktop::screencast::{CursorMode, Screencast, SourceType};
use ashpd::desktop::PersistMode;
use gstreamer::prelude::*;
use std::path::Path;
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::{debug, error, info, warn};

/// H.264 encoders in order of preference (hardware first, then software fallback)
const H264_ENCODERS: &[&str] = &[
    "vaapih264enc",  // Intel/AMD iGPU via VA-API
    "nvh264enc",     // NVIDIA via NVENC
    "x264enc",       // Software fallback (libx264)
];

/// Detect the best available H.264 encoder from GStreamer registry
///
/// Returns the element factory name of the best available encoder,
/// preferring hardware encoders over software fallback.
/// Returns None if no H.264 encoder is available.
pub fn detect_available_encoder() -> Option<&'static str> {
    // Ensure GStreamer is initialized (safe to call multiple times)
    if gstreamer::init().is_err() {
        warn!("Failed to initialize GStreamer for encoder detection");
        return None;
    }

    for encoder in H264_ENCODERS {
        if let Some(factory) = gstreamer::ElementFactory::find(encoder) {
            // Verify the factory can create an element (plugin is fully loaded)
            if factory.create().build().is_ok() {
                debug!("Found available H.264 encoder: {}", encoder);
                return Some(encoder);
            }
        }
    }

    warn!("No H.264 encoder found in GStreamer registry");
    None
}

/// Get the GStreamer muxer element name for the given container format
pub fn get_muxer_for_container(container: ContainerFormat) -> &'static str {
    match container {
        ContainerFormat::Mp4 => "mp4mux",
        ContainerFormat::Mkv => "matroskamux",
    }
}

/// Active recording pipeline
///
/// Manages a GStreamer pipeline for screen recording, tracking start time
/// and providing methods to start and stop recording.
pub struct RecordingPipeline {
    /// The GStreamer pipeline
    pipeline: gstreamer::Pipeline,
    /// Output file path
    output_path: std::path::PathBuf,
    /// Recording start time (set when pipeline starts playing)
    start_time: Option<std::time::Instant>,
    /// Video dimensions (captured from pipeline)
    width: u32,
    height: u32,
}

impl RecordingPipeline {
    /// Create a new recording pipeline
    ///
    /// Builds the pipeline: pipewiresrc ! videoconvert ! videoscale ! encoder ! muxer ! filesink
    pub fn new(
        node_id: u32,
        output_path: std::path::PathBuf,
        fps: u8,
        container: ContainerFormat,
        width: Option<u32>,
        height: Option<u32>,
    ) -> Result<Self, CaptureBackendError> {
        // Initialize GStreamer
        gstreamer::init().map_err(|e| {
            CaptureBackendError::Internal(format!("Failed to initialize GStreamer: {}", e))
        })?;

        // Detect encoder
        let encoder = detect_available_encoder().ok_or_else(|| {
            CaptureBackendError::Internal("No H.264 encoder available".to_string())
        })?;

        // Get muxer for container format
        let muxer = get_muxer_for_container(container);

        // Build pipeline description
        // Note: video/x-raw,framerate caps filter enforces consistent output framerate
        let pipeline_str = format!(
            "pipewiresrc path={node_id} ! \
             videoconvert ! \
             videoscale ! \
             video/x-raw,framerate={fps}/1 ! \
             {encoder} ! \
             {muxer} ! \
             filesink location={output_path}",
            node_id = node_id,
            fps = fps,
            encoder = encoder,
            muxer = muxer,
            output_path = output_path.display()
        );

        debug!("Creating recording pipeline: {}", pipeline_str);

        let pipeline = gstreamer::parse::launch(&pipeline_str)
            .map_err(|e| CaptureBackendError::Internal(format!("Failed to create pipeline: {}", e)))?;

        let pipeline = pipeline.downcast::<gstreamer::Pipeline>().map_err(|_| {
            CaptureBackendError::Internal("Failed to downcast to Pipeline".to_string())
        })?;

        Ok(Self {
            pipeline,
            output_path,
            start_time: None,
            width: width.unwrap_or(1920),
            height: height.unwrap_or(1080),
        })
    }

    /// Start recording
    pub fn start(&mut self) -> Result<(), CaptureBackendError> {
        info!("Starting recording pipeline to {:?}", self.output_path);

        self.pipeline.set_state(gstreamer::State::Playing).map_err(|e| {
            CaptureBackendError::Internal(format!("Failed to start pipeline: {}", e))
        })?;

        self.start_time = Some(std::time::Instant::now());
        Ok(())
    }

    /// Stop recording and finalize output file
    ///
    /// Sends EOS to pipeline, waits for finalization, and returns the recording result.
    pub fn stop(&mut self) -> Result<RecordingResult, CaptureBackendError> {
        info!("Stopping recording pipeline");

        // Calculate duration
        let duration_ms = self
            .start_time
            .map(|t| t.elapsed().as_millis() as u64)
            .unwrap_or(0);

        // Send EOS to trigger proper file finalization
        self.pipeline.send_event(gstreamer::event::Eos::new());

        // Wait for EOS to be processed
        let bus = self.pipeline.bus().ok_or_else(|| {
            CaptureBackendError::Internal("Failed to get pipeline bus".to_string())
        })?;

        // Wait for EOS or error (up to 5 seconds)
        let result = loop {
            match bus.timed_pop(gstreamer::ClockTime::from_seconds(5)) {
                Some(msg) => {
                    use gstreamer::MessageView;
                    match msg.view() {
                        MessageView::Eos(..) => {
                            debug!("Recording pipeline reached EOS");
                            break Ok(());
                        }
                        MessageView::Error(err) => {
                            let debug_info = err.debug().map(|d| format!(" ({:?})", d)).unwrap_or_default();
                            error!("Recording pipeline error: {}{}", err.error(), debug_info);
                            break Err(CaptureBackendError::Internal(format!(
                                "Pipeline error: {}{}",
                                err.error(),
                                debug_info
                            )));
                        }
                        _ => {}
                    }
                }
                None => {
                    warn!("Timed out waiting for EOS");
                    break Ok(()); // Proceed anyway, file may still be valid
                }
            }
        };

        // Stop the pipeline
        let _ = self.pipeline.set_state(gstreamer::State::Null);

        result?;

        // Verify output file exists
        if !self.output_path.exists() {
            return Err(CaptureBackendError::Internal(
                "Recording file was not created".to_string(),
            ));
        }

        info!(
            "Recording complete: {:?} ({} ms)",
            self.output_path, duration_ms
        );

        Ok(RecordingResult {
            path: self.output_path.to_string_lossy().to_string(),
            duration_ms,
            width: self.width,
            height: self.height,
        })
    }
}

impl std::fmt::Debug for RecordingPipeline {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RecordingPipeline")
            .field("output_path", &self.output_path)
            .field("start_time", &self.start_time)
            .field("width", &self.width)
            .field("height", &self.height)
            .finish()
    }
}

/// Linux capture backend using xdg-desktop-portal
pub struct LinuxCaptureBackend {
    /// Active screencast session (if any)
    session: Arc<Mutex<Option<ActiveSession>>>,
    /// Active recording pipeline (if recording)
    recording: Arc<Mutex<Option<RecordingPipeline>>>,
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
            recording: Arc::new(Mutex::new(None)),
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

impl std::fmt::Debug for LinuxCaptureBackend {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("LinuxCaptureBackend")
            .field("session", &"<session>")
            .field("recording", &"<recording>")
            .finish()
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
        selection: &SelectionResult,
        config: &CaptureConfig,
    ) -> Result<(), CaptureBackendError> {
        info!("Starting recording from node {}", selection.node_id);

        // Check if already recording
        {
            let recording_lock = self.recording.lock().await;
            if recording_lock.is_some() {
                return Err(CaptureBackendError::Internal(
                    "Recording already in progress".to_string(),
                ));
            }
        }

        // Create recording pipeline
        let output_path = std::path::PathBuf::from(&config.output_path);
        let mut pipeline = RecordingPipeline::new(
            selection.node_id,
            output_path,
            config.fps,
            config.container,
            selection.width,
            selection.height,
        )?;

        // Start the pipeline
        pipeline.start()?;

        // Store the pipeline
        let mut recording_lock = self.recording.lock().await;
        *recording_lock = Some(pipeline);

        info!("Recording started successfully");
        Ok(())
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

    #[test]
    fn test_detect_available_encoder_returns_valid_element() {
        // This test verifies that if an encoder is found, it's one we expect
        if let Some(encoder) = detect_available_encoder() {
            assert!(
                H264_ENCODERS.contains(&encoder),
                "Detected encoder '{}' should be in our known list",
                encoder
            );
        }
        // Note: It's OK if no encoder is found (e.g., CI without GStreamer plugins)
    }

    #[test]
    fn test_muxer_for_mp4() {
        assert_eq!(get_muxer_for_container(ContainerFormat::Mp4), "mp4mux");
    }

    #[test]
    fn test_muxer_for_mkv() {
        assert_eq!(get_muxer_for_container(ContainerFormat::Mkv), "matroskamux");
    }
}
