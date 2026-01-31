use serde::{Deserialize, Serialize};

/// Recording states
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum CaptureState {
    /// No capture in progress
    #[default]
    Idle,
    /// User is selecting capture source
    Selecting,
    /// Recording in progress
    Recording,
    /// Recording paused
    Paused,
    /// Finalizing output file
    Finalizing,
    /// Error state
    Error,
}

/// Error codes for capture errors
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ErrorCode {
    PermissionDenied,
    PortalError,
    EncoderUnavailable,
    PipelineError,
    IoError,
    InvalidConfig,
    Unknown,
}

/// Error details
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CaptureError {
    pub code: ErrorCode,
    pub message: String,
}

/// State transition error
#[derive(Debug, Clone, PartialEq)]
pub struct TransitionError {
    pub from: CaptureState,
    pub to: CaptureState,
    pub message: String,
}

impl std::fmt::Display for TransitionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Invalid transition from {:?} to {:?}: {}",
            self.from, self.to, self.message
        )
    }
}

impl std::error::Error for TransitionError {}

/// State machine for capture orchestration
#[derive(Debug)]
pub struct StateMachine {
    state: CaptureState,
    last_error: Option<CaptureError>,
}

impl Default for StateMachine {
    fn default() -> Self {
        Self::new()
    }
}

impl StateMachine {
    pub fn new() -> Self {
        Self {
            state: CaptureState::Idle,
            last_error: None,
        }
    }

    pub fn state(&self) -> CaptureState {
        self.state
    }

    pub fn last_error(&self) -> Option<&CaptureError> {
        self.last_error.as_ref()
    }

    /// Validate and perform state transition
    fn transition(&mut self, to: CaptureState) -> Result<CaptureState, TransitionError> {
        let from = self.state;
        
        let valid = match (from, to) {
            // From Idle
            (CaptureState::Idle, CaptureState::Selecting) => true,
            (CaptureState::Idle, CaptureState::Error) => true,
            
            // From Selecting
            (CaptureState::Selecting, CaptureState::Recording) => true,
            (CaptureState::Selecting, CaptureState::Idle) => true, // cancelled
            (CaptureState::Selecting, CaptureState::Error) => true,
            
            // From Recording
            (CaptureState::Recording, CaptureState::Paused) => true,
            (CaptureState::Recording, CaptureState::Finalizing) => true,
            (CaptureState::Recording, CaptureState::Error) => true,
            
            // From Paused
            (CaptureState::Paused, CaptureState::Recording) => true,
            (CaptureState::Paused, CaptureState::Finalizing) => true,
            (CaptureState::Paused, CaptureState::Error) => true,
            
            // From Finalizing
            (CaptureState::Finalizing, CaptureState::Idle) => true,
            (CaptureState::Finalizing, CaptureState::Error) => true,
            
            // From Error
            (CaptureState::Error, CaptureState::Idle) => true, // reset
            
            // Same state is always valid (no-op)
            (a, b) if a == b => true,
            
            _ => false,
        };

        if valid {
            self.state = to;
            if to != CaptureState::Error {
                self.last_error = None;
            }
            Ok(to)
        } else {
            Err(TransitionError {
                from,
                to,
                message: format!("Cannot transition from {:?} to {:?}", from, to),
            })
        }
    }

    /// Start capture (Idle → Selecting)
    pub fn start_selecting(&mut self) -> Result<CaptureState, TransitionError> {
        self.transition(CaptureState::Selecting)
    }

    /// Cancel selection (Selecting → Idle)
    pub fn cancel_selection(&mut self) -> Result<CaptureState, TransitionError> {
        self.transition(CaptureState::Idle)
    }

    /// Begin recording (Selecting → Recording)
    pub fn begin_recording(&mut self) -> Result<CaptureState, TransitionError> {
        self.transition(CaptureState::Recording)
    }

    /// Pause recording (Recording → Paused)
    pub fn pause(&mut self) -> Result<CaptureState, TransitionError> {
        self.transition(CaptureState::Paused)
    }

    /// Resume recording (Paused → Recording)
    pub fn resume(&mut self) -> Result<CaptureState, TransitionError> {
        self.transition(CaptureState::Recording)
    }

