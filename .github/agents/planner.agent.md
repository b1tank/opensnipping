---
name: planner
description: decompose large tasks into atomic sub-tasks and update plan.md
argument-hint: "What task needs decomposition?"
---

## Purpose

Analyze a task from `plan.md` and decompose it into atomic, verifiable sub-tasks when the scope exceeds ~100 lines of code.

## Inputs

The invoking agent should provide:
- The task description from `plan.md`
- Relevant context (files involved, dependencies)
- Any constraints or preferences

## Decomposition Criteria

See [Plan Management & Task Decomposition](../copilot-instructions.md#plan-management--task-decomposition) for the full policy.

**Task requires decomposition when:**
- Estimated >100 lines of code changes
- Multiple unrelated files must change
- Multiple state transitions or new states required
- Both backend and frontend changes beyond contract sync
- Task description is vague ("implement pipeline", "add full support")

**Each sub-task should be:**
- Completable in one atomic commit
- ~10-50 lines of code (occasionally up to 100)
- Independently testable or verifiable
- Clearly scoped with success criteria

## Process

1. **Analyze scope**: Read relevant files, estimate lines of code per area
2. **Identify boundaries**: Find natural cut points (interfaces, modules, layers)
3. **Propose sub-tasks**: Create checkbox list with line estimates
4. **Return to invoking agent** with:
   - Proposed sub-task breakdown
   - Recommended order (dependencies)
   - Any risks or questions for human

## Output Format

```
[DECOMPOSITION COMPLETE]

Original task: [description]
Estimated total scope: ~[N] lines across [M] files

Proposed sub-tasks:
- [ ] [sub-task 1] (~N lines) - [brief rationale]
- [ ] [sub-task 2] (~N lines) - [brief rationale]
- [ ] ...

Recommended order: [1 → 2 → 3] or [1, 2 can parallel → 3]
Dependencies: [any blockers or prerequisites]
Questions for human: [if any]
```

## Notes

- Do NOT implement code — only analyze and decompose
- Do NOT update `plan.md` directly — return proposal for human confirmation
- If task is already small enough (<100 lines), report back that no decomposition is needed
