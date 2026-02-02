use serde::{Deserialize, Serialize};

/// Source type for capture
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum CaptureSource {
    #[default]
    Screen,
    Monitor,
    Window,
    Region,
}

/// Container format for recordings
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum ContainerFormat {
    #[default]
    Mp4,
    Mkv,
}

/// Audio configuration
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct AudioConfig {
    /// Capture system audio
    pub system: bool,
    /// Capture microphone
    pub mic: bool,
}

/// Configuration for a capture session
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CaptureConfig {
    /// Type of capture source
    pub source: CaptureSource,
    /// Frames per second (1-60)
    pub fps: u8,
    /// Include cursor in capture
    pub include_cursor: bool,
    /// Audio settings
    pub audio: AudioConfig,
    /// Output container format
    pub container: ContainerFormat,
    /// Output file path
    pub output_path: String,
}

impl Default for CaptureConfig {
    fn default() -> Self {
        Self {
            source: CaptureSource::default(),
            fps: 30,
            include_cursor: true,
            audio: AudioConfig::default(),
            container: ContainerFormat::default(),
            output_path: String::new(),
        }
    }
}

/// Validation error for CaptureConfig
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ConfigError {
    pub field: String,
    pub message: String,
}

impl CaptureConfig {
    /// Validate the configuration
    pub fn validate(&self) -> Result<(), ConfigError> {
        if self.fps == 0 || self.fps > 60 {
            return Err(ConfigError {
                field: "fps".to_string(),
                message: "FPS must be between 1 and 60".to_string(),
            });
        }

        if self.output_path.is_empty() {
            return Err(ConfigError {
                field: "output_path".to_string(),
                message: "Output path cannot be empty".to_string(),
            });
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = CaptureConfig::default();
        assert_eq!(config.fps, 30);
        assert!(config.include_cursor);
        assert_eq!(config.source, CaptureSource::Screen);
        assert_eq!(config.container, ContainerFormat::Mp4);
    }

    #[test]
    fn test_validate_valid_config() {
        let config = CaptureConfig {
            output_path: "/tmp/recording.mp4".to_string(),
            ..Default::default()
        };
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_validate_fps_zero() {
        let config = CaptureConfig {
            fps: 0,
            output_path: "/tmp/recording.mp4".to_string(),
            ..Default::default()
        };
        let err = config.validate().unwrap_err();
        assert_eq!(err.field, "fps");
    }

    #[test]
    fn test_validate_fps_too_high() {
        let config = CaptureConfig {
            fps: 61,
            output_path: "/tmp/recording.mp4".to_string(),
            ..Default::default()
        };
        let err = config.validate().unwrap_err();
        assert_eq!(err.field, "fps");
    }

    #[test]
    fn test_validate_empty_output_path() {
        let config = CaptureConfig::default();
        let err = config.validate().unwrap_err();
        assert_eq!(err.field, "output_path");
    }

    #[test]
    fn test_serialize_deserialize() {
        let config = CaptureConfig {
            source: CaptureSource::Window,
            fps: 60,
            include_cursor: false,
            audio: AudioConfig {
                system: true,
                mic: true,
            },
            container: ContainerFormat::Mkv,
            output_path: "/tmp/test.mkv".to_string(),
        };

        let json = serde_json::to_string(&config).unwrap();
        let deserialized: CaptureConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(config, deserialized);
    }

    #[test]
    fn test_audio_config_combinations() {
        // Test that AudioConfig correctly represents all audio states
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
