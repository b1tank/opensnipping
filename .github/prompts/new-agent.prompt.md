---
name: new-agent
description: Create a new agent role (like hiring a team member). Invokes @lead to design the agent spec.
---

You want to create a new agent role. Think of it like hiring a new team member.

Invoke **@lead** to help design the agent:

## Provide Role Description

Example: "I need a devops agent to handle CI/CD and deployments"

## @lead Will Help Define

1. **Responsibilities**: What does this agent do?
2. **Decisions**: What can it decide autonomously vs needs approval?
3. **Skills**: What existing skills should it reference?
4. **Triggers**: When should this agent be invoked?
5. **Placement**: Shareable (`~/skills/agents/`) or repo-specific (`.github/agents/`)?

## Agent Design Pattern

The new agent file will follow this structure:

```markdown
---
name: [agent-name]
description: [one-line description of role and when to invoke]
---

## Purpose
[What this agent does]

## Responsibilities
- [responsibility 1]
- [responsibility 2]

## Workflow
1. [step]
2. [step]

## Guidelines
- [guideline]
```

## Output

A new `.agent.md` file created after your confirmation.

## Examples of Agents You Might Create

- `devops` — CI/CD pipelines, deployment automation
- `infra` — Cloud hosting, monitoring, scaling
- `analyst` — Code archaeology, "how does X work?"
- `security` — Security review, vulnerability scanning
- `docs` — Documentation, API references, READMEs
