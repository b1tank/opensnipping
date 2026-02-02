use super::*;

// Recording tests

#[tokio::test]
async fn test_fake_backend_start_recording_succeeds() {
    let backend = FakeCaptureBackend::succeeding();
    let config = test_config();
    let selection = SelectionResult {
        node_id: 42,
        stream_fd: None,
        width: Some(1920),
        height: Some(1080),
    };

    let result = backend.start_recording(&selection, &config).await;
    assert!(result.is_ok());
    assert!(backend.is_recording());
    assert_eq!(backend.start_recording_count(), 1);
}

#[tokio::test]
async fn test_fake_backend_start_recording_fails_when_configured() {
    let backend = FakeCaptureBackend::permission_denied();
    let config = test_config();
    let selection = SelectionResult {
        node_id: 42,
        stream_fd: None,
        width: Some(1920),
        height: Some(1080),
    };

    let result = backend.start_recording(&selection, &config).await;
    assert!(result.is_err());
    assert!(matches!(
        result.unwrap_err(),
        CaptureBackendError::PermissionDenied(_)
    ));
    assert!(!backend.is_recording());
}

#[tokio::test]
async fn test_fake_backend_start_recording_fails_if_already_recording() {
    let backend = FakeCaptureBackend::succeeding();
    let config = test_config();
    let selection = SelectionResult {
        node_id: 42,
        stream_fd: None,
        width: Some(1920),
        height: Some(1080),
    };

    // First start succeeds
    backend.start_recording(&selection, &config).await.unwrap();

    // Second start fails
    let result = backend.start_recording(&selection, &config).await;
    assert!(result.is_err());
    assert!(matches!(
        result.unwrap_err(),
        CaptureBackendError::Internal(_)
    ));
    assert_eq!(backend.start_recording_count(), 2); // Both calls counted
}

#[tokio::test]
async fn test_fake_backend_stop_recording_succeeds() {
    let backend = FakeCaptureBackend::succeeding();
    let config = test_config();
    let selection = SelectionResult {
        node_id: 42,
        stream_fd: None,
        width: Some(1920),
        height: Some(1080),
    };

    // Start recording first
    backend.start_recording(&selection, &config).await.unwrap();
    assert!(backend.is_recording());

    // Stop recording
    let result = backend.stop_recording().await;
    assert!(result.is_ok());

    let recording = result.unwrap();
    assert_eq!(recording.path, config.output_path);
    assert_eq!(recording.width, 1920);
    assert_eq!(recording.height, 1080);
    assert!(!backend.is_recording());
    assert_eq!(backend.stop_recording_count(), 1);
}

#[tokio::test]
async fn test_fake_backend_stop_recording_fails_if_not_recording() {
    let backend = FakeCaptureBackend::succeeding();

    let result = backend.stop_recording().await;
    assert!(result.is_err());
    assert!(matches!(
        result.unwrap_err(),
        CaptureBackendError::Internal(_)
    ));
}

#[tokio::test]
async fn test_fake_backend_full_recording_flow() {
    let backend = FakeCaptureBackend::succeeding();
    let config = test_config();

    // Step 1: Selection
    let selection = backend.request_selection(&config).await.unwrap();
    assert_eq!(backend.selection_count(), 1);

    // Step 2: Start recording
    backend.start_recording(&selection, &config).await.unwrap();
    assert!(backend.is_recording());

    // Step 3: Stop recording
    let result = backend.stop_recording().await.unwrap();
    assert!(!backend.is_recording());
    assert_eq!(result.path, config.output_path);

    // Verify counts
    assert_eq!(backend.start_recording_count(), 1);
    assert_eq!(backend.stop_recording_count(), 1);
}

// Pause/Resume tests

#[tokio::test]
async fn test_fake_backend_pause_recording_succeeds() {
    let backend = FakeCaptureBackend::succeeding();
    let config = test_config();
    let selection = SelectionResult {
        node_id: 42,
        stream_fd: None,
        width: Some(1920),
        height: Some(1080),
    };

    // Start recording first
    backend.start_recording(&selection, &config).await.unwrap();
    assert!(backend.is_recording());
    assert!(!backend.is_paused());

    // Pause recording
    let result = backend.pause_recording().await;
    assert!(result.is_ok());
    assert!(backend.is_recording());
    assert!(backend.is_paused());
    assert_eq!(backend.pause_recording_count(), 1);
}

