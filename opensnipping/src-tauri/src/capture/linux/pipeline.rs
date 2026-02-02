use crate::capture::{CaptureBackendError, RecordingResult};
use crate::config::{AudioConfig, ContainerFormat};
use gstreamer::prelude::*;
use tracing::{debug, error, info, warn};

use super::{
    detect_available_audio_encoder, detect_available_encoder, get_muxer_for_container,
    get_system_audio_source,
};

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

            // Build audio pipeline based on configuration
            if has_mic && has_system {
                // Both mic and system audio: use audiomixer to combine both sources
                info!(
                    "Recording with mic + system audio (mixed), encoder: {}",
                    audio_encoder
                );
                format!(
                    "pipewiresrc path={node_id} ! \
                     videoconvert ! \
                     videoscale ! \
                     video/x-raw,framerate={fps}/1 ! \
                     {video_encoder} ! mux. \
                     audiomixer name=mix ! \
                     audioconvert ! \
                     audioresample ! \
                     {audio_encoder} ! mux. \
                     pulsesrc ! audioconvert ! audioresample ! mix. \
                     pulsesrc device={system_device} ! audioconvert ! audioresample ! mix. \
                     {muxer} name=mux ! \
                     filesink location={output_path}",
                    node_id = node_id,
                    fps = fps,
                    video_encoder = video_encoder,
                    audio_encoder = audio_encoder,
                    system_device = get_system_audio_source(),
                    muxer = muxer,
                    output_path = output_path.display()
                )
            } else if has_system {
                // System audio only
                info!("Recording with system audio, encoder: {}", audio_encoder);
                format!(
                    "pipewiresrc path={node_id} ! \
                     videoconvert ! \
                     videoscale ! \
                     video/x-raw,framerate={fps}/1 ! \
                     {video_encoder} ! mux. \
                     pulsesrc device={system_device} ! \
                     audioconvert ! \
                     audioresample ! \
                     {audio_encoder} ! mux. \
                     {muxer} name=mux ! \
                     filesink location={output_path}",
                    node_id = node_id,
                    fps = fps,
                    video_encoder = video_encoder,
                    system_device = get_system_audio_source(),
                    audio_encoder = audio_encoder,
                    muxer = muxer,
                    output_path = output_path.display()
                )
            } else {
                // Mic only
                info!(
                    "Recording with microphone audio, encoder: {}",
                    audio_encoder
                );
                format!(
                    "pipewiresrc path={node_id} ! \
                     videoconvert ! \
                     videoscale ! \
                     video/x-raw,framerate={fps}/1 ! \
                     {video_encoder} ! mux. \
                     pulsesrc ! \
                     audioconvert ! \
                     audioresample ! \
                     {audio_encoder} ! mux. \
                     {muxer} name=mux ! \
                     filesink location={output_path}",
                    node_id = node_id,
                    fps = fps,
                    video_encoder = video_encoder,
                    audio_encoder = audio_encoder,
                    muxer = muxer,
                    output_path = output_path.display()
                )
            }
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

        let pipeline = gstreamer::parse::launch(&pipeline_str).map_err(|e| {
            CaptureBackendError::Internal(format!("Failed to create pipeline: {}", e))
        })?;

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

        self.pipeline
            .set_state(gstreamer::State::Playing)
            .map_err(|e| {
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

        self.pipeline
            .set_state(gstreamer::State::Paused)
            .map_err(|e| {
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

        self.pipeline
            .set_state(gstreamer::State::Playing)
            .map_err(|e| {
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
                            let debug_info = err
                                .debug()
                                .map(|d| format!(" ({:?})", d))
                                .unwrap_or_default();
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
