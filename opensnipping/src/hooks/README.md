# src/hooks

Boundary: React hooks only.

Rules:
- Hooks can depend on `src/tauri` wrappers, but must not embed raw `invoke` strings.
- Prefer small hooks with explicit inputs/outputs; no hidden timers without lifecycle.