#[tokio::test]
async fn test_fake_backend_pause_recording_fails_if_not_recording() {
    let backend = FakeCaptureBackend::succeeding();

    let result = backend.pause_recording().await;
    assert!(result.is_err());
    assert!(matches!(
        result.unwrap_err(),
        CaptureBackendError::Internal(_)
    ));
}

#[tokio::test]
async fn test_fake_backend_pause_recording_fails_if_already_paused() {
    let backend = FakeCaptureBackend::succeeding();
    let config = test_config();
    let selection = SelectionResult {
        node_id: 42,
        stream_fd: None,
        width: Some(1920),
        height: Some(1080),
    };

    // Start and pause
    backend.start_recording(&selection, &config).await.unwrap();
    backend.pause_recording().await.unwrap();

    // Second pause fails
    let result = backend.pause_recording().await;
    assert!(result.is_err());
    assert!(matches!(
        result.unwrap_err(),
        CaptureBackendError::Internal(_)
    ));
}

#[tokio::test]
async fn test_fake_backend_resume_recording_succeeds() {
    let backend = FakeCaptureBackend::succeeding();
    let config = test_config();
    let selection = SelectionResult {
        node_id: 42,
        stream_fd: None,
        width: Some(1920),
        height: Some(1080),
    };

    // Start, pause, then resume
    backend.start_recording(&selection, &config).await.unwrap();
    backend.pause_recording().await.unwrap();
    assert!(backend.is_paused());

    let result = backend.resume_recording().await;
    assert!(result.is_ok());
    assert!(backend.is_recording());
    assert!(!backend.is_paused());
    assert_eq!(backend.resume_recording_count(), 1);
}

#[tokio::test]
async fn test_fake_backend_resume_recording_fails_if_not_recording() {
    let backend = FakeCaptureBackend::succeeding();

    let result = backend.resume_recording().await;
    assert!(result.is_err());
    assert!(matches!(
        result.unwrap_err(),
        CaptureBackendError::Internal(_)
    ));
}

#[tokio::test]
async fn test_fake_backend_resume_recording_fails_if_not_paused() {
    let backend = FakeCaptureBackend::succeeding();
    let config = test_config();
    let selection = SelectionResult {
        node_id: 42,
        stream_fd: None,
        width: Some(1920),
        height: Some(1080),
    };

    // Start recording but don't pause
    backend.start_recording(&selection, &config).await.unwrap();

    let result = backend.resume_recording().await;
    assert!(result.is_err());
    assert!(matches!(
        result.unwrap_err(),
        CaptureBackendError::Internal(_)
    ));
}

#[tokio::test]
async fn test_fake_backend_full_recording_with_pause_flow() {
    let backend = FakeCaptureBackend::succeeding();
    let config = test_config();

    // Step 1: Selection
    let selection = backend.request_selection(&config).await.unwrap();

    // Step 2: Start recording
    backend.start_recording(&selection, &config).await.unwrap();
    assert!(backend.is_recording());
    assert!(!backend.is_paused());

    // Step 3: Pause
    backend.pause_recording().await.unwrap();
    assert!(backend.is_recording());
    assert!(backend.is_paused());

    // Step 4: Resume
    backend.resume_recording().await.unwrap();
    assert!(backend.is_recording());
    assert!(!backend.is_paused());

    // Step 5: Stop
    let result = backend.stop_recording().await.unwrap();
    assert!(!backend.is_recording());
    assert!(!backend.is_paused());

    // Verify counts
    assert_eq!(backend.start_recording_count(), 1);
    assert_eq!(backend.pause_recording_count(), 1);
    assert_eq!(backend.resume_recording_count(), 1);
    assert_eq!(backend.stop_recording_count(), 1);
    assert_eq!(result.path, config.output_path);
}
