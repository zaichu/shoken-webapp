---
name: pr-review
description: |
  Claude の実装完了後に、Codex へコードレビューを依頼する運用を標準化する。
  Use when: 「PRレビューをCodexに回したい」「Claude作業後のレビュー依頼文を作りたい」
  「Claude→Codexレビュー運用にしたい」と依頼された時。
---

# PR Review Handoff

## Overview
Claude が実装を終えたら、このスキルでレビュー材料を固定フォーマットに整理し、Codex へレビュー依頼する。  
Codex は findings first で重大度順レビューを返す前提で依頼文を作成する。

## Workflow

1. 実装完了状態を確定する。
- `git status -sb` で差分が意図どおりか確認する。
- レビュー対象コミット/差分を特定する。
- PR がある場合は PR 番号と URL を控える。

2. レビュー材料を収集する。
- 必ず以下を取得する。
  - `git diff --name-status`
  - `git diff --stat`
  - `git diff`
- PR がある場合は以下も取得する。
  - `gh pr view <PR番号> --json number,title,url,body`
  - `gh pr view <PR番号> --comments`
- PR がない場合は `main...develop` の差分を明記する。

3. 検証結果を確定する。
- 変更範囲に応じて `lint` / `test` / `build` を実行する。
- 実行コマンドと結果（pass/fail）をそのまま記録する。
- 失敗していても隠さず依頼文に含める。

4. Codex への依頼文を作成する。
- `references/codex-review-request-template.md` を読み、必須項目を埋める。
- 曖昧語を使わない。
- レビュー観点（バグ、回帰、テスト不足、設計リスク）を明示する。

5. Codex レビュー結果を取り込む。
- 指摘を重大度順に処理する。
- 修正後に同じ検証コマンドを再実行する。
- 必要に応じて再レビューを Codex に依頼する。

## Review Request Rules
- Codex への依頼文には必ず対象範囲を入れる。
  - PR URL または比較範囲（例: `main...develop`）
- Codex への依頼文には必ず検証結果を入れる。
  - 実行したコマンドと pass/fail
- Codex への依頼文には必ず期待する出力形式を入れる。
  - findings first
  - 重大度順
  - ファイルパスと行番号

## Output
Codex へ渡す最終依頼文を Markdown で返す。

## Reference
- 依頼テンプレートは `references/codex-review-request-template.md` を使う。
