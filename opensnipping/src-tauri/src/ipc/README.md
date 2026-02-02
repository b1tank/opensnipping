# ipc

Boundary: Tauri IPC glue between frontend and backend.

## Files

- `mod.rs` — Module surface + re-exports
- `commands.rs` — `#[tauri::command]` entrypoints (thin wrappers calling domain logic)
- `emit.rs` — Event emission helpers (`emit_state_changed`, `emit_error`, etc.)
- `errors.rs` — Error mapping from backend errors to IPC error responses

## Rules

- Keep domain logic out of command handlers (delegate to `state.rs`, `capture/*`)
- Every command must have explicit failure behavior (error code + message)
- Event emission goes through `emit.rs` helpers, not raw `app_handle.emit()`
