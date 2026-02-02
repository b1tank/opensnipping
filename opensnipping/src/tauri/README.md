# src/tauri

Boundary: Tauri integration lives here.

Rules:
- Only files in this folder should call `invoke("...")` or subscribe to Tauri events.
- UI components should depend on typed wrappers/hooks exported from here.
- Keep command names/event names aligned with Rust (`src-tauri/src/*`) and TS contract (`src/types.ts`).
