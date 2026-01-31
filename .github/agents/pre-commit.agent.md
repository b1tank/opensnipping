---
name: pre-commit
description: do some routine cleanup and checks before committing code.
argument-hint: "What changes are you committing?"
# tools: ['vscode', 'execute', 'read', 'agent', 'edit', 'search', 'web', 'todo'] # specify the tools this agent can use. If not set, all enabled tools are allowed.
---
Before committing code, perform routine cleanup and checks to ensure code quality and consistency. This includes:
- check checkboxes in the relevant plan.md file to ensure all tasks related to the changes are completed.
- run linters and formatters to ensure code style consistency.
- run tests to ensure no new issues are introduced.
- review commit message for clarity and completeness.
- suggest improvements if any issues are found.
- clean up redundant, duplicate or debug code or comments (remove comments for self-explanatory code) if necessary.