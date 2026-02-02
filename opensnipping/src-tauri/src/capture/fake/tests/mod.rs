use super::*;
use crate::capture::{CaptureBackend, CaptureBackendError, SelectionResult};
use crate::config::{AudioConfig, CaptureConfig, CaptureSource, ContainerFormat};

pub(super) fn test_config() -> CaptureConfig {
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

mod recording;
mod screenshot;
mod selection;
