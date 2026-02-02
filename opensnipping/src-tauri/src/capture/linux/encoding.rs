// Linux capture backend using xdg-desktop-portal and PipeWire
//
// This module integrates with the Freedesktop portal for screen capture
// on Linux (Wayland and X11).

use crate::config::ContainerFormat;
use tracing::{debug, warn};

/// H.264 encoders in order of preference (hardware first, then software fallback)
pub(super) const H264_ENCODERS: &[&str] = &[
    "vaapih264enc", // Intel/AMD iGPU via VA-API
    "nvh264enc",    // NVIDIA via NVENC
    "x264enc",      // Software fallback (libx264)
];

/// AAC audio encoders in order of preference
pub(super) const AAC_ENCODERS: &[&str] = &[
    "fdkaacenc", // FDK AAC (best quality, may need licensing)
    "voaacenc",  // VO-AAC (LGPL, good quality)
    "avenc_aac", // libavcodec AAC (fallback)
];

/// Opus audio encoders (for MKV)
pub(super) const OPUS_ENCODERS: &[&str] = &[
    "opusenc", // Standard Opus encoder
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
pub const fn get_system_audio_source() -> &'static str {
    // @DEFAULT_MONITOR@ is a special PulseAudio device name that resolves
    // to the monitor source of the current default output device.
    // This works with both PulseAudio and PipeWire (via pipewire-pulse).
    "@DEFAULT_MONITOR@"
}
