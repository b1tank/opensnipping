// Fake capture backend for testing
//
// This module provides a mock implementation of CaptureBackend for use in tests
// without requiring actual portal/PipeWire integration.

mod backend;

pub use backend::{FakeCaptureBackend, FakeError};

#[cfg(test)]
mod tests;
