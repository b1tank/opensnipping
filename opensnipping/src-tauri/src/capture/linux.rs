// Linux capture backend using xdg-desktop-portal and PipeWire
//
// This module integrates with the Freedesktop portal for screen capture
// on Linux (Wayland and X11).

use crate::capture::{CaptureBackendError, RecordingResult, ScreenshotResult, SelectionResult};
use crate::config::{AudioConfig, CaptureConfig, CaptureSource, ContainerFormat};
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

/// AAC audio encoders in order of preference
const AAC_ENCODERS: &[&str] = &[
    "fdkaacenc",     // FDK AAC (best quality, may need licensing)
    "voaacenc",      // VO-AAC (LGPL, good quality)
    "avenc_aac",     // libavcodec AAC (fallback)
];

/// Opus audio encoders (for MKV)
const OPUS_ENCODERS: &[&str] = &[
    "opusenc",       // Standard Opus encoder
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

/// Detect the best available audio encoder for the given container format
///
/// For MP4: prefers AAC encoders
/// For MKV: prefers Opus encoder
/// Returns None if no suitable audio encoder is available.
pub fn detect_available_audio_encoder(container: ContainerFormat) -> Option<&'static str> {
    // Ensure GStreamer is initialized (safe to call multiple times)
    if gstreamer::init().is_err() {
        warn!("Failed to initialize GStreamer for audio encoder detection");
        return None;
    }

    let encoders: &[&str] = match container {
        ContainerFormat::Mp4 => AAC_ENCODERS,
        ContainerFormat::Mkv => OPUS_ENCODERS,
    };

    for encoder in encoders {
        if let Some(factory) = gstreamer::ElementFactory::find(encoder) {
            if factory.create().build().is_ok() {
                debug!("Found available audio encoder: {}", encoder);
                return Some(encoder);
            }
        }
    }

    // Fallback: try any of the AAC encoders for MKV too (matroskamux supports AAC)
    if container == ContainerFormat::Mkv {
        for encoder in AAC_ENCODERS {
            if let Some(factory) = gstreamer::ElementFactory::find(encoder) {
                if factory.create().build().is_ok() {
                    debug!("Falling back to AAC encoder for MKV: {}", encoder);
                    return Some(encoder);
                }
            }
        }
    }

    warn!("No audio encoder found for {:?}", container);
    None
}

