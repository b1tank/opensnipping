# Repository Guidelines

Remote repo: https://github.com/b1tank/opensnipping

This repo is a Tauri v2 desktop app (React/TypeScript frontend + Rust backend) for a lightweight screen recorder + screenshot tool.

Primary product docs:
- Spec: `opensnipping/spec.md`
- Roadmap/plan: `opensnipping/plan.md`
- Public-facing README: `README.md`

If instructions conflict, prefer the spec/plan over convenience.

## Project Structure & Module Organization

- Frontend app (Vite + React + TS): `opensnipping/`
	- Entry: `opensnipping/src/main.tsx`
	- App UI: `opensnipping/src/App.tsx`
	- Shared contract types/events (TS): `opensnipping/src/types.ts`
	- Tests + mocks:
		- Vitest setup: `opensnipping/src/test/setup.ts`
		- Example test: `opensnipping/src/App.test.tsx`
- Tauri backend (Rust): `opensnipping/src-tauri/`
	- Tauri config: `opensnipping/src-tauri/tauri.conf.json`
	- App entrypoint: `opensnipping/src-tauri/src/main.rs`
	- Tauri commands + event emission glue: `opensnipping/src-tauri/src/lib.rs`
	- Domain state machine: `opensnipping/src-tauri/src/state.rs`
	- Capture config + validation: `opensnipping/src-tauri/src/config.rs`
	- Event payload structs + event names: `opensnipping/src-tauri/src/events.rs`

Notes:
- `opensnipping/src-tauri/target/` is build output; don’t edit or rely on it.
- This repo currently contains an MVP “orchestration shell” (state machine + UI wiring) more than a full capture pipeline.

## Coding Principles

- Prioritize separation of concerns between frontend (UI) and backend (capture logic).
- Write code in modular, reusable, and testable components/functions/files.
- Make small, incremental changes that are easy to review and test.
- Keep platform-specific logic in Rust: Anything OS/media/capture-related belongs in the backend; frontend stays declarative and testable with mocks.
- Error paths are first-class: Every new command/event should have explicit failure behavior (error code + message), and the UI should display something user-visible rather than silently failing.
- No hidden side effects: Don't introduce implicit background tasks/timers without an explicit lifecycle (start/stop) and a clear owner (usually the state machine).
- Prefer additive over refactor: Avoid broad refactors while implementing features. If a refactor is truly required, do it as a separate atomic step with no behavior change.

## Build, Test, and Development Commands

Frontend + Tauri dev (recommended):
- `cd opensnipping && npm install`
- `cd opensnipping && npm run tauri dev`

Frontend-only:
- `cd opensnipping && npm run dev`
- `cd opensnipping && npm run build`

Tests:
- UI tests: `cd opensnipping && npm test`
- Rust tests: `cd opensnipping/src-tauri && cargo test`

Tooling notes:
- Vite dev server is pinned to port 1420 (see `opensnipping/vite.config.ts`).
- Vitest runs in JSDOM and mocks Tauri APIs (see `opensnipping/vitest.config.ts` and `opensnipping/src/test/setup.ts`).

## Cross-Layer Contract (TS ⇄ Rust)

Keep the Rust backend and the TS frontend in sync. The UI assumes these are stable:

- Event names
	- Rust source of truth: `opensnipping/src-tauri/src/events.rs` (`event_names::*`)
	- TS constants: `opensnipping/src/types.ts` (`EVENT_*`)

- Event payloads
	- Rust structs: `StateChangedEvent`, `ErrorEvent`, etc. in `opensnipping/src-tauri/src/events.rs`
	- TS interfaces: `StateChangedEvent`, `ErrorEvent`, etc. in `opensnipping/src/types.ts`

- Core enums/types
	- Rust: `CaptureState`, `ErrorCode`, `CaptureError` in `opensnipping/src-tauri/src/state.rs`
	- TS: `CaptureState`, `ErrorCode`, `CaptureError` in `opensnipping/src/types.ts`

- Capture configuration
	- Rust schema + validation: `opensnipping/src-tauri/src/config.rs` (`CaptureConfig::validate`)
	- TS shape: `opensnipping/src/types.ts` (`CaptureConfig`)

When adding/removing fields or enum variants, update BOTH sides and adjust tests.

**Contract-first changes:** If you touch events/commands/types, update both sides (Rust in `events.rs` / `lib.rs` and TS in `types.ts`) in the same change, and keep names/fields byte-for-byte consistent.

## Orchestration & State Machine

The Rust `StateMachine` is the source of truth for legal capture transitions:
- See `opensnipping/src-tauri/src/state.rs`.
- UI should not “invent” state; it should react to Rust events (`capture:state_changed`) and/or query `get_state`.

**State machine is law:** UI should never "fix up" state; it can only request actions and render backend state. If a UI behavior needs a new state/transition, add it to the Rust `StateMachine` first (and tests there), then wire UI.

If you add new user actions (commands), wire them end-to-end:
- Add a `#[tauri::command]` in `opensnipping/src-tauri/src/lib.rs`
- Export it via `tauri::generate_handler![...]`
- Call it via `invoke(...)` in the UI
- Add/adjust tests:
	- UI: update mocks in `opensnipping/src/test/setup.ts` and tests in `opensnipping/src/App.test.tsx`
	- Rust: add unit tests in the relevant module

## Testing Guidelines

Frontend:
- Prefer UI tests that assert user-visible behavior (button click → invoke called → UI updates).
- Keep Tauri interactions mocked via `opensnipping/src/test/setup.ts`.

Rust:
- Keep domain logic testable without a running app window.
- The existing state machine tests in `opensnipping/src-tauri/src/state.rs` are the pattern to follow.

**Deterministic tests over "real device" tests:** Add/adjust unit tests around the Rust domain logic and Vitest UI mocks; avoid tests that require a running Tauri window unless absolutely necessary.

## Guardrails / Hygiene

- Keep changes scoped and “atomic”: one feature slice per PR/change.
- Avoid large refactors unless required by the spec/plan.
- Don’t hardcode platform paths in UI beyond clearly temporary MVP scaffolding.
	- Current UI uses `/tmp/recording.mp4` in `opensnipping/src/App.tsx`; treat this as a placeholder.
- Don’t edit generated build artifacts (especially `opensnipping/src-tauri/target/`).

**Small diffs, explicit PR intent:** Each change should have a single intent ("add command X", "add event Y", "tighten validation"), and avoid drive-by formatting/renames.

## Agent Notes

There is an agent entrypoint at `.github/agents/engineer.agent.md` that points to the spec/plan and requests atomic steps + tests.

When uncertain:
- Read `opensnipping/spec.md` and `opensnipping/plan.md` first.
- Prefer verified answers from code over guesses.
