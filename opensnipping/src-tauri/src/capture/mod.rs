// Capture backend abstraction
//
// This module defines the contract for capture backends and provides
// OS-specific implementations.

#[cfg(target_os = "linux")]
pub mod linux;

#[cfg(test)]
pub mod fake;

use crate::config::CaptureConfig;
use serde::{Deserialize, Serialize};
use std::fmt;

/// Result of a successful screen/window/region selection from portal
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SelectionResult {
    /// PipeWire node ID for the selected source
    pub node_id: u32,
    /// Stream descriptor (path or identifier)
    pub stream_fd: Option<i32>,
    /// Width of the selected source
    pub width: Option<u32>,
    /// Height of the selected source
    pub height: Option<u32>,
}

/// Errors that can occur during capture operations
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum CaptureBackendError {
    /// User denied permission or cancelled selection
    PermissionDenied(String),
    /// Portal communication error
    PortalError(String),
    /// No suitable capture source available
    NoSourceAvailable(String),
    /// Backend not available on this platform
    NotSupported(String),
    /// Internal error
    Internal(String),
}

impl fmt::Display for CaptureBackendError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::PermissionDenied(msg) => write!(f, "Permission denied: {}", msg),
            Self::PortalError(msg) => write!(f, "Portal error: {}", msg),
            Self::NoSourceAvailable(msg) => write!(f, "No source available: {}", msg),
            Self::NotSupported(msg) => write!(f, "Not supported: {}", msg),
            Self::Internal(msg) => write!(f, "Internal error: {}", msg),
        }
    }
}

impl std::error::Error for CaptureBackendError {}

/// Trait for capture backends
///
/// Each OS implements this trait to provide screen/window/region selection
/// and capture functionality.
pub trait CaptureBackend: Send + Sync {
    /// Request screen/window/region selection from the user
    ///
    /// On Linux, this opens the xdg-desktop-portal picker dialog.
    /// Returns a SelectionResult with the PipeWire node ID on success.
    fn request_selection(
        &self,
        config: &CaptureConfig,
    ) -> impl std::future::Future<Output = Result<SelectionResult, CaptureBackendError>> + Send;

    /// Cancel an ongoing selection (if supported)
    fn cancel_selection(&self) -> impl std::future::Future<Output = Result<(), CaptureBackendError>> + Send;
}

/// Get the appropriate capture backend for the current platform
#[cfg(target_os = "linux")]
pub fn get_backend() -> impl CaptureBackend {
    linux::LinuxCaptureBackend::new()
}

/// Stub backend for unsupported platforms
#[cfg(not(target_os = "linux"))]
pub fn get_backend() -> impl CaptureBackend {
    StubBackend
}

/// Stub backend for unsupported platforms
#[cfg(not(target_os = "linux"))]
#[derive(Debug, Default)]
pub struct StubBackend;

#[cfg(not(target_os = "linux"))]
impl CaptureBackend for StubBackend {
    async fn request_selection(
        &self,
        _config: &CaptureConfig,
    ) -> Result<SelectionResult, CaptureBackendError> {
        Err(CaptureBackendError::NotSupported(
            "Capture not implemented for this platform".to_string(),
        ))
    }

    async fn cancel_selection(&self) -> Result<(), CaptureBackendError> {
        Ok(())
    }
}
