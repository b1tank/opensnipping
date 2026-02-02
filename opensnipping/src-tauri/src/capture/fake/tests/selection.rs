use super::*;

#[tokio::test]
async fn test_fake_backend_succeeds() {
    let backend = FakeCaptureBackend::succeeding();
    let config = test_config();

    let result = backend.request_selection(&config).await;
    assert!(result.is_ok());

    let selection = result.unwrap();
    assert_eq!(selection.node_id, 42);
    assert_eq!(backend.selection_count(), 1);
}

#[tokio::test]
async fn test_fake_backend_permission_denied() {
    let backend = FakeCaptureBackend::permission_denied();
    let config = test_config();

    let result = backend.request_selection(&config).await;
    assert!(result.is_err());
    assert!(matches!(
        result.unwrap_err(),
        CaptureBackendError::PermissionDenied(_)
    ));
}

#[tokio::test]
async fn test_fake_backend_portal_error() {
    let backend = FakeCaptureBackend::portal_error();
    let config = test_config();

    let result = backend.request_selection(&config).await;
    assert!(result.is_err());
    assert!(matches!(
        result.unwrap_err(),
        CaptureBackendError::PortalError(_)
    ));
}

#[tokio::test]
async fn test_fake_backend_cancel() {
    let backend = FakeCaptureBackend::new();

    let result = backend.cancel_selection().await;
    assert!(result.is_ok());
    assert_eq!(backend.cancel_count(), 1);
}

#[tokio::test]
async fn test_fake_backend_custom_node_id() {
    let backend = FakeCaptureBackend::succeeding();
    backend.set_node_id(123);

    let config = test_config();
    let result = backend.request_selection(&config).await.unwrap();
    assert_eq!(result.node_id, 123);
}
