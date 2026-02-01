# Agent & Skill Setup

AI agent configuration for this repository. For the full architecture reference, see [~/skills/ARCHITECTURE.md](https://github.com/b1tank/skills/blob/main/ARCHITECTURE.md).

## Quick Reference

### Prompts (triggers)

| Prompt | Purpose | Invokes |
|--------|---------|---------|
| `/new-project` | Start new project from one-liner idea | `@product` |
| `/new-agent` | Create a new agent role | `@lead` |
| `/work-on-next` | Pick up next task from plan.md | `@lead` |
| `/self-improve` | Improve agent instructions | (meta) |

### Agents (team roles)

| Agent | Role |
|-------|------|
| `@product` | Product designer — drafts spec.md from ideas |
| `@lead` | Tech lead — orchestrates work, generates plan.md |
| `@engineer` | Software engineer — dedicated implementation |
| `@reviewer` | Code reviewer — critical "grill me" feedback |

### Skills (procedures)

| Skill | Description |
|-------|-------------|
| `decompose-task` | Break large tasks into atomic sub-tasks |
| `diff-check` | Author's cleanup before commit/PR |
| `tauri-contract` | TS ⇄ Rust contract sync (repo-specific) |

---

## Usage

### Start a new project
```
/new-project
> "A lightweight screen recorder like GNOME Screencast"
```

### Work on next task
```
/work-on-next
```

### Get code reviewed
Invoke `@reviewer` with your changes or PR URL.

### Improve instructions
```
/self-improve
```
After making a mistake that should be prevented, ask to update instructions.

---

## Syncing & Maintenance

### Install/Update Global Skills
```bash
npx skills add b1tank/skills
```

### Sync Agents & Prompts from ~/skills
```bash
cd ~/skills && ./sync.sh ~/opensnipping
```

### Add a New Repo-Specific Skill
Create in `.github/skills/<name>/SKILL.md`

### Improve Shareable Skills/Agents
1. Edit files in `~/skills/`
2. Push: `cd ~/skills && git add . && git commit -m "..." && git push`
3. Re-sync to repos: `./sync.sh <target-repo>`

---

## File Structure

```
.github/
├── copilot-instructions.md   # Repo-wide rules (source of truth)
├── README.md                 # This file
├── agents/                   # Synced from ~/skills + repo-specific
├── prompts/                  # Synced from ~/skills + repo-specific
├── skills/                   # Repo-specific only
│   └── tauri-contract/
└── archive/                  # Old agents (reference)

AGENTS.md (root)              # → symlink to .github/copilot-instructions.md
.claude/CLAUDE.md             # → symlink to ../.github/copilot-instructions.md
```

---

## Links

- [Golden Architecture Guide](https://github.com/b1tank/skills/blob/main/ARCHITECTURE.md) — Full reference for agent/skill/prompt design
- [b1tank/skills](https://github.com/b1tank/skills) — Shareable skills, agents, prompts
