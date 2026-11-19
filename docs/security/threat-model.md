# 脅威モデル

## 守るもの

- 利用者の資産データ(保有数量・金額・銘柄・口座・取引日・配当)
- 識別子(users.email・google_id)と認証秘密(sessions.token_hash)
- 集計結果の正しさ(税・損益の再計算を含む)
- サービスの可用性(CSV 取込・cron・検索)

## 攻撃者

- ネットワーク上の第三者(セッション奪取・CSRF・なりすまし)
- 他の利用者(他人の user_id への水平アクセス)
- 悪意ある CSV・拡張機能・OAuth リダイレクトを触る者
- 依存・CI 経由の侵入者(サプライチェーン)
- DB に直接触れる者(管理者・漏洩した接続情報)は別境界とする

## 信頼の境界

```mermaid
flowchart LR
  subgraph Browser[ブラウザ Pages]
    FE[Leptos CSR]
  end
  subgraph Edge[Cloudflare]
    W[Workers Axum]
    RL[レート制限]
  end
  subgraph IdP[Google]
    O[OAuth tokeninfo]
  end
  subgraph Data[Supabase Postgres]
    AW[app_worker 接続]
    AR[app_restricted 利用者Tx]
    EX[限定関数 executor]
    UT[(利用者表 RLS)]
    ST[(共有表)]
  end
  FE -->|HTTPS Cookie| W
  W --> RL
  W -->|code検証| O
  W -->|app_worker| AW
  AW -->|app_enterで切替| AR
  AR -->|user_id一致のみ| UT
  AW -->|SELECTのみ| ST
  AW -->|EXECUTEのみ| EX
```

境界: (1)ブラウザ-Workers 間は Cookie+Origin、(2)Workers-Google 間は tokeninfo 信頼、(3)Workers-DB 間は app_worker(NOBYPASSRLS・表の所有者ロールに所属しない)で接続し、利用者の要求では Tx 内で app_enter により app_restricted に切り替えて RLS(user_id = app.current_user_id)を効かせる。共有表は SELECT のみ、利用者 ID が定まる前の認証・利用者をまたぐ処理・共有表の書き込みは引数と戻りを絞った限定関数(executor ロールの SECURITY DEFINER)に閉じ込め EXECUTE のみを与える。FORCE ROW LEVEL SECURITY は付けない(docs/security/db-rls-plan.md §28)。本番の接続確認は #2017(11/18 の cron の DB access ログ)。構成の正本は docs/security/db-rls-plan.md §25・§28、backend/migrations/README.md「ビューと新しい表の RLS の決まり」。

## ログインの流れ

```mermaid
sequenceDiagram
  participant U as 利用者
  participant FE as Pages
  participant W as Workers
  participant G as Google
  U->>W: GET oauth authorize
  W->>U: state/PKCE/nonce Cookie+302
  U->>G: 認可+同意
  G->>U: code+state
  U->>W: GET callback
  W->>W: state照合・Cookie破棄
  W->>G: code+verifier交換・tokeninfo
  G->>W: sub/email/nonce/iss/aud/exp
  W->>W: upsert+旧セッション削除
  W->>U: session_token Cookie 7日
```
