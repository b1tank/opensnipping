use crate::capture::CaptureBackendError;
use crate::state::{CaptureError, ErrorCode};

pub(crate) fn backend_error_to_capture_error(err: &CaptureBackendError) -> CaptureError {
    match err {
        CaptureBackendError::PermissionDenied(msg) => CaptureError {
            code: ErrorCode::PermissionDenied,
            message: msg.clone(),
        },
        CaptureBackendError::PortalError(msg) => CaptureError {
            code: ErrorCode::PortalError,
            message: msg.clone(),
        },
        CaptureBackendError::NoSourceAvailable(msg) => CaptureError {
            code: ErrorCode::PortalError,
            message: msg.clone(),
        },
        CaptureBackendError::NotSupported(msg) => CaptureError {
            code: ErrorCode::Unknown,
            message: msg.clone(),
        },
        CaptureBackendError::Internal(msg) => CaptureError {
            code: ErrorCode::Unknown,
            message: msg.clone(),
        },
    }
}
