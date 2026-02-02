// Linux capture backend using xdg-desktop-portal and PipeWire
//
// This module integrates with the Freedesktop portal for screen capture
// on Linux (Wayland and X11).

mod backend;
mod encoding;
mod pipeline;

pub use backend::LinuxCaptureBackend;
pub use encoding::{
    detect_available_audio_encoder, detect_available_encoder, get_muxer_for_container,
    get_system_audio_source,
};
pub use pipeline::RecordingPipeline;

#[cfg(test)]
mod tests;
