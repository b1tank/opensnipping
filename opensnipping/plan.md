# MVP Plan — Screen Recorder + Screenshot Tool

This plan is derived from:
- Product requirements in [spec.md](spec.md)
- A “seams first” architecture principle: keep UI + orchestration cross-platform, keep capture OS-specific.

## Stack Summary (by layer)

| Layer | Choice (MVP) | Notes |
| --- | --- | --- |
| UI / UX | Tauri v2 + React + TypeScript + Vite | Minimal ephemeral controls; screenshot annotation in web UI |
| Orchestration core | Rust (Tokio + Serde + tracing) | State machine, IPC/events to UI, config validation |
| Linux capture | xdg-desktop-portal (via `ashpd`) + PipeWire | Wayland-first; also works on X11; native picker/permissions |
| Encode + mux | GStreamer | H.264 (HW accel when available, x264 fallback) → MP4/MKV mux |
| Packaging | Tauri bundling | Linux: `deb` + AppImage (Flatpak later if desired) |
| Windows backend (Phase 2) | DXGI Desktop Duplication + WASAPI + Media Foundation/FFmpeg | Implement behind the same capture contract |
| macOS backend (Phase 2) | ScreenCaptureKit + CoreAudio + AVAssetWriter/FFmpeg | Implement behind the same capture contract |

The result below is a single coherent stack + an implementation plan that builds an MVP on Linux (GNOME/Wayland/X11) without painting you into a corner for Windows/macOS.

---

## How to Iterate (atomic, fast, verifiable)

Each step below is intentionally sized so you can:
1) implement in minutes to a couple hours,
2) run a quick local verification (UI action and/or CLI test command),
3) commit safely.

**Definition of “atomic step” used in this plan**
- Adds one thin slice of functionality behind a stable contract
- Includes a basic test (unit/integration) appropriate to the slice
- Includes a simple manual smoke check (e.g., click a button, produce a file)

**Testing strategy (critical, start early)**
- UI tests: **Vitest + React Testing Library** (component + simple interaction tests)
- Rust tests: `cargo test` for state machine, config validation, backend selection logic
- “Contract tests”: run the orchestration against a **FakeCaptureBackend** that simulates events (no OS capture needed)
- Linux “smoke tests”: minimal end-to-end runs that verify a pipeline can start/stop and produce output (guarded by env checks; skip if dependencies missing)

---

## 0) What “MVP” Means Here (so we don’t overbuild)

**MVP target platform**: Linux (Ubuntu/GNOME), Wayland-first with X11 support.

**“Windows/macOS supported” in MVP**: project compiles and the app shell runs; capture backends can be stubbed. Platform capture backends become Phase 2.

**Explicit non-goals** (per spec): no video annotation, no streaming, no video editing.

---

## 1) Final Stack (Recommended)

### 1.1 UI / App Shell (cross-platform)
- **Tauri v2** (desktop shell)
- **React + TypeScript + Vite** (UI)
- **Web Canvas annotation** (screenshot-only): HTML Canvas with minimal freehand pen, undo/clear, export
  - Recommended library: **Konva.js** (simple) or plain canvas (simplest long-term)

Why: meets “popular & familiar technologies” while keeping binary size and OS integration far better than Electron for Linux GNOME capture.

### 1.2 Orchestration Core (cross-platform)
- **Rust** (core orchestration, state machine, IPC with UI)
- **Serde** (config and events)
- **Tokio** (async orchestration) + **tracing** (structured logs)

### 1.3 Linux Capture Backend (MVP)
- **xdg-desktop-portal** (Wayland-friendly permissions & selection)
  - Rust binding: `ashpd`
- **PipeWire** (video frames, screen/window/monitor/region)
- **GStreamer** (recording pipeline, muxing, A/V sync)
  - Rust crates: `gstreamer`, `gstreamer-video`, `gstreamer-audio`, `gstreamer-pbutils`

Linux encoding approach:
- Primary: **GStreamer encoders**
  - Software: `x264enc`
  - Hardware (when available): `vaapih264enc` (Intel/AMD iGPU), `nvh264enc` (NVIDIA), `amfh264enc` (AMD)
- Muxing: `mp4mux` (MP4), `matroskamux` (MKV)

Audio approach:
- **PipeWire / PulseAudio via GStreamer sources**
  - system audio: via PipeWire session / portal stream (preferred) or pulse monitor (fallback on X11)
  - mic: `pulsesrc` / PipeWire source
- Use `audiomixer` + timestamps for sync where needed.

### 1.4 Windows/macOS Capture Backends (Phase 2)
Keep behind the same interface:
- **macOS**: ScreenCaptureKit + CoreAudio (Process Tap) + AVAssetWriter (or FFmpeg)
- **Windows**: Desktop Duplication (DXGI) + WASAPI loopback + Media Foundation (or FFmpeg)

### 1.5 Packaging / Distribution
- Tauri bundling per OS
- Linux packaging: `deb` + `AppImage` (Snap/Flatpak later)
  - If Flatpak is desired, portals become even more important (good).

---

## 2) Architecture (Seams First)

