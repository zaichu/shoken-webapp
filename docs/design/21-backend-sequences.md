# バックエンドのシーケンス

シーケンス図は、部品間の呼び出しと応答を上から下へ時系列で示す図。本資料はバックエンド側で追える処理を示し、ブラウザの実画面操作やGoogle側の内部処理は扱わない。
入口・共通middlewareは [層構造とルート](20-backend-structure.md)、分岐の詳細は [アクティビティ](22-backend-activity.md) を参照する。

## Google OAuth とセッション作成

| 用語 | 意味 |
| --- | --- |
| OAuth | OAuth は外部サービスでの認証結果を受け取る仕組み。 |
| Cookie・セッション | Cookie はブラウザが保存し、リクエスト時に送り返す値で、セッションはログイン状態をDBと照合する仕組み。 |
| state・PKCE・nonce | state は認証開始と結果を結び付ける値、PKCE は認可コードを盗まれても交換されにくくする検証方式、nonce は受け取るIDトークンを今回の認証に結び付ける値。 |

この図で分かること: 認証開始で短命Cookieを作り、callbackで照合してからGoogleへのコード交換とDB保存へ進む順序。正常に最後まで進む経路を示す。

```mermaid
sequenceDiagram
    participant B as ブラウザ
    participant H as auth/oauth handler
    participant A as services/auth
    participant G as Google
    participant D as DB
    B->>H: GET /api/v1/oauth/google/authorize
    H->>A: begin_auth
    A-->>H: 認可先・state・PKCE・nonce
    H-->>B: 短命Cookieを設定して認可先へリダイレクト
    Note over B,G: Googleでの操作は図の対象外
    B->>H: GET callback（code・state・Cookie）
    H->>H: state一致とPKCE・nonceの存在を確認
    H->>A: authenticate_with_google_code
    A->>G: コードとPKCEでIDトークンへ交換
    G-->>A: IDトークン
    A->>G: tokeninfoで検証情報を取得
    G-->>A: 発行者・宛先・期限・nonce等
    A->>A: クレーム検証
    A->>D: BEGIN・ユーザー追加または更新
    A->>D: 旧セッション等を削除・新セッションのハッシュを保存
    A->>D: COMMIT
    A-->>H: 新セッショントークン
    H-->>B: 短命Cookie削除・セッションCookie・フロントへリダイレクト
```

根拠:

- `backend/src/handlers/v1/auth/oauth.rs:99`（開始）、`backend/src/handlers/v1/auth/oauth.rs:140`（callbackの照合とCookie操作）。
- `backend/src/services/auth/google_common.rs:62`（認証開始用の値生成）、`backend/src/services/auth/google_common.rs:183`（クレーム検証）。
- `backend/src/services/auth/google_worker.rs:15`（交換）、`backend/src/services/auth/google_worker.rs:63`（tokeninfo）、`backend/src/services/auth/google_worker.rs:82`（呼出順序）。
- `backend/src/services/auth.rs:61`（認証）、`backend/src/services/auth.rs:107`（トランザクション）、`backend/src/services/auth.rs:153`（旧削除と新発行）、`backend/src/services/auth.rs:174`（ユーザー更新）。

| 用語 | 意味 |
| --- | --- |
| クレーム・検証 | クレームはIDトークンの発行者・宛先・期限などの属性。検証は `iss`・`aud`・`exp`・`nonce` と `sub`・`email` の存在を確認する。 |
| ハッシュ・セッション | ハッシュは元の値をそのまま保存せず照合に使う変換値で、セッションは SHA-256 のハッシュをDBに保存する（`backend/src/services/auth.rs:29`）。 |
| 短命Cookie・セッションCookie・HttpOnly | 短命Cookieは10分、セッションCookieは7日。どちらも `HttpOnly`（JavaScriptから読み取れない属性）を持ち、`secure_cookie` の値で `Secure`・`SameSite` を決める（`backend/src/handlers/v1/auth/oauth.rs:36`・`backend/src/handlers/v1/auth/oauth.rs:51`・`backend/src/handlers/v1/auth/oauth.rs:67`）。 |

state不一致や必須Cookie欠落は後続に進まず、Google検証失敗時はユーザー・セッションを作らない。図中のDB操作は一つのトランザクションである。

## 認証付きAPIの1リクエスト

この図で分かること: 配当一覧を例に、Cookieの認証が成功してから本人のデータを検索する順序。共通middlewareを通過した後を描く。

```mermaid
sequenceDiagram
    participant B as API利用側
    participant E as AuthenticatedUser
    participant A as services/auth
    participant H as dividends::list
    participant S as 共通検索service
    participant D as DB
    B->>E: GET /api/v1/dividends と Cookie
    E->>E: CookieからSessionTokenを解析
    E->>A: select_user_by_session
    A->>D: ハッシュ一致・期限内のsessionとuserを検索
    D-->>A: ユーザーまたは該当なし
    A-->>E: 照合結果
    alt 有効なユーザー
        E->>H: AuthenticatedUserを引数に渡す
        H->>S: user_idと検索条件
        S->>D: 本人に絞った件数・一覧等のクエリ
        D-->>S: 検索結果
        S-->>H: ページ付きレスポンス
        H-->>B: 200 JSON
    else Cookie不正・期限切れ・検証失敗
        E-->>B: 401（handlerは実行しない）
    end
```

根拠:

- `backend/src/extractors/auth.rs:20`（Cookie解析・検証・拒否）、`backend/src/services/auth.rs:119`（期限とハッシュの照合）。
- `backend/src/handlers/v1/dividends.rs:47`、`backend/src/handlers/common.rs:17`（共通検索への引数）。
- `backend/src/services/domain/search.rs:63`・`backend/src/services/domain/search.rs:168`、`backend/src/services/domain/search_filters.rs:199`（本人と検索条件による絞り込み）。

