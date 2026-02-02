---
name: verify-ux
description: Verify UX by invoking @ui-tester agent with optional feature specification.
---

Invoke @ui-tester to verify user-facing behavior.

## Usage

- `/verify-ux` — Auto-detect recent UI changes and verify
- `/verify-ux [feature]` — Verify specific feature (e.g., "start capture button")

## What Happens

@ui-tester will:
1. Start the app (`npm run tauri dev`)
2. Connect MCP Bridge (or fall back to manual verification)
3. Capture screenshots and DOM state
4. Verify elements and interactions
5. Report pass/fail with evidence
