---
name: work-on-next
description: work on the next atomic task from the plan
---

Using [plan](../../opensnipping/plan.md), identify the next atomic task. The task should be logically cohesive and verifiable by test or manual check.

## Before Starting (MANDATORY)

### 1. Parallel Analysis
- Review 2-4 upcoming atomic tasks for safe parallelization
- If any can run in parallel, provide a ready-to-copy prompt for a separate agent (with context and clear spec)

### 2. Task Decomposition Check
- **Estimate lines of code** for the selected task
- **If >100 lines**: Use `runSubagent` to invoke planner—see [Task Decomposition Workflow](../agents/engineer.agent.md#task-decomposition-workflow)
- Wait for human confirmation before proceeding with decomposed sub-tasks

## Execute Task

1. Implement
2. Compile
3. Test

## Before Committing (MANDATORY)

1. **Run pre-commit subagent** via `runSubagent`—see [Pre-Commit Subagent](../agents/engineer.agent.md#pre-commit-subagent-via-runsubagent)
2. Address any findings
3. Commit per [Commit and Push Policy](../copilot-instructions.md#commit-and-push-policy)
