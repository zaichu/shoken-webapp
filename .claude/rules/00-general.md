# 全体開発ルール

## 言語・表現

- 日本語で回答する
- コードコメントは日本語
- コミットメッセージは日本語
- 変数名・関数名は英語

## 開発フロー（標準）

### 役割分担
- **Claude**: 設計・タスク定義・コードレビュー・マージ判断
- **Codex**: 実装・テスト・コミット・push・PR 作成

### 標準フロー
1. Claude がタスクを設計し `docs/tasks/<branch-name>.md` を作成する
2. Claude が `.claude/skills/claude-codex-handoff/SKILL.md` で Codex に実装を委譲する
3. Codex が実装・lint/test/build・commit・push・PR 作成を行う
4. Claude が `/pr-review` スキルで PR をレビューし、GitHub にコメントを投稿する
5. **修正が必要な場合**: Claude が `claude-codex-handoff` で指摘一覧（コメント ID 付き）を Codex に渡す
6. **Codex が指摘への対応をする**: 各指摘コメントに返信 → 修正実装 → lint/test/build → push（PR 作成は不要）
7. Claude が再レビュー（ステップ 4 に戻る）
8. LGTM → Claude がマージ

**重要: ステップ 5〜7 は LGTM が出るまで繰り返す。返信なし・未修正の指摘が 1 件でも残ればマージ禁止。**

### タスク管理ルール
- `docs/tasks/<branch-name>.md` が存在する場合は、作業前に必ず読み、進捗とレビュー指摘を更新する
- 中規模以上のタスクでは `docs/tasks/TEMPLATE.md` を元に task file を作成する
- task file はローカルの一時ファイルとして扱い、ユーザー明示指示がない限りコミット・PR に含めない
- task 完了時または作業中止時には、対応する task file を削除する
- backend の API 契約変更時は `bash scripts/check-openapi.sh` を実行して `docs/openapi.json` と `frontend/src/generated/api.ts` を同期する

### Codex が遵守するルール
- 実装後は必ず lint/test/build を通してからコミットする
- push 後に PR を作成し、Claude のレビューを待つ
- Claude からレビュー指摘が来たら、**まず各指摘コメントに返信し、その後修正して再 push する**
  - 返信なしで修正だけするのは禁止
  - 修正しない場合（スコープ外など）も「対応しない理由」を必ずコメントに返信する
- CodeRabbit 等の自動レビューコメントにも同様に返信する
- Codex から Claude に設計相談・調査依頼する場合は `.claude/skills/codex-claude-handoff/SKILL.md` を使用する

## 出力制約

- 思考過程・内部推論・検討ログは出力しない
- 出力は必要最小限。前置きや冗長な説明は禁止
- 指示されていない改善・設計提案・リファクタは禁止
- 指定がない限り返答は 200 tokens 以内
- 長文が必要な場合は事前に確認する