Cookie欠落・不正はDB照会より前に401になる。extractor内のDBエラーも401へ変換されるため、図の失敗側はDBに到達しない場合も含む。

## CSV取込の2リクエスト

CSV は表をカンマ区切りのテキストにした形式。multipart はファイルと項目を一緒に送るHTTPの形式。

この図で分かること: プレビューと確定取込は別リクエストで、確定時にもファイルを読み直すこと。両方とも認証を通過した配当CSVの正常系を示す。

```mermaid
sequenceDiagram
    participant B as API利用側
    participant H as CSV handler
    participant C as CsvImport / pipeline
    participant S as dividend / bulk service
    participant D as DB
    B->>H: POST dividend-import-validations（multipart）
    H->>H: file項目と.csv拡張子を確認
    H->>C: preview_csv（バイト列）
    C->>C: デコード・パース・行変換
    C-->>H: 有効行・行エラー
    H-->>B: 200 プレビュー（保存なし）
    B->>H: POST dividend-imports（multipart）
    H->>C: upload_csv（本人・上限・バイト列）
    C->>C: 再パース・行変換・保存前検証
    C->>S: bulk_create（有効行）
    S->>D: BEGIN・利用者ロック・上限確認・一括挿入・COMMIT
    D-->>S: 挿入件数
    S-->>C: inserted・skipped
    C-->>H: 件数と行エラー
    H->>D: 取込履歴を追加または更新
    H-->>B: 201 取込結果
```

根拠:

- `backend/src/handlers/v1/dividends.rs:94`（検証）、`backend/src/handlers/v1/dividends.rs:114`（取込）、`backend/src/handlers/v1/csv_import.rs:12`・`backend/src/handlers/v1/csv_import.rs:34`・`backend/src/handlers/v1/csv_import.rs:52`（共通処理）。
- `backend/src/services/csv/pipeline.rs:31`、`backend/src/services/csv/import.rs:48`・`backend/src/services/csv/import.rs:62`（パースと変換）。
- `backend/src/services/dividend.rs:115`、`backend/src/services/domain/bulk.rs:162`・`backend/src/services/domain/bulk.rs:176`（追記）、`backend/src/services/csv.rs:11`（履歴）。

配当CSVは有効行を追記し重複をスキップする。全行エラーで挿入・スキップとも0なら履歴更新を省略する。保有CSVの全置換と拒否条件は [アクティビティ](22-backend-activity.md) を参照。
履歴更新は取込のCOMMIT後に行う別操作であり、全体を一つのトランザクションとは描いていない。

## 一覧の検索とページ送り

| 用語 | 意味 |
| --- | --- |
| ページ送り・LIMIT・OFFSET | ページ送りは結果を指定件数ずつ取り出すこと。`LIMIT` は取得件数、`OFFSET` は先頭から読み飛ばす件数。 |
| summary・facets | summary は検索全体の集計、facets は絞り込み候補を表す。 |

この図で分かること: 同じ検索条件で件数とページ内の行を取得し、次ページは新しいGETとして処理すること。配当一覧の認証後を例とする。

```mermaid
sequenceDiagram
    participant B as API利用側
    participant H as list / handle_search
    participant S as domain::search
    participant D as DB
    B->>H: GET dividends（q・page・per_page等）
    H->>S: user_id・検索パラメータ
    S->>S: 日付条件検証・語の分割・ページ値を決定
    par 検索条件全体
        S->>D: COUNT（本人と検索条件）
        D-->>S: total
    and ページ内
        S->>D: SELECT（同条件・固定順序・LIMIT・OFFSET）
        D-->>S: data
    and 任意の追加情報
        opt include_summary または include_facets
            S->>D: 指定された集計・候補
            D-->>S: summary・facets
        end
    end
    S-->>H: data・total・page・per_page・任意の追加情報
    H-->>B: 200 JSON
    B->>H: 次ページのGET（同条件・pageを変更）
    Note over H,D: 毎回認証し、同じ検索処理を再実行
```

根拠:

- `backend/src/handlers/v1/dividends.rs:47`、`backend/src/handlers/common.rs:17`（入口）。
- `backend/src/services/domain/search.rs:46`・`backend/src/services/domain/search.rs:157`（検証・並行取得・レスポンス）。
- `backend/src/services/domain/search_filters.rs:166`・`backend/src/services/domain/search_filters.rs:199`（語の分割・条件）、`backend/src/services/dividend.rs:69`（決済日降順・ID降順）。
- `shared/src/common.rs:28`（ページ値）、`shared/src/common.rs:86`（集計・候補は既定false）、`shared/src/common.rs:155`（応答型）。

pageは既定1・下限1、per_pageは既定200・1〜1000に収める。OFFSETは `(page - 1) * per_page`。語の間はAND、1語の検索対象列の間はORで絞る。集計と候補にはページのLIMITをかけない。

## 未確認

- Googleの同意画面の内部処理、実際のCookieの送受信、デプロイ済み環境での一連の通信は実行していない。
- 同時にデータが増減した場合に、別クエリの件数・一覧が同一時点の状態になることは保証として確認していない。

## 文書との食い違い

- [architecture.md](../architecture.md) のCSV概要は4種類を例示する。現行ルートには投資信託保有の検証・取込もあるため、本資料のルート一覧では5種類を示す（`backend/src/handlers/v1.rs:85`・`backend/src/handlers/v1.rs:113`）。
