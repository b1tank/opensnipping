use super::*;

#[tokio::test]
async fn test_fake_backend_screenshot_creates_file() {
    let backend = FakeCaptureBackend::succeeding();
    let selection = SelectionResult {
        node_id: 42,
        stream_fd: None,
        width: Some(64),
        height: Some(48),
    };

    let temp_dir = std::env::temp_dir();
    let output_path = temp_dir.join(format!("test_screenshot_{}.png", uuid::Uuid::new_v4()));

    let result = backend.capture_screenshot(&selection, &output_path).await;
    assert!(result.is_ok());

    let screenshot = result.unwrap();
    assert_eq!(screenshot.width, 64);
    assert_eq!(screenshot.height, 48);
    assert!(std::path::Path::new(&screenshot.path).exists());

    // Cleanup
    let _ = std::fs::remove_file(&output_path);
}

#[tokio::test]
async fn test_fake_backend_screenshot_uses_default_dimensions() {
    let backend = FakeCaptureBackend::succeeding();
    let selection = SelectionResult {
        node_id: 42,
        stream_fd: None,
        width: None,
        height: None,
    };

    let temp_dir = std::env::temp_dir();
    let output_path = temp_dir.join(format!("test_screenshot_{}.png", uuid::Uuid::new_v4()));

    let result = backend.capture_screenshot(&selection, &output_path).await;
    assert!(result.is_ok());

    let screenshot = result.unwrap();
    assert_eq!(screenshot.width, 100); // default
    assert_eq!(screenshot.height, 100); // default

    // Cleanup
    let _ = std::fs::remove_file(&output_path);
}

#[tokio::test]
async fn test_fake_backend_screenshot_fails_when_configured() {
    let backend = FakeCaptureBackend::permission_denied();
    let selection = SelectionResult {
        node_id: 42,
        stream_fd: None,
        width: Some(100),
        height: Some(100),
    };

    let temp_dir = std::env::temp_dir();
    let output_path = temp_dir.join(format!("test_screenshot_{}.png", uuid::Uuid::new_v4()));

    let result = backend.capture_screenshot(&selection, &output_path).await;
    assert!(result.is_err());
    assert!(matches!(
        result.unwrap_err(),
        CaptureBackendError::PermissionDenied(_)
    ));

    // File should not exist
    assert!(!output_path.exists());
}

/// Test that ScreenshotResult has all fields required for ScreenshotCompleteEvent emission
#[tokio::test]
async fn test_screenshot_result_has_all_event_fields() {
    use crate::events::ScreenshotCompleteEvent;

    let backend = FakeCaptureBackend::succeeding();
    let selection = SelectionResult {
        node_id: 42,
        stream_fd: None,
        width: Some(800),
        height: Some(600),
    };

    let temp_dir = std::env::temp_dir();
    let output_path = temp_dir.join(format!("test_screenshot_{}.png", uuid::Uuid::new_v4()));

    let result = backend
        .capture_screenshot(&selection, &output_path)
        .await
        .unwrap();

    // Verify we can construct a ScreenshotCompleteEvent from the result
    let event = ScreenshotCompleteEvent {
        path: result.path.clone(),
        width: result.width,
        height: result.height,
    };

    // Verify event has expected values
    assert!(!event.path.is_empty(), "Event path should not be empty");
    assert!(
        event.path.ends_with(".png"),
        "Event path should end with .png"
    );
    assert_eq!(event.width, 800);
    assert_eq!(event.height, 600);

    // Cleanup
    let _ = std::fs::remove_file(&output_path);
}

/// Test the full selection → screenshot flow (mirrors take_screenshot command logic)
#[tokio::test]
async fn test_full_screenshot_flow_selection_to_capture() {
    let backend = FakeCaptureBackend::succeeding();
    backend.set_node_id(99);
    let config = test_config();

    // Step 1: Request selection (like portal picker)
    let selection = backend.request_selection(&config).await.unwrap();
    assert_eq!(selection.node_id, 99);
    assert_eq!(backend.selection_count(), 1);

    // Step 2: Capture screenshot using selection result
    let temp_dir = std::env::temp_dir();
    let output_path = temp_dir.join(format!("test_screenshot_{}.png", uuid::Uuid::new_v4()));

    let screenshot = backend
        .capture_screenshot(&selection, &output_path)
        .await
        .unwrap();

    // Verify screenshot uses selection dimensions
    assert_eq!(screenshot.width, selection.width.unwrap());
    assert_eq!(screenshot.height, selection.height.unwrap());
    assert!(std::path::Path::new(&screenshot.path).exists());

    // Cleanup
    let _ = std::fs::remove_file(&output_path);
}

/// Test that screenshot failure doesn't affect subsequent selection requests
#[tokio::test]
async fn test_screenshot_failure_is_isolated() {
    let backend = FakeCaptureBackend::new();
    let config = test_config();

    // First selection succeeds
    let selection1 = backend.request_selection(&config).await.unwrap();

    // Configure to fail
    backend.set_should_succeed(false);

    // Screenshot fails
    let temp_dir = std::env::temp_dir();
    let output_path = temp_dir.join(format!("test_screenshot_{}.png", uuid::Uuid::new_v4()));
    let screenshot_result = backend.capture_screenshot(&selection1, &output_path).await;
    assert!(screenshot_result.is_err());

    // Configure to succeed again
    backend.set_should_succeed(true);

    // New selection should succeed
    let selection2 = backend.request_selection(&config).await.unwrap();
    assert_eq!(selection2.node_id, 42);
    assert_eq!(backend.selection_count(), 2);
}
