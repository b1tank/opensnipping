# capture/linux

Linux screen capture backend using PipeWire and the XDG Desktop Portal.

## Files

- `mod.rs` — Module surface + re-exports
- `backend.rs` — `LinuxCaptureBackend` implementation (portal interaction, screenshot)
- `encoding.rs` — Encoder/muxer detection helpers (H.264, VP8, audio codecs)
- `pipeline.rs` — GStreamer recording pipeline implementation
- `tests.rs` — Unit tests for encoder detection

## Rules

- Keep each module under 500 LOC
- Public API is `LinuxCaptureBackend` only; internals are `pub(crate)` or private
- Pipeline owns GStreamer lifecycle; backend owns portal session lifecycle
