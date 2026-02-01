---
name: new-project
description: Start a new project from a one-liner idea. Invokes @product to draft spec.md.
---

You want to start a new project. Invoke the **@product** agent to help you:

1. **Provide your one-liner idea** (e.g., "A lightweight screen recorder like GNOME Screencast")

2. **Answer clarifying questions** about:
   - Target platform(s)
   - Key features and differentiators
   - UX reference points
   - Explicit non-goals

3. **Review market research** on competitors/alternatives

4. **Iterate on spec.md** until satisfied

The @product agent will:
- Ask questions in small batches
- Research existing products using the `market-research` skill
- Draft spec.md using the `spec-template` skill (9-section format)
- Help you define what NOT to build (non-goals)

## Output

A complete `spec.md` file ready to hand off to @lead for plan.md generation.

## Next Steps

After spec.md is approved:
- Run `/work-on-next` to invoke @lead for plan generation
- Or manually invoke `@lead` with the spec