### 2.1 Process model
Single-process (Tauri app) is fine for MVP. If stability issues arise for long recordings, move capture into a helper process later.

### 2.2 Modules
- **UI layer (TS/React)**: ephemeral controls, region selection overlay, annotation screen
- **Orchestration (Rust)**: recording state machine, permission checks, config validation
- **Capture backends (Rust modules per OS)**:
  - `capture/linux/*`
  - `capture/windows/*` (stub in MVP)
  - `capture/macos/*` (stub in MVP)

### 2.3 Stable Capture Contract (Rust-side)
Define this early and don’t leak OS details into UI:

- Commands:
  - `start_capture(config)`
  - `pause_capture()`
  - `resume_capture()`
  - `stop_capture()`
  - `take_screenshot(config)`

- Events emitted to UI:
  - `status_changed({state})`
  - `permission_needed({kind})`
  - `progress({duration_ms})`
  - `error({code, message})`

UI only understands “needs_permission: screen” — not “portal failed with …”.

---

## 3) Feature Mapping (Spec → Implementation)

- Screen capture (full / monitor / window / region): Linux via portal selection + PipeWire stream
- Wayland + X11: portal path works for Wayland; X11 may support extra fallbacks if needed
- HiDPI / fractional scaling: rely on portal metadata + PipeWire stream size; ensure cursor scale handling
- Recording: GStreamer pipeline with H.264 + MP4/MKV
- HW accel: select best available encoder element at runtime
- Audio: capture system + mic and mux, maintain sync
- Mouse cursor: portal stream usually includes cursor; verify; add toggle optional
- Minimal UI: floating always-on-top control; hotkeys; indicator
- Screenshot + annotation: capture frame → UI annotation → export PNG

---

## 4) Atomic Implementation Plan (Step-by-Step)

Each step should end with a **demoable artifact** (a visible behavior or an output file).

### Milestone 0 — “Hello App” + Test Harness (fastest first) (30–90 minutes)
- [x] 1. Scaffold Tauri v2 app (Vite + React + TS) and make `npm run dev` open a desktop window.
- [x] 2. Add a minimal “Hello” view plus two buttons: “Ping Rust” and “Toggle Mode”.
- [x] 3. Wire a single Tauri command (Rust) that returns a string and show it in the UI.
- [x] 4. Add test harnesses:
   - UI: Vitest + React Testing Library, one smoke test that renders the app and clicks “Toggle Mode”.
   - Rust: `cargo test` running one trivial unit test.

**Done when**: app opens and is clickable; `npm test` and `cargo test` both pass.

### Milestone 1 — Contract + State Machine (half day)
- [x] 5. Define `CaptureConfig` (serde) aligned with spec:
   - source: screen|monitor|window|region
   - fps, include_cursor
   - audio: mic/system toggles
   - container: mp4|mkv
   - output path
- [x] 6. Implement orchestration state machine (Rust):
   - states: `Idle`, `Selecting`, `Recording`, `Paused`, `Finalizing`, `Error`
   - validate transitions (e.g., pause only from Recording)
- [x] 7. Implement event bus from Rust → UI (Tauri events):
   - status changes, errors
- [x] 8. Add tests:
   - state transition tests (valid/invalid)
   - config validation tests (bad inputs rejected)

**Done when**: UI displays state changes; `cargo test` covers state transitions.

### Milestone 2 — Linux Permissions + Portal Selection (half day)
- [x] 9. Add Linux-only portal integration (`ashpd`):
   - request screencast session
   - source selection (screen/window/region)
- [x] 10. Return a "selection token / PipeWire node id" to Rust capture backend.
- [x] 11. UI: keep selection UX minimal; prefer the portal picker.
- [x] 12. Add tests:
   - contract test using a FakeCaptureBackend to ensure “Start → Selecting → Recording” event flow works
   - (optional) Linux-only integration test that is skipped unless `XDG_CURRENT_DESKTOP` and portal are present

**Done when**: clicking “Start” shows the GNOME portal picker and returns a usable stream descriptor (logged).

### Milestone 3 — Screenshot MVP (1 day)
- [ ] 13. Build Linux screenshot pipeline:
   - connect to PipeWire stream
   - grab a single frame
   - write PNG to disk (`image` crate) OR via GStreamer `pngenc`
- [ ] 14. UI: after screenshot captured, show annotation screen:
   - single pen color
   - fixed stroke width
   - undo/clear
   - export (overwrite/copy)
- [ ] 15. Add tests:
   - UI: annotation component tests (draw action mocked; undo/clear actions)
   - Rust: unit test for filename/path generation and “screenshot completed” event emission

**Done when**: user can take a region/window/screen screenshot and export an annotated PNG.

### Milestone 4 — Recording MVP (no audio) (1–2 days)
- [ ] 16. Create GStreamer recording pipeline (video only) from PipeWire:
   - source → colorspace/scale → encoder (hw if available else x264) → mux (mp4/mkv) → filesink
- [ ] 17. Implement Start/Stop end-to-end, producing playable files.
- [ ] 18. Implement Pause/Resume:
   - simplest: pause the pipeline / block dataflow (verify output correctness)
   - fallback if pause is hard: implement “segmented recording” and concat (only if necessary)