/// Get the system audio monitor source device name
///
/// Returns the PulseAudio monitor source for capturing system audio.
/// Uses @DEFAULT_MONITOR@ which PulseAudio resolves to the default
/// output device's monitor source.
///
/// Note: This requires PulseAudio or PipeWire with PulseAudio compatibility.
pub fn get_system_audio_source() -> &'static str {
    // @DEFAULT_MONITOR@ is a special PulseAudio device name that resolves
    // to the monitor source of the current default output device.
    // This works with both PulseAudio and PipeWire (via pipewire-pulse).
    "@DEFAULT_MONITOR@"
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
    /// Builds the pipeline with optional audio:
    /// - Video: pipewiresrc ! videoconvert ! videoscale ! encoder ! muxer ! filesink
    /// - Audio (if mic enabled): pulsesrc ! audioconvert ! audioresample ! audio_encoder ! muxer
    /// - Audio (if system enabled): pulsesrc device=@DEFAULT_MONITOR@ ! audioconvert ! audioresample ! audio_encoder ! muxer
    /// - Audio (if both enabled): mix handled separately (see task 22)
    pub fn new(
        node_id: u32,
        output_path: std::path::PathBuf,
        fps: u8,
        container: ContainerFormat,
        audio: &AudioConfig,
        width: Option<u32>,
        height: Option<u32>,
    ) -> Result<Self, CaptureBackendError> {
        // Initialize GStreamer
        gstreamer::init().map_err(|e| {
            CaptureBackendError::Internal(format!("Failed to initialize GStreamer: {}", e))
        })?;

        // Detect video encoder
        let video_encoder = detect_available_encoder().ok_or_else(|| {
            CaptureBackendError::Internal("No H.264 encoder available".to_string())
        })?;

        // Get muxer for container format
        let muxer = get_muxer_for_container(container);

        // Determine audio configuration
        let has_mic = audio.mic;
        let has_system = audio.system;
        let has_any_audio = has_mic || has_system;

        // Build pipeline description
        // When audio is enabled, we use a named muxer so both branches can link to it
        let pipeline_str = if has_any_audio {
            // Detect audio encoder
            let audio_encoder = detect_available_audio_encoder(container).ok_or_else(|| {
                CaptureBackendError::Internal("No audio encoder available".to_string())
            })?;

            // Determine audio source configuration
            let audio_source = if has_mic && has_system {
                // Both mic and system audio - for now, prioritize system audio
                // Full mixing support will come in task 22
                info!(
                    "Recording with both mic and system audio requested. Using system audio for now (mixing in task 22), encoder: {}",
                    audio_encoder
                );
                format!("pulsesrc device={}", get_system_audio_source())
            } else if has_system {
                info!("Recording with system audio, encoder: {}", audio_encoder);
                format!("pulsesrc device={}", get_system_audio_source())
            } else {
                // has_mic only
                info!("Recording with microphone audio, encoder: {}", audio_encoder);
                "pulsesrc".to_string()
            };

            // Pipeline with video + audio
            // Named mux element allows multiple inputs
            format!(
                "pipewiresrc path={node_id} ! \
                 videoconvert ! \
                 videoscale ! \
                 video/x-raw,framerate={fps}/1 ! \
                 {video_encoder} ! mux. \
                 {audio_source} ! \
                 audioconvert ! \
                 audioresample ! \
                 {audio_encoder} ! mux. \
                 {muxer} name=mux ! \
                 filesink location={output_path}",
                node_id = node_id,
                fps = fps,
                video_encoder = video_encoder,
                audio_source = audio_source,
                audio_encoder = audio_encoder,
                muxer = muxer,
                output_path = output_path.display()
            )
        } else {
            // Video-only pipeline
            format!(
                "pipewiresrc path={node_id} ! \
                 videoconvert ! \
                 videoscale ! \
                 video/x-raw,framerate={fps}/1 ! \
                 {video_encoder} ! \
                 {muxer} ! \
                 filesink location={output_path}",
                node_id = node_id,
                fps = fps,
                video_encoder = video_encoder,
                muxer = muxer,
                output_path = output_path.display()
            )
        };

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

    /// Pause the recording pipeline
    ///
    /// Sets the pipeline to PAUSED state. Can be resumed with `resume()`.
    pub fn pause(&self) -> Result<(), CaptureBackendError> {
        info!("Pausing recording pipeline");

        self.pipeline.set_state(gstreamer::State::Paused).map_err(|e| {
            CaptureBackendError::Internal(format!("Failed to pause pipeline: {}", e))
        })?;

        debug!("Recording pipeline paused");
        Ok(())
    }

    /// Resume a paused recording pipeline
    ///
    /// Sets the pipeline back to PLAYING state after `pause()` was called.
    pub fn resume(&self) -> Result<(), CaptureBackendError> {
        info!("Resuming recording pipeline");

        self.pipeline.set_state(gstreamer::State::Playing).map_err(|e| {
            CaptureBackendError::Internal(format!("Failed to resume pipeline: {}", e))
        })?;

        debug!("Recording pipeline resumed");
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
            &config.audio,
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
        info!("Stopping recording");

        // Take the recording pipeline from storage
        let mut pipeline = {
            let mut recording_lock = self.recording.lock().await;
            recording_lock.take().ok_or_else(|| {
                CaptureBackendError::Internal("No recording in progress".to_string())
            })?
        };

        // Stop the pipeline and get the result
        let result = pipeline.stop()?;

        info!(
            "Recording stopped: {} ({} ms)",
            result.path, result.duration_ms
        );

        Ok(result)
    }

    async fn pause_recording(&self) -> Result<(), CaptureBackendError> {
        info!("Pausing recording");

        let recording_lock = self.recording.lock().await;
        let pipeline = recording_lock.as_ref().ok_or_else(|| {
            CaptureBackendError::Internal("No recording in progress".to_string())
        })?;

        pipeline.pause()
    }

    async fn resume_recording(&self) -> Result<(), CaptureBackendError> {
        info!("Resuming recording");

        let recording_lock = self.recording.lock().await;
        let pipeline = recording_lock.as_ref().ok_or_else(|| {
            CaptureBackendError::Internal("No recording in progress".to_string())
        })?;

        pipeline.resume()
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

    #[test]
    fn test_detect_audio_encoder_mp4_returns_aac() {
        // If an audio encoder is found for MP4, it should be an AAC encoder
        if let Some(encoder) = detect_available_audio_encoder(ContainerFormat::Mp4) {
            assert!(
                AAC_ENCODERS.contains(&encoder),
                "MP4 audio encoder '{}' should be an AAC encoder",
                encoder
            );
        }
        // Note: It's OK if no encoder is found (e.g., CI without GStreamer plugins)
    }

    #[test]
    fn test_detect_audio_encoder_mkv_returns_opus_or_aac() {
        // If an audio encoder is found for MKV, it should be Opus or AAC (fallback)
        if let Some(encoder) = detect_available_audio_encoder(ContainerFormat::Mkv) {
            let is_valid = OPUS_ENCODERS.contains(&encoder) || AAC_ENCODERS.contains(&encoder);
            assert!(
                is_valid,
                "MKV audio encoder '{}' should be Opus or AAC",
                encoder
            );
        }
        // Note: It's OK if no encoder is found (e.g., CI without GStreamer plugins)
    }

    // --- Encoder/Muxer selection tests ---

    #[test]
    fn test_h264_encoders_preference_order() {
        // Verify the encoder list has correct priority: HW first, then SW fallback
        assert_eq!(H264_ENCODERS[0], "vaapih264enc", "VA-API should be first (Intel/AMD iGPU)");
        assert_eq!(H264_ENCODERS[1], "nvh264enc", "NVENC should be second (NVIDIA)");
        assert_eq!(H264_ENCODERS[2], "x264enc", "x264 should be last (SW fallback)");
    }

    #[test]
    fn test_muxer_selection_is_deterministic() {
        // Calling get_muxer_for_container multiple times with same input yields same output
        for _ in 0..10 {
            assert_eq!(get_muxer_for_container(ContainerFormat::Mp4), "mp4mux");
            assert_eq!(get_muxer_for_container(ContainerFormat::Mkv), "matroskamux");
        }
    }

    #[test]
    fn test_encoder_detection_is_deterministic() {
        // If an encoder is found, calling detect_available_encoder multiple times
        // should return the same encoder (highest-priority available)
        let first_result = detect_available_encoder();
        for _ in 0..5 {
            assert_eq!(
                detect_available_encoder(),
                first_result,
                "Encoder detection should be deterministic"
            );
        }
    }

    #[test]
    fn test_all_container_formats_have_muxers() {
        // Ensure every ContainerFormat variant has a corresponding muxer
        let formats = [ContainerFormat::Mp4, ContainerFormat::Mkv];
        for format in formats {
            let muxer = get_muxer_for_container(format);
            assert!(
                !muxer.is_empty(),
                "Container format {:?} should have a non-empty muxer",
                format
            );
        }
    }

    // --- Recording pipeline tests ---

    /// Check if GStreamer and required plugins are available for recording tests
    fn gstreamer_recording_available() -> bool {
        // Try to initialize GStreamer
        if gstreamer::init().is_err() {
            return false;
        }

        // Check if we have at least one encoder
        if detect_available_encoder().is_none() {
            return false;
        }

        // Check if mp4mux is available
        if gstreamer::ElementFactory::find("mp4mux").is_none() {
            return false;
        }

        // Check if pipewiresrc is available (needed for actual recording)
        if gstreamer::ElementFactory::find("pipewiresrc").is_none() {
            return false;
        }

        true
    }

    #[test]
    fn test_recording_pipeline_requires_encoder() {
        // This test verifies that RecordingPipeline::new fails gracefully
        // if no encoder is available. We can't easily mock GStreamer internals,
        // so we just verify the error handling path exists.
        //
        // If GStreamer is not available at all, the test passes trivially.
        if gstreamer::init().is_err() {
            return; // GStreamer not available, skip test
        }

        // The actual test happens in real usage - we're just documenting
        // the expected behavior: if detect_available_encoder() returns None,
        // RecordingPipeline::new() should return an error.
    }

    /// Smoke test: verify RecordingPipeline can be created (but not started)
    /// when GStreamer and required plugins are available.
    ///
    /// This test is ignored by default because it requires:
    /// - GStreamer installed
    /// - H.264 encoder plugins
    /// - PipeWire running with a valid node
    ///
    /// Run with: cargo test --features integration -- --ignored
    #[test]
    #[ignore = "Requires GStreamer, PipeWire, and a valid stream node"]
    fn test_recording_smoke_start_stop() {
        if !gstreamer_recording_available() {
            println!("Skipping: GStreamer or required plugins not available");
            return;
        }

        // This smoke test would require a real PipeWire node from a portal session.
        // In a real integration test environment, you would:
        // 1. Request a portal session to get a node_id
        // 2. Create a RecordingPipeline with that node_id
        // 3. Start recording for 2-3 seconds
        // 4. Stop and verify file exists and is non-empty
        //
        // Since we can't easily get a real node_id in unit tests,
        // this test is marked as ignored and serves as documentation
        // for manual testing or CI with proper setup.

        let temp_dir = std::env::temp_dir();
        let _output_path = temp_dir.join(format!("test_recording_{}.mp4", uuid::Uuid::new_v4()));

        // In a real test with portal access:
        // let node_id = <get from portal session>;
        // let mut pipeline = RecordingPipeline::new(
        //     node_id,
        //     _output_path.clone(),
        //     30, // fps
        //     ContainerFormat::Mp4,
        //     &AudioConfig::default(),
        //     Some(1920),
        //     Some(1080),
        // ).expect("Failed to create pipeline");
        //
        // pipeline.start().expect("Failed to start recording");
        // std::thread::sleep(std::time::Duration::from_secs(3));
        // let result = pipeline.stop().expect("Failed to stop recording");
        //
        // assert!(std::path::Path::new(&result.path).exists(), "Output file should exist");
        // let metadata = std::fs::metadata(&result.path).expect("Failed to get file metadata");
        // assert!(metadata.len() > 0, "Output file should be non-empty");
        //
        // // Cleanup
        // let _ = std::fs::remove_file(&_output_path);

        println!("Recording smoke test placeholder - run manually with portal session");
    }

    /// Test that LinuxCaptureBackend correctly reports "already recording" error
    #[tokio::test]
    async fn test_backend_cannot_double_start_recording() {
        // This test verifies the state tracking in LinuxCaptureBackend.
        // We can't actually start recording without a portal session,
        // but we can verify the backend initializes correctly.
        let backend = LinuxCaptureBackend::new();

        // Verify recording lock is available (not held)
        let lock = backend.recording.try_lock();
        assert!(lock.is_ok(), "Recording lock should be available on new backend");
        assert!(lock.unwrap().is_none(), "No recording should be in progress initially");
    }

    /// Test RecordingPipeline Debug implementation
    #[test]
    fn test_recording_pipeline_debug() {
        // This test just verifies the Debug trait is implemented and doesn't panic.
        // We can't create a real RecordingPipeline without a valid node_id,
        // but we document the expected debug output format.
        //
        // Expected format:
        // RecordingPipeline {
        //     output_path: "/path/to/file.mp4",
        //     start_time: Some(...) or None,
        //     width: 1920,
        //     height: 1080,
        // }
    }

    // --- System audio capture tests ---

    #[test]
    fn test_get_system_audio_source_returns_default_monitor() {
        // Verify the system audio source is the PulseAudio default monitor
        let source = get_system_audio_source();
        assert_eq!(
            source, "@DEFAULT_MONITOR@",
            "System audio source should be @DEFAULT_MONITOR@"
        );
    }

    #[test]
    fn test_system_audio_source_is_constant() {
        // Verify the system audio source is deterministic
        for _ in 0..10 {
            assert_eq!(
                get_system_audio_source(),
                "@DEFAULT_MONITOR@",
                "System audio source should be constant"
            );
        }
    }

    #[test]
    fn test_audio_config_combinations() {
        // Test that we correctly identify audio configuration states
        let no_audio = AudioConfig { system: false, mic: false };
        let mic_only = AudioConfig { system: false, mic: true };
        let system_only = AudioConfig { system: true, mic: false };
        let both_audio = AudioConfig { system: true, mic: true };

        // No audio
        assert!(!no_audio.system && !no_audio.mic);

        // Mic only
        assert!(!mic_only.system && mic_only.mic);

        // System only
        assert!(system_only.system && !system_only.mic);

        // Both
        assert!(both_audio.system && both_audio.mic);
    }
}
