---
name: verify-ux
description: Start the app locally and verify UX works based on user input or latest features.
---

Launch the app and verify user-facing behavior works correctly.

## Start the App

```bash
cd opensnipping && npm run tauri dev
```

Wait for the app window to appear before proceeding.

## Determine What to Verify

**Option A: User specifies feature**
- Verify the exact feature/flow mentioned

**Option B: Latest changes (default)**
- Check recent commits: `git log --oneline -5`
- Identify user-facing changes (UI, interactions, state transitions)
- Skip if only backend/test/doc changes with no visible effect

## Verification Steps

For each feature to verify:

1. **Print clear steps** for the user to follow:
   ```
   Verification: [Feature Name]
   1. [Action to take]
   2. [Expected result]
   3. [Action to take]
   4. [Expected result]
   ```

2. **Wait for user confirmation**:
   - ✓ Works as expected
   - ✗ Issue found (describe)

3. **If issues found**:
   - Log the problem
   - Offer to create a fix task or investigate immediately

## After Verification

- Summarize what was verified
- Note any issues to address
- Suggest next steps (commit, fix, or continue development)
