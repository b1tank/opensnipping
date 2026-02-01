---
name: tauri-contract
description: TS ⇄ Rust contract sync procedure for Tauri v2 apps. Use when modifying events, commands, types, or payloads that cross the frontend/backend boundary. Ensures both sides stay in sync.
---

# Tauri Contract Sync

Keep Rust backend and TS frontend in sync when modifying the cross-layer contract.

## When to Use

- Adding/removing event names
- Modifying event payloads
- Adding/removing Tauri commands
- Changing core enums/types (states, errors)
- Modifying configuration schemas

## File Mappings

| Contract Element | Rust Source | TS Mirror |
|------------------|-------------|-----------|
| Event names | `src-tauri/src/events.rs` (`event_names::*`) | `src/types.ts` (`EVENT_*`) |
| Event payloads | `src-tauri/src/events.rs` (structs) | `src/types.ts` (interfaces) |
| Core enums | `src-tauri/src/state.rs` | `src/types.ts` |
| Error types | `src-tauri/src/state.rs` | `src/types.ts` |
| Config schema | `src-tauri/src/config.rs` | `src/types.ts` |
| Commands | `src-tauri/src/lib.rs` | `invoke(...)` calls |

## Sync Checklist

When modifying contract elements:

- [ ] **1. Rust first**: Make changes in Rust source files
- [ ] **2. TS mirror**: Update corresponding TS types/interfaces
- [ ] **3. Name consistency**: Keep names byte-for-byte identical
- [ ] **4. Field consistency**: Match field names, types, optionality
- [ ] **5. Tests**: Update both Rust and TS tests
- [ ] **6. Single commit**: Contract changes go in one atomic commit

## Adding a New Command

1. Add `#[tauri::command]` in `src-tauri/src/lib.rs`
2. Export via `tauri::generate_handler![...]`
3. Define return/error types if new
4. Add TS type for return value in `src/types.ts`
5. Call via `invoke(...)` in UI
6. Update mocks in `src/test/setup.ts`
7. Add tests in both layers

## Adding a New Event

1. Add event name constant in Rust `events.rs` (`event_names::*`)
2. Add payload struct in Rust `events.rs` (with `#[derive(Serialize)]`)
3. Add TS event name constant in `types.ts` (`EVENT_*`)
4. Add TS payload interface in `types.ts`
5. Emit from Rust via `app_handle.emit(...)`
6. Listen in TS via `listen(EVENT_*, callback)`

## Guidelines

- **Contract-first**: Always design the contract before implementing
- **Minimal payloads**: Only include data the UI needs
- **Explicit errors**: Every command returns `Result<T, E>` with typed errors
- **Versioning**: For breaking changes, consider deprecation path