- [ ] 19. Add tests:
   - Rust: backend selection logic chooses expected encoder/mux given availability flags
   - Linux smoke test: start/stop a 2–3s recording and assert output file exists and is non-empty (skip if deps missing)

**Done when**: user can record screen/window/region to MP4/MKV with pause/resume.

### Milestone 5 — Add Audio (system + mic) + Sync (2–4 days)
- [ ] 20. Add microphone audio source and encode (AAC/Opus depending on container):
   - MP4: AAC recommended
   - MKV: Opus acceptable
- [ ] 21. Add system audio capture:
   - prefer: portal-provided audio with the screencast session if available
   - fallback: PulseAudio monitor source (X11 / non-portal environments)
- [ ] 22. Mix mic + system (if both enabled).
- [ ] 23. Verify A/V sync over a 10–20 minute recording.
- [ ] 24. Add tests:
   - Rust: config matrix tests (mic only / system only / both)
   - Linux smoke test: short recording with audio enabled produces a playable file

**Done when**: recordings include mic + system audio with stable sync.

### Milestone 6 — Cursor Correctness + HiDPI (1–2 days)
- [ ] 25. Verify cursor visibility, shape, and scaling in recordings.
- [ ] 26. If needed, implement cursor overlay:
   - read cursor metadata + composite into frames (only if portal stream lacks correct cursor)
- [ ] 27. Add tests:
   - Rust: cursor config toggle tests
   - Manual test checklist: record on HiDPI/fractional scaling and verify cursor behavior

**Done when**: cursor looks correct on HiDPI/fractional scaling setups.

### Milestone 7 — UX Polish: Ephemeral UI, Indicator, Hotkeys (1–2 days)
- [ ] 28. Implement minimal floating control window:
   - always-on-top
   - tiny footprint
   - hides when idle
- [ ] 29. Add recording indicator (red dot / timer).
- [ ] 30. Add global hotkeys:
   - Start/Stop
   - Pause/Resume
   - Screenshot
- [ ] 31. Add error UX:
   - permission needed
   - portal denied
   - encoder unavailable
- [ ] 32. Implement portal restore token for persistent screen selection:
   - Use `PersistMode::Application` instead of `DoNot`
   - Store returned `restore_token` (Tauri store plugin or file)
   - Pass token to subsequent `select_sources` calls to skip picker
   - Handle token invalidation gracefully (fall back to picker)
- [ ] 33. Add tests:
   - UI: indicator + timer rendering tests
   - Rust: hotkey command wiring tested via unit tests around command handlers (logic, not OS key registration)

**Done when**: full spec control flow works without a “settings app” feel.

### Milestone 8 — Reliability + Linux Packaging (2–5 days)
- [ ] 34. Recovery & cleanup:
   - crash-safe finalization
   - temp files
   - handle portal session invalidation
- [ ] 35. Add basic telemetry logs (local only): per-recording pipeline summary.
- [ ] 36. Add CI to prevent regressions:
   - run UI tests (`npm test`) and Rust tests (`cargo test`) on every push
   - (optional) Linux integration tests behind a separate job and/or feature flag
- [ ] 37. Packaging:
   - `deb` + AppImage
   - document dependencies (GStreamer plugins)

**Done when**: installable builds exist; long recordings are stable.

---

## 5) Phase 2 (Windows/macOS) Plan (High Level)

1. Keep UI + orchestration identical.
2. Implement `CaptureBackend` for each OS.
3. Match config & events contract; do not leak OS-specific permission UI.
4. Ensure encoding happens inside native backend for performance.
5. Cross-platform distribution (GitHub Releases):
   - Add GitHub Actions build matrix (Linux/Windows/macOS) with Tauri bundling.
   - Use `tauri-action` (or equivalent) to produce native installers:
     - Windows: `.msi` (and/or `.exe`)
     - macOS: `.dmg` (and/or `.app` zip)
     - Linux: `deb` + AppImage
   - Configure release workflow to attach artifacts to GitHub Releases (tagged builds).
   - Add code signing placeholders and documentation:
     - Windows: Authenticode cert (optional for MVP)
     - macOS: Apple Developer ID + notarization (optional for MVP)
   - Add a release checklist in README (manual steps if signing is skipped).
   - Done when: a tagged release produces installers for all 3 OSes and publishes them to GitHub Releases.

---

## 6) Key Risks & Mitigations

- **Wayland capture**: must go through portal/PipeWire. Mitigation: portal-first design.
- **Audio routing differences**: system audio capture varies by distro. Mitigation: portal audio preferred; fallback strategies.
- **Pause/Resume correctness**: pipelines may not “pause” cleanly for MP4. Mitigation: test early; use MKV for pause robustness if needed.
- **GStreamer plugin availability**: encoders/muxers differ by install. Mitigation: dependency checks + clear error messages.

---

## 7) Next Action (If you want me to proceed)

If you confirm this stack, the next concrete step is to scaffold the Tauri app and wire the Rust command/event contract (Milestones 0–1).
