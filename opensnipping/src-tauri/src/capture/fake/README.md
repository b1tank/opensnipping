# capture/fake (planned split)

This folder is a landing zone for splitting `capture/fake.rs` into smaller modules.

Rules:
- Keep the fake backend deterministic and test-friendly.
- Final goal: keep each module under 500 LOC.
