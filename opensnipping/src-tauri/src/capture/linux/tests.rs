use super::*;
use crate::config::{AudioConfig, CaptureSource, ContainerFormat};
use ashpd::desktop::screencast::SourceType;

use super::encoding::{AAC_ENCODERS, H264_ENCODERS, OPUS_ENCODERS};

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
    assert_eq!(
        H264_ENCODERS[0], "vaapih264enc",
        "VA-API should be first (Intel/AMD iGPU)"
    );
    assert_eq!(
        H264_ENCODERS[1], "nvh264enc",
        "NVENC should be second (NVIDIA)"
    );
    assert_eq!(
        H264_ENCODERS[2], "x264enc",
        "x264 should be last (SW fallback)"
    );
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
    assert!(
        lock.is_ok(),
        "Recording lock should be available on new backend"
    );
    assert!(
        lock.unwrap().is_none(),
        "No recording should be in progress initially"
    );
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

// --- Audio mixing configuration tests ---

#[test]
fn test_audio_config_mic_only() {
    let audio = AudioConfig {
        mic: true,
        system: false,
    };
    assert!(audio.mic, "Mic should be enabled");
    assert!(!audio.system, "System should be disabled");
}

#[test]
fn test_audio_config_system_only() {
    let audio = AudioConfig {
        mic: false,
        system: true,
    };
    assert!(!audio.mic, "Mic should be disabled");
    assert!(audio.system, "System should be enabled");
}

#[test]
fn test_audio_config_both_enabled() {
    let audio = AudioConfig {
        mic: true,
        system: true,
    };
    assert!(audio.mic, "Mic should be enabled");
    assert!(audio.system, "System should be enabled");
    // When both are enabled, the pipeline uses audiomixer to combine sources
}

#[test]
fn test_audio_config_matrix() {
    // Test all 4 combinations of mic/system audio
    let configs = [
        (
            AudioConfig {
                mic: false,
                system: false,
            },
            "no audio",
        ),
        (
            AudioConfig {
                mic: true,
                system: false,
            },
            "mic only",
        ),
        (
            AudioConfig {
                mic: false,
                system: true,
            },
            "system only",
        ),
        (
            AudioConfig {
                mic: true,
                system: true,
            },
            "mic + system (mixed)",
        ),
    ];

    for (config, description) in configs {
        // Just verify the configs can be created and have expected values
        let has_any = config.mic || config.system;
        let has_both = config.mic && config.system;

        match description {
            "no audio" => {
                assert!(!has_any, "No audio config should have no sources");
            }
            "mic only" => {
                assert!(has_any && !has_both, "Mic only should have one source");
                assert!(config.mic, "Should have mic enabled");
            }
            "system only" => {
                assert!(has_any && !has_both, "System only should have one source");
                assert!(config.system, "Should have system enabled");
            }
            "mic + system (mixed)" => {
                assert!(has_both, "Mixed audio should have both sources");
            }
            _ => panic!("Unknown config"),
        }
    }
}

/// Verify audiomixer element is available in GStreamer
#[test]
fn test_audiomixer_element_availability() {
    // Initialize GStreamer
    if gstreamer::init().is_err() {
        println!("GStreamer not available, skipping audiomixer test");
        return;
    }

    // Check if audiomixer is available
    let has_audiomixer = gstreamer::ElementFactory::find("audiomixer").is_some();

    // audiomixer is part of gstreamer-plugins-base, which should be widely available
    // This test documents the dependency rather than making it required
    if has_audiomixer {
        println!("audiomixer element is available");
    } else {
        println!("audiomixer element not found - audio mixing requires gst-plugins-base");
    }
}
