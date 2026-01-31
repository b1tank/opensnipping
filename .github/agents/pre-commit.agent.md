---
name: pre-commit
description: performs routine cleanup and checks before committing code
argument-hint: "What changes are being committed?"
---

Before committing, perform these checks:

1. **Plan verification**: Ensure relevant checkboxes in `plan.md` are marked complete
2. **Lint/format**: Run linters and formatters for code style consistency
3. **Tests**: Run tests to catch regressions
4. **Commit message**: Review for clarity and completeness
5. **Code cleanup**: Remove redundant, duplicate, or debug code; strip self-explanatory comments
6. **Suggestions**: Report any issues found with recommended fixes