    /// Stop recording (Recording/Paused → Finalizing)
    pub fn stop(&mut self) -> Result<CaptureState, TransitionError> {
        self.transition(CaptureState::Finalizing)
    }

    /// Finalize complete (Finalizing → Idle)
    pub fn finalize_complete(&mut self) -> Result<CaptureState, TransitionError> {
        self.transition(CaptureState::Idle)
    }

    /// Set error state
    pub fn set_error(&mut self, error: CaptureError) -> CaptureState {
        self.last_error = Some(error);
        self.state = CaptureState::Error;
        CaptureState::Error
    }

    /// Reset from error (Error → Idle)
    pub fn reset(&mut self) -> Result<CaptureState, TransitionError> {
        self.transition(CaptureState::Idle)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_initial_state_is_idle() {
        let sm = StateMachine::new();
        assert_eq!(sm.state(), CaptureState::Idle);
    }

    #[test]
    fn test_valid_full_recording_flow() {
        let mut sm = StateMachine::new();
        
        // Idle → Selecting
        assert!(sm.start_selecting().is_ok());
        assert_eq!(sm.state(), CaptureState::Selecting);
        
        // Selecting → Recording
        assert!(sm.begin_recording().is_ok());
        assert_eq!(sm.state(), CaptureState::Recording);
        
        // Recording → Paused
        assert!(sm.pause().is_ok());
        assert_eq!(sm.state(), CaptureState::Paused);
        
        // Paused → Recording
        assert!(sm.resume().is_ok());
        assert_eq!(sm.state(), CaptureState::Recording);
        
        // Recording → Finalizing
        assert!(sm.stop().is_ok());
        assert_eq!(sm.state(), CaptureState::Finalizing);
        
        // Finalizing → Idle
        assert!(sm.finalize_complete().is_ok());
        assert_eq!(sm.state(), CaptureState::Idle);
    }

    #[test]
    fn test_cancel_selection() {
        let mut sm = StateMachine::new();
        sm.start_selecting().unwrap();
        assert!(sm.cancel_selection().is_ok());
        assert_eq!(sm.state(), CaptureState::Idle);
    }

    #[test]
    fn test_invalid_transition_idle_to_recording() {
        let mut sm = StateMachine::new();
        let result = sm.begin_recording();
        assert!(result.is_err());
        assert_eq!(sm.state(), CaptureState::Idle);
    }

    #[test]
    fn test_invalid_transition_idle_to_paused() {
        let mut sm = StateMachine::new();
        let result = sm.pause();
        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_transition_selecting_to_paused() {
        let mut sm = StateMachine::new();
        sm.start_selecting().unwrap();
        let result = sm.pause();
        assert!(result.is_err());
    }

    #[test]
    fn test_error_state_and_reset() {
        let mut sm = StateMachine::new();
        sm.start_selecting().unwrap();
        
        sm.set_error(CaptureError {
            code: ErrorCode::PortalError,
            message: "Portal denied access".to_string(),
        });
        
        assert_eq!(sm.state(), CaptureState::Error);
        assert!(sm.last_error().is_some());
        assert_eq!(sm.last_error().unwrap().code, ErrorCode::PortalError);
        
        // Reset from error
        assert!(sm.reset().is_ok());
        assert_eq!(sm.state(), CaptureState::Idle);
    }

    #[test]
    fn test_stop_from_paused() {
        let mut sm = StateMachine::new();
        sm.start_selecting().unwrap();
        sm.begin_recording().unwrap();
        sm.pause().unwrap();
        
        assert!(sm.stop().is_ok());
        assert_eq!(sm.state(), CaptureState::Finalizing);
    }

    #[test]
    fn test_same_state_transition_is_noop() {
        let mut sm = StateMachine::new();
        sm.start_selecting().unwrap();
        sm.begin_recording().unwrap();
        
        // Calling begin_recording again should be a no-op
        let result = sm.transition(CaptureState::Recording);
        assert!(result.is_ok());
        assert_eq!(sm.state(), CaptureState::Recording);
    }
}
