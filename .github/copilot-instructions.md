# Repository Guidelines

Remote repo: https://github.com/b1tank/opensnipping

A Tauri v2 desktop app (React/TypeScript frontend + Rust backend) for lightweight screen recording and screenshots.

Primary docs:
- Spec: `opensnipping/spec.md`
- Plan: `opensnipping/plan.md`
- README: `README.md`

When instructions conflict, spec/plan takes precedence.

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
- `opensnipping/src-tauri/target/` is build output—do not edit or depend on it.
- Currently an MVP "orchestration shell" (state machine + UI wiring), not a full capture pipeline.

## Coding Principles

- **Separation of concerns**: Frontend handles UI; backend handles capture logic.
- **Modular code**: Write reusable, testable components and functions.
- **Small changes**: Make incremental changes that are easy to review and test.
- **Platform logic in Rust**: OS/media/capture code belongs in the backend; frontend stays declarative and mockable.
- **First-class errors**: Every command/event must have explicit failure behavior (error code + message); UI must surface errors visibly.
- **Intentional logging**: Log at key control points (command entry/exit, state transitions, error boundaries).
- **No hidden side effects**: Background tasks/timers require explicit lifecycle (start/stop) and clear ownership.
- **Additive over refactor**: Avoid broad refactors during feature work. If necessary, refactor in a separate commit with no behavior change.

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

> **See also:** [tauri-contract skill](.github/skills/tauri-contract/SKILL.md) for detailed sync checklist.

Keep Rust backend and TS frontend in sync. The UI depends on these being stable:

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

When adding/removing fields or enum variants, update both sides and adjust tests.

**Contract-first changes:** When modifying events/commands/types, update both Rust (`events.rs`/`lib.rs`) and TS (`types.ts`) in the same commit. Keep names/fields byte-for-byte consistent.

## Orchestration & State Machine

The Rust `StateMachine` is the single source of truth for capture transitions:
- Definition: `opensnipping/src-tauri/src/state.rs`
- UI reacts to Rust events (`capture:state_changed`) or queries `get_state`—never invents state.

**State machine is law:** UI never "fixes up" state; it only requests actions and renders backend state. New states/transitions go into Rust `StateMachine` first (with tests), then wire to UI.

New user actions (commands) require end-to-end wiring:
1. Add `#[tauri::command]` in `opensnipping/src-tauri/src/lib.rs`
2. Export via `tauri::generate_handler![...]`
3. Call via `invoke(...)` in UI
4. Add tests:
	- UI: update mocks in `opensnipping/src/test/setup.ts` and tests in `opensnipping/src/App.test.tsx`
	- Rust: add unit tests in the relevant module

## Testing Guidelines

**Frontend:**
- Test user-visible behavior (button click → invoke called → UI updates).
- Mock Tauri interactions via `opensnipping/src/test/setup.ts`.

**Rust:**
- Keep domain logic testable without a running app window.
- Follow the pattern in `opensnipping/src-tauri/src/state.rs`.

**Deterministic over "real device" tests:** Prefer unit tests around Rust domain logic and Vitest UI mocks. Avoid tests requiring a running Tauri window unless necessary.

**Manual verification:** Print verification steps before running any server command (e.g., `npm run tauri dev`) so users can follow along.

**Error handling:** When tests reveal new error types, add explicit handling immediately—map to user-visible errors, log with context, and update tests.

## Guardrails / Hygiene

- **Atomic changes**: One feature slice per PR/commit.
- **Avoid large refactors** unless required by spec/plan.
- **Zero warnings**: Compilation must have no warnings or errors. Fix hygiene issues immediately (e.g., `cargo check` dead_code warnings).
- **No hardcoded paths** in UI beyond temporary MVP scaffolding (e.g., `/tmp/recording.mp4` is a placeholder).
- **Never edit build artifacts** (especially `opensnipping/src-tauri/target/`).

**Small diffs, explicit intent:** Each change has one purpose ("add command X", "add event Y"). Avoid drive-by formatting/renames.

## Work Categories

Classify all work before starting to maintain atomic commits:

| Category   | Description                                      | Typical Risk |
|------------|--------------------------------------------------|---------------|
| `feat`     | New feature or capability                        | High |
| `fix`      | Bug fix                                          | Medium-High |
| `docs`     | Documentation only                               | Low |
| `refactor` | Code restructure without behavior change         | Medium |
| `test`     | Adding or updating tests                         | Low-Medium |
| `chore`    | Build, CI, config, or maintenance tasks          | Low |
| `agent`    | Agent/copilot instructions, prompts, AGENTS.md, CLAUDE.md | Low |

## Plan Management & Task Decomposition

> **See also:** `decompose-task` skill for detailed decomposition procedure.

**`plan.md` is a living document**—not a static roadmap. It evolves as complexity is discovered.

