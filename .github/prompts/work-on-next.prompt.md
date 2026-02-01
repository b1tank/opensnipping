---
name: work-on-next
description: Work on the next atomic task from the plan. Invokes @lead for orchestration.
---

Using the project's `plan.md`, identify and execute the next atomic task.

## Invoke @lead

The **@lead** agent will:

1. **Identify next task** from plan.md
2. **Check task size** — if >100 lines, decompose using `decompose-task` skill
3. **Delegate or execute**:
   - Simple task → @lead handles directly
   - Complex task → delegates to @engineer

## Before Starting (MANDATORY)

### 1. Parallel Analysis
- Review 2-4 upcoming tasks for safe parallelization
- If any can run in parallel, provide a ready-to-copy prompt for a separate @engineer window

### 2. Task Decomposition Check
- Estimate lines of code
- If >100 lines: invoke `decompose-task` skill
- Wait for human confirmation before proceeding

## Execute Task

1. Implement changes
2. Compile/build
3. Run tests

## Before Committing (MANDATORY)

1. Run `diff-check` skill for cleanup validation
2. Address any findings
3. Commit with clear message: `[category]: description`

## After Completion

@lead will:
- Update plan.md checkboxes
- Offer to continue or suggest new session
- Identify next parallelizable work
