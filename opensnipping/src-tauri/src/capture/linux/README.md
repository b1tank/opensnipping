# capture/linux (planned split)

This folder is a landing zone for splitting `capture/linux.rs` into smaller modules.

Rules:
- Do not add Rust module wiring here until the split is executed (to avoid compile ambiguity).
- Final goal: keep each module under 500 LOC.
