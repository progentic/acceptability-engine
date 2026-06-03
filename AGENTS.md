# AGENTS.md

## Scope

This file applies to the entire repository unless a more specific AGENTS.md exists in a child directory.

## Required coding standard

Follow `docs/CODING_STYLE.md` for all code changes.

Before editing code:
- Read `docs/CODING_STYLE.md`.
- Apply its rules to new code, modified code, tests, examples, and scripts.
- Do not introduce style exceptions unless the repository already documents an explicit exception.

## Platform requirements

Code must be multi-platform.

Do not tailor behavior to the current local development host. Avoid assumptions about:
- absolute local paths
- usernames
- shell-specific behavior
- macOS-only, Linux-only, or Windows-only commands
- locally installed tools not declared by the project
- machine-specific environment variables
- network access
- filesystem case sensitivity
- path separators

Use portable APIs and project-relative paths. When platform-specific behavior is unavoidable, isolate it behind a small abstraction, document why it is required, and cover it with tests or clear validation logic.

## Change discipline

Prefer small, focused changes.

Do not mix unrelated refactors, formatting churn, dependency changes, and behavior changes in one task.

Preserve existing public behavior unless the task explicitly requires changing it.

## Validation

Run the relevant project checks before reporting completion.

At minimum, use the checks documented by the repository. If no project-specific validation exists, run the narrowest available formatting, linting, and test commands for the files changed.

Report:
- what changed
- what validation ran
- any validation that could not be run and why