# src/tauri

Boundary: Tauri integration lives here.

## Files

- `commands.ts` — Typed wrappers around `invoke(...)` for all backend commands
- `events.ts` — `useCaptureEvents` hook for subscribing to backend events
- `index.ts` — Re-exports everything for convenient imports

## Rules

- Only files in this folder should call `invoke("...")` or subscribe to Tauri events directly.
- UI components should import typed wrappers/hooks from `./tauri` (via index.ts).
- Keep command names/event names aligned with Rust (`src-tauri/src/ipc/*`) and TS contract (`src/types.ts`).
- When adding a new command: add to `commands.ts`, update mocks in `test/setup.ts`.
- When adding a new event: add handler type to `events.ts`, wire through `useCaptureEvents`.
