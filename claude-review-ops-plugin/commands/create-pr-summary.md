---
description: Draft a Japanese PR title and body from the current branch diff and test results
argument-hint: [base-branch]
---

# Create PR Summary

Draft a concise Japanese PR title and body for the current branch.

## Context

Use this right before `gh pr create` or when the PR description needs to be refreshed.

## Instructions

1. Inspect the current branch name, recent commits, and diff against `$ARGUMENTS` or `main`.
2. Extract only user-facing or reviewer-relevant changes.
3. Build:
   - a short Japanese PR title
   - a body with `変更内容` and `テスト`
4. If tests were not run, state that explicitly instead of inventing results.
5. Keep the body tight enough to paste directly into `gh pr create` or `gh pr edit`.

## Arguments

- `$ARGUMENTS`: Optional base branch. Default is `main`.
