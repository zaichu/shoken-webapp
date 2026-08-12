---
name: review-implementing
description: |
  CodeRabbit、GitHub review、Codex review、ユーザー指摘などのレビューコメントを確認し、妥当性判断、修正、返信、再検証まで行う。Use when: レビュー指摘が貼られた時、PR コメントを確認して対応するとき、CodeRabbit の actionable comment がある時、マージ前に未対応レビューを潰す時。
---

# Review Implementing

## 原則

- レビュー指摘は必ず読む。確認せずにマージしない。
- 指摘ごとに「対応する」「対応しない」を判断し、理由を残す。
- 妥当な指摘は最小差分で直す。
- 不要または過剰な指摘は、なぜ採用しないかを PR コメントで説明する。
- コメント投稿は `--body-file` を使い、実改行で投稿する。

## Workflow

1. PR 番号または現在ブランチを確認する。
2. `gh pr view <PR> --json comments,reviews,latestReviews,reviewDecision,mergeStateStatus` を読む。
3. inline comments は `gh api repos/<owner>/<repo>/pulls/<PR>/comments` で読む。
4. 各指摘を分類する。
   - valid: 実装する。
   - already-fixed: 現在差分では解消済み。
   - out-of-scope: 別 Issue にする。
   - disagree: 理由を PR に残す。
5. valid な指摘だけ実装する。Claude 委譲が必要なら `codex-claude-handoff` を使う。
6. 変更範囲に応じた lint/test/build を実行する。
7. PR に対応コメントを残す。
8. checks と review 状態を再確認してからマージ可否を判断する。

## 判断基準

- README と実装がズレている場合は、実装追加か README 修正のどちらが今回の目的に合うかで決める。
- CI に重い検証を足す場合は、検出したい失敗とコストが釣り合うか確認する。
- 「別タスクで検討」と判断したら `issue-task-lifecycle` を使って Issue を作る。
- 指摘対応で unrelated refactor を混ぜない。

## PR コメント

一時ファイルに本文を書いて投稿する。

```bash
gh pr comment <PR> --body-file /tmp/<topic>-review-response.md
```

inline reply API が使えない場合は、通常 PR コメントで対象指摘と対応方針を明記する。
