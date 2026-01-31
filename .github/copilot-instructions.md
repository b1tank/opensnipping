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
- Add clear, intentional logging at key control points (command entry/exit, state transitions, and error boundaries) to aid debugging and maintenance.
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

**Manual verification guidance:** When asking for or running visual verification, always print the manual verification steps before running any server/startup command (e.g., `npm run tauri dev`) so the user can follow along.

**Error handling maintenance:** If tests reveal new error types or cases, add explicit handling in code immediately (map to user-visible errors, log with context, and update tests). Keep error handling additive and consistent to make future fixes easier.

## Guardrails / Hygiene

- Keep changes scoped and “atomic”: one feature slice per PR/change.
- Avoid large refactors unless required by the spec/plan.
- Compilation should have no warnings (and no errors). Fix hygiene warnings immediately (e.g., `cargo check` dead_code warnings like unused fields).
- Don’t hardcode platform paths in UI beyond clearly temporary MVP scaffolding.
	- Current UI uses `/tmp/recording.mp4` in `opensnipping/src/App.tsx`; treat this as a placeholder.
- Don’t edit generated build artifacts (especially `opensnipping/src-tauri/target/`).

**Small diffs, explicit PR intent:** Each change should have a single intent ("add command X", "add event Y", "tighten validation"), and avoid drive-by formatting/renames.

## Work Categories

All work falls into: **feat**, **fix**, **docs**, **refactor**, **test**, **chore**, **agent**. Classify before starting to maintain atomic commits.

| Category   | Description                                      |
|------------|--------------------------------------------------|
| `feat`     | New feature or capability                        |
| `fix`      | Bug fix                                          |
| `docs`     | Documentation only                               |
| `refactor` | Code restructure without behavior change         |
| `test`     | Adding or updating tests                         |
| `chore`    | Build, CI, config, or maintenance tasks          |
| `agent`    | Improving agent instructions or prompts          |

## Commit Message Guide

Use the format: `<category>: <short description>`

Examples:
- `feat: add region selection overlay`
- `fix: handle null capture config`
- `docs: update README with install steps`
- `refactor: extract validation into config module`
- `test: add state machine transition tests`
- `chore: update Tauri to v2.1`
- `agent: clarify commit message conventions`

Keep the description lowercase, imperative, and under 50 characters.

## Commit and Push Policy

- Commit and push without asking for approval, but only when the commit is logically cohesive, compiled and at least minimally verified by simple tests or manual checks.

## Parallel Work & Agent Delegation

When you discover independent secondary work (bugs, missing docs/tests, refactor opportunities) while on a primary task, prompt the human with delegation options:
- **Monitored**: Another agent works in parallel, human reviews both outputs
- **YOLO/Background**: Agent works autonomously on low-risk tasks (docs, tests, chore)

See `.github/agents/engineer.agent.md` for detailed delegation protocols and prompt formats.

## Agent Notes

There is an agent entrypoint at `.github/agents/engineer.agent.md` that points to the spec/plan and requests atomic steps + tests.

When uncertain:
- Read `opensnipping/spec.md` and `opensnipping/plan.md` first.
- Prefer verified answers from code over guesses.
