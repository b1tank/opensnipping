---
name: self-improve
description: Improve agent/skill/prompt instructions in this repo. Meta-level nudge for setup refinement.
---

Improve the instructions for agents, skills, and prompts. Keep instructions concise and clear.

## What to Review

### Global (shareable)
- `~/.copilot/skills/` — installed skills
- `~/skills/` — source repo for shareable content

### Repo-Specific
- `.github/copilot-instructions.md` — repo-wide rules
- `.github/agents/*.agent.md` — agent definitions
- `.github/prompts/*.prompt.md` — prompt triggers
- `.github/skills/` — repo-specific skills

## Guidelines

### When Adding Guidelines
- Deduplicate across files (use reference links)
- Unless context-specific repetition is valuable
- Keep each file focused on its role

### What Belongs Where

| Content | Location |
|---------|----------|
| Repo-wide rules | `copilot-instructions.md` |
| Agent behavior | `agents/*.agent.md` |
| Reusable procedures | `skills/` |
| Quick triggers | `prompts/*.prompt.md` |

### Improvement Triggers

Consider improvements when:
- You catch yourself repeating the same correction
- A common mistake keeps happening
- You discover a better pattern
- Context is missing that would help

### After Corrections

Ask: "Should I update instructions so this doesn't happen again?"

Update the appropriate file:
- Behavioral preference → `copilot-instructions.md`
- Agent-specific → relevant `.agent.md`
- Procedural → convert to skill if reusable
