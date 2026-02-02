# domain

Boundary: pure domain logic.

Intended contents:
- State machine
- Config validation
- Domain errors

Rules:
- No Tauri types here (no `AppHandle`, no event emission).
- Keep this unit-testable without a running app.
