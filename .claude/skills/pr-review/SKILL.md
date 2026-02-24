---
name: pr-review
description: |
  Claude の実装完了後に、発行済みPRを自動検出してレビュー開始する運用を標準化する。
  Use when: 「PRレビューを開始したい」「Claude作業後にそのままレビューしたい」
  「レビュー依頼文を作りたい」と依頼された時。
---

# PR Review Handoff

## Overview
Claude が実装を終えたら、このスキルで `gh` から発行済み PR を自動検出し、  
差分と検証結果を集めて **そのままレビューを開始** する。

依頼文作成モードは補助機能として残し、  
ユーザーが「依頼文を作って」と明示した場合のみ使う。

## Workflow

1. レビュー対象PRを自動検出する。
- `git branch --show-current` で現在ブランチを確認する。
- まず `gh pr list --state open --head <current-branch>` で open PR を探す。
- 見つからない場合のみ `gh pr list --state open --base main --head develop` を試す。
- それでも見つからない場合は、ユーザーに PR 番号 or URL を確認する。

2. 実装完了状態を確定する。
- `git status -sb` で差分が意図どおりか確認する。
- レビュー対象コミット/差分を特定する。
- PR 番号と URL を確定する。

3. レビュー材料を収集する。
- 必ず以下を取得する。
  - `git diff --name-status`
  - `git diff --stat`
  - `git diff`
- PR がある場合は以下も取得する。
  - `gh pr view <PR番号> --json number,title,url,body`
  - `gh pr view <PR番号> --comments`
- PR がない場合は `main...develop` の差分を明記する。

4. 検証結果を確定する。
- 変更範囲に応じて `lint` / `test` / `build` を実行する。
- 実行コマンドと結果（pass/fail）をそのまま記録する。
- 失敗していても隠さず依頼文に含める。

5. レビューを実行する（デフォルト）。
- findings first で重大度順に出す。
- `path:line` を付ける。
- 期待フォーマット:
  1. Findings（重大度順、`path:line` 付き）
  2. Open questions / assumptions
  3. 修正方針サマリー（短く）

6. 依頼文作成モード（明示要求時のみ）。
- `references/codex-review-request-template.md` を読み、必須項目を埋める。
- 曖昧語を使わない。
- レビュー観点（バグ、回帰、テスト不足、設計リスク）を明示する。

7. レビュー結果を取り込む。
- 指摘を重大度順に処理する。
- 修正後に同じ検証コマンドを再実行する。
- 必要に応じて再レビューを Codex に依頼する。

## Review Request Rules
- レビュー結果には必ず対象範囲を入れる。
  - PR URL または比較範囲（例: `main...develop`）
- レビュー結果には必ず検証結果を入れる。
  - 実行したコマンドと pass/fail
- findings first / 重大度順 / ファイルパスと行番号を必須とする。

## Output
- デフォルト: レビュー結果（findings first）。
- 例外: ユーザーが依頼文作成を明示した場合のみ、Codex へ渡す依頼文を Markdown で返す。

## Reference
- 依頼テンプレートは `references/codex-review-request-template.md` を使う。
