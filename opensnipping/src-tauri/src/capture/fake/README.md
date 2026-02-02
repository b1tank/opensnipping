# capture/fake

Fake capture backend for testing and development without real hardware.

## Files

- `mod.rs` — Module surface + re-exports
- `backend.rs` — `FakeCaptureBackend` implementation
- `tests/` — Test modules:
  - `mod.rs` — Test module wiring
  - `recording.rs` — Recording flow tests
  - `screenshot.rs` — Screenshot flow tests
  - `selection.rs` — Selection/portal mock tests

## Rules

- Keep each module under 500 LOC
- Backend must be deterministic and side-effect-free
- Tests should cover all state machine transitions
