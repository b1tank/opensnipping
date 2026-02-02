# Refactor Plan (Risk-Reducing, ≤500 LOC)

Goal: reduce refactor risk as AI-generated code accumulates by enforcing clear module boundaries, shrinking oversized files, and preventing TS ⇄ Rust contract drift.

Non-goals:
- No feature changes.
- No UI/UX changes.
- No backend behavior changes.

Hard constraints:
- No source file should exceed 500 LOC unless explicitly allowlisted with a written reason and a removal deadline.
- Keep TS ⇄ Rust contract in sync (commands/events/types). Contract-first changes must update both sides in the same commit.
- Refactors must be atomic and verifiable: tests passing after every step.

Current oversized files (must be reduced):
- `src-tauri/src/capture/linux.rs` (~1200 LOC)
- `src-tauri/src/capture/fake.rs` (~860 LOC)
- `src-tauri/src/lib.rs` (~630 LOC)

---

## Phase 0 — Guardrails (no behavior change)

- [x] Add a LOC gate script at `scripts/check-loc.sh` (default limit: 500 lines)
- [x] Add `scripts/loc.allowlist.tsv` for temporary exceptions (path + reason + expiry)
- [x] Add `npm run check:loc` to run the LOC gate locally
- [x] Add a short doc section describing how to handle exceptions (must include reason + expiry)

Success criteria:
- Running `npm run check:loc` fails on new oversized files.
- Existing oversized files are allowlisted *temporarily* with explicit expiry.

---

## Phase 1 — Scaffolding for better organization (safe folders/files only)

Frontend (React/TS):
- [x] Add `src/tauri/README.md` (boundary: all invoke/listen wrappers live here)
- [x] Add `src/hooks/README.md` (boundary: React hooks only)
- [x] Add `src/features/README.md` (boundary: feature slices; no shared infra)
- [x] Add `src/utils/README.md` (boundary: pure helpers)

Backend (Rust):
- [x] Add `src-tauri/src/ipc/README.md` (boundary: Tauri commands + event emit helpers)
- [x] Add `src-tauri/src/domain/README.md` (boundary: pure state/config/domain logic)
- [x] Add `src-tauri/src/capture/linux/README.md` (future split target; no module wiring yet)
- [x] Add `src-tauri/src/capture/fake/README.md` (future split target; no module wiring yet)

Success criteria:
- No compilation changes required (folders/docs only).

---

## Phase 2 — Contract drift prevention (low-risk refactor)

- [x] Add `contract.surface.md` that lists:
  - Rust event names source: `src-tauri/src/events.rs`
  - TS event constants source: `src/types.ts`
  - Rust command names exposed: `src-tauri/src/lib.rs`
  - TS invoke callsites: `src/App.tsx` (to be moved later)
  - Test mocks source: `src/test/setup.ts`
- [x] Add a short “sync checklist” at bottom of `contract.surface.md`

Success criteria:
- A reviewer can quickly verify contract changes by reading one file.

---

## Phase 3 — Backend refactor to ≤500 LOC (behavior preserved)

### 3A) Split `src-tauri/src/lib.rs`

Target:
- `src-tauri/src/ipc/commands.rs` — all `#[tauri::command]` fns (thin glue)
- `src-tauri/src/ipc/emit.rs` — event emission helpers
- `src-tauri/src/ipc/errors.rs` — error mapping helpers
- `src-tauri/src/lib.rs` — minimal wiring + handler registration

Atomic tasks:
- [x] Create `ipc` module with `mod.rs` (or `ipc.rs`) and stubs
- [x] Move emit helpers first (pure extraction)
- [x] Move error mapping helpers next (pure extraction)
- [x] Move command functions last (no signature changes)
- [x] Run `cargo test` and ensure no warnings

Success criteria:
- `src-tauri/src/lib.rs` ends under 500 LOC.

### 3B) Split Linux backend (was `src-tauri/src/capture/linux.rs`)

Target layout:
- `capture/linux/mod.rs` — module surface + re-exports
- `capture/linux/encoding.rs` — encoder/muxer detection helpers
- `capture/linux/pipeline.rs` — recording pipeline implementation
- `capture/linux/backend.rs` — `LinuxCaptureBackend` implementation
- `capture/linux/tests.rs` — unit tests

Atomic tasks:
- [x] Convert Linux backend into a directory module
- [x] Split linux code into <= 500 LOC submodules
- [x] Keep public `LinuxCaptureBackend` API stable; only reorganize internals
- [x] Run `cargo test` and (optional) manual smoke run on Linux

Success criteria:
- No individual Linux capture module exceeds 500 LOC.
- `capture/linux.rs` no longer exists (or becomes a tiny `mod.rs`).
Success criteria:
### 3C) Split fake backend (was `src-tauri/src/capture/fake.rs`)

Target layout:
- `capture/fake/mod.rs` — module surface + re-exports
- `capture/fake/backend.rs` — `FakeCaptureBackend` + `CaptureBackend` impl
- `capture/fake/tests/*` — split tests to keep files <= 500 LOC

Atomic tasks:
- [x] Convert fake backend into a directory module
- [x] Split runtime code + tests into multiple files under `capture/fake/`
- [x] Keep behavior identical; run existing tests

Success criteria:
- No fake backend file exceeds 500 LOC.

---

## Phase 4 — Frontend boundary refactor (behavior preserved)

- [x] Create `src/tauri/commands.ts` to centralize `invoke("...")` names
- [x] Create `src/tauri/events.ts` (or a hook) to centralize event subscription
- [x] Update `App.tsx` to use wrappers (no logic changes)
- [x] Update `src/test/setup.ts` to mock the wrappers cleanly
- [x] Run `npm test` and `npm run build`

Success criteria:
- `App.tsx` shrinks and UI logic becomes easier to test.

---

## Verification checklist (run on every refactor step)

- `cd opensnipping && npm run check:loc`
- `cd opensnipping && npm test`
- `cd opensnipping && npm run build`
- `cd opensnipping/src-tauri && cargo test`

Notes:
- If a step requires >100 LOC changes, decompose it into multiple atomic commits.
- No edits under `src-tauri/target/`.
