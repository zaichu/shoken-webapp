---
description: Run the project PR review loop from the current branch and require re-review after fixes
argument-hint: [pr-number]
---

# PR Review Loop

Run the project's standard PR review loop for the current branch.

## Context

Use this command after implementation is done, before merge, or after review fixes were pushed.

## Instructions

1. Detect the current branch and associated PR. If `$ARGUMENTS` is present, treat it as the PR number.
2. If no PR exists yet, explain that this command should be run after PR creation and stop.
3. Run the project-standard review entrypoint:
   - Prefer `codex exec review`
   - If that is unavailable, use `codex exec "/pr-review してください"`
4. Summarize the findings with `findings first`.
5. If findings exist, tell the user that fixes must be pushed and the same review loop must be run again.
6. If no findings exist, state that re-review is no longer required for the current diff.

## Arguments

- `$ARGUMENTS`: Optional PR number to review explicitly.

## Notes

- Do not replace Codex CLI review with ad-hoc manual review.
- Re-review is mandatory after every fix push until findings are cleared.