**Before starting any task**, evaluate decomposition need:

| Task Size Indicator | Typical Lines Changed | Action |
|---------------------|----------------------|--------|
| Simple fix | 1–10 lines | Proceed directly |
| Small piece (test, function, interface) | 10–50 lines | Proceed directly |
| Medium task | 50–100 lines | Acceptable, but review scope |
| Large task | >100 lines | **Must decompose first** |

**Decomposition workflow:**
1. If task appears to require >100 lines, stop before implementing
2. Break into sub-tasks with checkboxes in `plan.md` (each = one atomic commit)
3. Ask human: "I've decomposed [task] into [N] sub-tasks. Does this look right?"
4. Proceed only after confirmation

**Signs a task needs decomposition:**
- Multiple unrelated files changing
- Multiple state transitions or new states
- Backend + frontend changes beyond contract sync
- Vague descriptions like "implement pipeline" or "add full support"

**Do not over-decompose:** Tasks completable in <50 lines with clear intent stay as one task.

## Commit Message Guide

Format: `<category>: <short description>`

Examples:
- `feat: add region selection overlay`
- `fix: handle null capture config`
- `docs: update README with install steps`
- `refactor: extract validation into config module`
- `test: add state machine transition tests`
- `chore: update Tauri to v2.1`
- `agent: clarify commit message conventions`

Keep descriptions lowercase, imperative, under 50 characters.

**Always use `agent:` prefix** for changes to agent instructions (`.github/agents/`, `.github/prompts/`, `.github/skills/`, `copilot-instructions.md`, `AGENTS.md`, `CLAUDE.md`).

## Commit and Push Policy

**Atomic commits are mandatory**—each commit must be self-contained and verifiable:
- Each numbered task in `plan.md` (e.g., 13a, 15a) = at least one commit
- Multiple commits per task are fine if logically separated
- Never bundle unrelated changes
- Test tasks are real tasks—commit separately

**Scope your commits:** Only commit files you changed. Other agents may work in parallel.
- Run `git status` before committing
- Stage explicitly (`git add file1 file2`), not `git add -A`
- Ask human if you see unexpected changes

**Commit quality evidence:** Tests passing is necessary but not sufficient. State evidence of correctness:
- **New test added**: Test exercises the new code path and passes
- **Contract-only change**: Type/event additions with no runtime behavior yet
- **Refactor with existing coverage**: Existing tests cover the changed behavior
- **Manual verification**: Steps performed and results observed

Include brief rationale (e.g., "Tests pass; new `capture_screenshot` method is interface-only, no callers yet").

**Pre-commit checks:** Use the `diff-check` skill to validate changes before committing.

**Human verification decision flow:**
- Verifiable yourself (tests pass, contract-only, deterministic logic) → commit and push
- Requires visual/UX verification → ask human first with steps, then commit after confirmation

**Proactive UI/UX verification:** When changes affect user-visible behavior, offer the human a chance to see it before committing—even if tests pass. Frame as opportunity, not blocker (e.g., "Want to see the new [feature] before I commit?"). Print verification steps first.

## Parallel Work & Agent Delegation

When you discover independent secondary work (bugs, missing docs/tests, refactor opportunities) during a primary task, offer delegation options:
- **Monitored**: Another agent works in parallel; human reviews both outputs
- **YOLO/Background**: Agent works autonomously on low-risk tasks (docs, tests, chore)

See [engineer.agent.md](agents/engineer.agent.md) for delegation protocols.

## Agent Notes

Available agents:
- **product** (`.github/agents/product.agent.md`): Product designer—drafts spec.md from ideas
- **lead** (`.github/agents/lead.agent.md`): Tech lead—orchestrates work, generates plan.md
- **engineer** (`.github/agents/engineer.agent.md`): Dedicated implementation—writes code, tests, commits
- **reviewer** (`.github/agents/reviewer.agent.md`): Code reviewer—critical "grill me" feedback

Available skills:
- **decompose-task**: Break large tasks (>100 lines) into atomic sub-tasks
- **diff-check**: Author cleanup before commit/PR submit
- **tauri-contract** (`.github/skills/tauri-contract/SKILL.md`): TS ⇄ Rust contract sync

Available prompts:
- `/new-project`: Start new project → @product
- `/new-agent`: Create new agent → @lead
- `/work-on-next`: Next task → @lead
- `/self-improve`: Improve agent setup

When uncertain:
- Read `opensnipping/spec.md` and `opensnipping/plan.md` first
- Prefer verified answers from code over guesses

## Learning Mode

When explaining code or changes:
- Explain the *why* behind changes, not just the what
- Use ASCII diagrams to illustrate architecture, data flow, protocols
- Offer to generate visual HTML presentations for complex concepts
- When reviewing unfamiliar code, summarize structure before diving in
- Ask follow-up questions to fill knowledge gaps
