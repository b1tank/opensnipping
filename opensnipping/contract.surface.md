# Contract Surface (TS ⇄ Rust)

Purpose: make it easy to review and keep the cross-layer contract stable.

## Sources of truth

Event names:
- Rust: `src-tauri/src/events.rs` (`event_names::*`)
- TS: `src/types.ts` (`EVENT_*` constants)

Event payloads:
- Rust: `src-tauri/src/events.rs` payload structs
- TS: `src/types.ts` payload interfaces

Core enums/types:
- Rust: `src-tauri/src/state.rs` (`CaptureState`, `ErrorCode`, `CaptureError`)
- TS: `src/types.ts` mirrors (string unions / interfaces)

Commands:
- Rust: `src-tauri/src/lib.rs` (`#[tauri::command]` functions, `generate_handler![...]`)
- TS: `src/App.tsx` (`invoke("...")` callsites)
- Tests/mocks: `src/test/setup.ts`

## Sync checklist (use on every contract change)

- Update Rust and TS in the same commit (no partial contract changes).
- If you change event names:
  - Update Rust `event_names::*`
  - Update TS `EVENT_*`
  - Update UI listeners and test mocks
- If you change payload fields:
  - Update Rust struct + TS interface byte-for-byte
  - Update any serde casing rules (Rust) and expectations (TS)
- If you add/remove a command:
  - Add/remove `#[tauri::command]`
  - Update handler registration
  - Update UI `invoke` callsites
  - Update `src/test/setup.ts` mocks and any Vitest expectations
