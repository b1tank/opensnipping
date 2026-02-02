# ipc

Boundary: Tauri IPC glue.

Intended contents:
- `commands.rs`: `#[tauri::command]` entrypoints (thin wrappers)
- `emit.rs`: event emission helpers
- `errors.rs`: mapping from backend errors to domain errors/events

Rules:
- Keep domain logic out of command handlers.
- Every command must have explicit failure behavior (error code + message).
