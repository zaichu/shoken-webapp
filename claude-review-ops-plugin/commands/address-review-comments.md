---
description: Resolve GitHub PR review comments on the current branch and prepare a minimal fix plan
argument-hint: [pr-number]
---

# Address Review Comments

Collect open review comments for the current PR, group them by fix area, and prepare an implementation plan.

## Context

Use this when a PR already exists and the user asks to handle reviewer comments.

## Instructions

1. Identify the current PR. If `$ARGUMENTS` is present, treat it as the PR number.
2. Verify GitHub authentication before attempting to fetch comments.
3. Retrieve unresolved review comments and issue comments relevant to code changes.
4. Group comments into:
   - must-fix correctness issues
   - contract / typing issues
   - tests / verification gaps
   - low-priority cleanup
5. Produce a minimal implementation plan that preserves existing behavior unless the comment explicitly asks for behavior change.
6. After fixes are made, instruct the user to run `/review-ops:pr-review-loop`.

## Arguments

- `$ARGUMENTS`: Optional PR number.

## Notes

- Keep fixes scoped to the comment intent.
- Prefer concrete reproduction and validation commands over broad suggestions.
