//! PostgreSQL アクセス層。ホスト（migrate bin・DB 統合テスト）は deadpool-postgres、
//! wasm (Workers) は Hyperdrive 経由の tokio-postgres を内部に持ち、サービス層は同一の API を使う。

mod bind;
mod builder;
#[cfg(not(target_arch = "wasm32"))]
mod migrate;
pub mod numeric;

pub use bind::Bind;
pub use builder::QueryBuilder;
#[cfg(not(target_arch = "wasm32"))]
pub use migrate::run_migrations;
pub use numeric::Numeric;
pub use tokio_postgres::Row;
pub use tokio_postgres::error::SqlState;

use postgres_types::{FromSql, ToSql, Type};
use std::pin::Pin;
use std::sync::Arc;

type DbFuture<'a, T> = Pin<Box<dyn std::future::Future<Output = T> + Send + 'a>>;

#[cfg(not(target_arch = "wasm32"))]
fn db_future<'a, T>(fut: impl std::future::Future<Output = T> + Send + 'a) -> DbFuture<'a, T> {
    Box::pin(fut)
}

#[cfg(target_arch = "wasm32")]
fn db_future<'a, T>(fut: impl std::future::Future<Output = T> + 'a) -> DbFuture<'a, T> {
    // wasm32 はシングルスレッドのため Send 化は安全。axum の handler が
    // `Future: Send` を要求するため `Rc<Client>` を跨ぐ future を包む
    Box::pin(worker::send::SendFuture::new(fut))
}

/// DB 実行エラー。`sql_state()` で PostgreSQL の SQLSTATE を取り出せる
#[derive(Debug)]
pub enum DbError {
    /// tokio-postgres が返すエラー（接続・クエリ実行・デコード）
    Pg(tokio_postgres::Error),
    /// 行が見つからない（fetch_one 相当）
    RowNotFound,
    /// プール取得失敗・接続設定など Postgres 以外の失敗
    Other(String),
    /// SQLSTATE を直接保持するエラー（Worker 経路・テスト用）
    State { state: SqlState, message: String },
}

impl DbError {
    pub fn sql_state(&self) -> Option<SqlState> {
        match self {
            DbError::Pg(e) => e.as_db_error().map(|e| e.code().clone()),
            DbError::State { state, .. } => Some(state.clone()),
            DbError::RowNotFound | DbError::Other(_) => None,
        }
    }

    pub fn is_unique_violation(&self) -> bool {
        self.sql_state()
            .is_some_and(|s| s == SqlState::UNIQUE_VIOLATION)
    }

    pub fn is_foreign_key_violation(&self) -> bool {
        self.sql_state()
            .is_some_and(|s| s == SqlState::FOREIGN_KEY_VIOLATION)
    }
}

impl std::fmt::Display for DbError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DbError::Pg(e) => write!(f, "{e}"),
            DbError::RowNotFound => f.write_str("no rows returned by a query that expected one"),
            DbError::Other(m) => f.write_str(m),
            DbError::State { message, .. } => f.write_str(message),
        }
    }
}

impl std::error::Error for DbError {}

impl From<tokio_postgres::Error> for DbError {
    fn from(e: tokio_postgres::Error) -> Self {
        DbError::Pg(e)
    }
}

/// クエリを実行できる相手。`Db`（プール/共有クライアント）と `Tx`（トランザクション）
/// の双方に実装し、`&Db`/`&mut Tx` でも呼べるようにする
pub trait Executor {
    fn query_rows<'a>(
        &'a self,
        sql: &'a str,
        params: &'a [Bind],
    ) -> DbFuture<'a, Result<Vec<Row>, DbError>>;
    fn execute<'a>(
        &'a self,
        sql: &'a str,
        params: &'a [Bind],
    ) -> DbFuture<'a, Result<u64, DbError>>;
    /// `BEGIN`/`COMMIT` や複数文（migration）など、パラメータなしの一括実行
    fn batch_execute<'a>(&'a self, sql: &'a str) -> DbFuture<'a, Result<(), DbError>>;
}

/// `query_typed`/`execute_typed` に渡す「値 + 宣言型」のペア列。
/// Hyperdrive（caching 無効）では `query()`/`execute()` の untyped prepare が
/// 1接続2回目の呼び出しで接続を desync/Closed にするため、型を明示して
/// Parse+Bind+Execute を1往復に固める typed 経路を native/wasm 共通で使う
fn typed_params(params: &[Bind]) -> Vec<(&(dyn ToSql + Sync), Type)> {
    params
        .iter()
        .map(|p| (p as &(dyn ToSql + Sync), p.declared_type()))
        .collect()
}

/// ログ識別用に SQL を1行・先頭のみに整形する。SQL は静的リテラルで値は
/// `$n` バインド経由のため、クエリ本文の出力は秘密値を含まない
fn query_tag(sql: &str) -> String {
    sql.split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .chars()
        .take(80)
        .collect()
}

/// DB 実行失敗を、どのクエリがなぜ失敗したか分かる形で残す（sqlstate・サーバー
/// メッセージ・クエリ識別タグ）。wasm は tracing subscriber を持たないため console へ
fn db_err(op: &'static str, sql: &str, e: tokio_postgres::Error) -> DbError {
    let e = DbError::Pg(e);
    let sqlstate = e
        .sql_state()
        .map(|s| s.code().to_string())
        .unwrap_or_else(|| "-".to_string());
    let tag = query_tag(sql);
    #[cfg(target_arch = "wasm32")]
    worker::console_error!("DB {op} 失敗 sqlstate={sqlstate} sql=`{tag}`: {e}");
    #[cfg(not(target_arch = "wasm32"))]
    tracing::error!(sqlstate, sql = tag, error = %e, "DB {op} 失敗");
    e
}

/// `Row` → モデル変換。`sqlx::FromRow` 相当を手書き impl で提供する
pub trait FromRow: Sized {
    fn from_row(row: &Row) -> Result<Self, DbError>;
}

/// 組み立て済みクエリ。`db::query()` または `QueryBuilder::build*()` で作る
pub struct Query {
    pub(crate) sql: String,
    pub(crate) params: Vec<Bind>,
}

pub fn query(sql: impl Into<String>, params: Vec<Bind>) -> Query {
    Query {
        sql: sql.into(),
        params,
    }
}

/// `sqlx::query_as` 相当。戻り行は `T::from_row` で変換する
pub fn query_as<T: FromRow>(sql: impl Into<String>, params: Vec<Bind>) -> QueryAs<T> {
    QueryAs::new(query(sql, params))
}

/// `sqlx::query_scalar` 相当（先頭行の先頭カラムを読む）
pub fn query_scalar<S: for<'a> FromSql<'a>>(
    sql: impl Into<String>,
    params: Vec<Bind>,
) -> QueryScalar<S> {
    QueryScalar::new(query(sql, params))
}

/// 単一カラム行をタプルとして読む（テスト・補助クエリ用）
impl<T0> FromRow for (T0,)
where
    T0: for<'a> FromSql<'a>,
{
    fn from_row(row: &Row) -> Result<Self, DbError> {
        Ok((row.try_get::<_, T0>(0).map_err(DbError::Pg)?,))
    }
}

/// 2カラム行をタプルとして読む
impl<T0, T1> FromRow for (T0, T1)
where
    T0: for<'a> FromSql<'a>,
    T1: for<'a> FromSql<'a>,
{
    fn from_row(row: &Row) -> Result<Self, DbError> {
        Ok((
            row.try_get::<_, T0>(0).map_err(DbError::Pg)?,
            row.try_get::<_, T1>(1).map_err(DbError::Pg)?,
        ))
    }
}

/// `query_as!` 相当。`Query` に行変換型を乗せたもの
pub struct QueryAs<T> {
    inner: Query,
    _marker: std::marker::PhantomData<fn() -> T>,
}

/// `query_scalar!` 相当（先頭行の先頭カラムを読む）
pub struct QueryScalar<S> {
    inner: Query,
    _marker: std::marker::PhantomData<fn() -> S>,
}

impl Query {
    pub async fn execute<E: Executor>(&self, executor: E) -> Result<u64, DbError> {
        executor.execute(&self.sql, &self.params).await
    }

    pub async fn fetch_all<T: FromRow, E: Executor>(&self, executor: E) -> Result<Vec<T>, DbError> {
        let rows = executor.query_rows(&self.sql, &self.params).await?;
        rows.iter().map(T::from_row).collect()
    }

    pub async fn fetch_optional<T: FromRow, E: Executor>(
        &self,
        executor: E,
    ) -> Result<Option<T>, DbError> {
        let rows = executor.query_rows(&self.sql, &self.params).await?;
        rows.first().map(T::from_row).transpose()
    }

    pub async fn fetch_one<T: FromRow, E: Executor>(&self, executor: E) -> Result<T, DbError> {
        self.fetch_optional(executor)
            .await?
            .ok_or(DbError::RowNotFound)
    }

    /// 先頭行の先頭カラム。行が無ければ RowNotFound
    pub async fn fetch_scalar<S, E>(&self, executor: E) -> Result<S, DbError>
    where
        S: for<'a> FromSql<'a>,
        E: Executor,
    {
        self.fetch_scalar_optional(executor)
            .await?
            .ok_or(DbError::RowNotFound)
    }

    pub async fn fetch_scalar_optional<S, E>(&self, executor: E) -> Result<Option<S>, DbError>
    where
        S: for<'a> FromSql<'a>,
        E: Executor,
    {
        let rows = executor.query_rows(&self.sql, &self.params).await?;
        rows.first()
            .map(|row| row.try_get::<_, S>(0).map_err(DbError::from))
            .transpose()
    }
}

impl<T: FromRow> QueryAs<T> {
    pub(crate) fn new(inner: Query) -> Self {
        Self {
            inner,
            _marker: std::marker::PhantomData,
        }
    }

    pub async fn fetch_all<E: Executor>(&self, executor: E) -> Result<Vec<T>, DbError> {
        self.inner.fetch_all::<T, E>(executor).await
    }

    pub async fn fetch_one<E: Executor>(&self, executor: E) -> Result<T, DbError> {
        self.inner.fetch_one::<T, E>(executor).await
    }

    pub async fn fetch_optional<E: Executor>(&self, executor: E) -> Result<Option<T>, DbError> {
        self.inner.fetch_optional::<T, E>(executor).await
    }
}

impl<S> QueryScalar<S>
where
    S: for<'a> FromSql<'a>,
{
    pub(crate) fn new(inner: Query) -> Self {
        Self {
            inner,
            _marker: std::marker::PhantomData,
        }
    }

    pub async fn fetch_one<E: Executor>(&self, executor: E) -> Result<S, DbError> {
        self.inner.fetch_scalar::<S, E>(executor).await
    }

    pub async fn fetch_optional<E: Executor>(&self, executor: E) -> Result<Option<S>, DbError> {
        self.inner.fetch_scalar_optional::<S, E>(executor).await
    }
}

/// コネクションプール（ホスト）/ 共有クライアント（wasm）のハンドル。
/// `Clone` は内部ハンドルの共有のみ
#[derive(Clone)]
pub struct Db {
    inner: Arc<DbInner>,
}

impl std::fmt::Debug for Db {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Db").finish_non_exhaustive()
    }
}

enum DbInner {
    #[cfg(not(target_arch = "wasm32"))]
    Pool(deadpool_postgres::Pool),
    #[cfg(target_arch = "wasm32")]
    Worker(WorkerDb),
}

/// `DATABASE_URL` の `sslmode` に応じた TLS 検証方針（libpq と同じ意味）。
/// Supabase の Session pooler は独自 CA のチェーンを提示するため、OS の
/// 信頼ストアだけでは検証できない。検証する運用ではダッシュボード配布の
/// CA を `sslrootcert` で足し、できない運用では `sslmode=require` を選ぶ。
#[cfg(not(target_arch = "wasm32"))]
#[derive(Debug, PartialEq)]
enum TlsMode {
    /// `disable`・`prefer`・未指定。システムルートで検証する（従来通り）
    System,
    /// `verify-ca`・`verify-full`。システムルートに加え `sslrootcert` の CA も信頼する
    WithExtraCa(String),
    /// `require`。libpq と同じく暗号化のみで検証はしない
    Unverified,
}

/// `sslmode` は後勝ち（tokio-postgres の上書きと同じ向き）
#[cfg(not(target_arch = "wasm32"))]
fn tls_mode_from_url(url: &str) -> Result<TlsMode, String> {
    let parsed =
        url::Url::parse(url).map_err(|_| "DATABASE_URL の解析に失敗しました".to_string())?;
    let mut sslmode = "prefer";
    let mut sslrootcert = None;
    for (key, value) in parsed.query_pairs() {
        if key == "sslmode" {
            sslmode = match value.as_ref() {
                "disable" | "prefer" => "prefer",
                "require" => "require",
                "verify-ca" | "verify-full" => "verify",
                _ => continue,
            };
        } else if key == "sslrootcert" && !value.is_empty() {
            sslrootcert = Some(value.into_owned());
        }
    }
    match sslmode {
        "require" => Ok(TlsMode::Unverified),
        "verify" => match sslrootcert {
            Some(path) => Ok(TlsMode::WithExtraCa(path)),
            None => Err("sslmode=verify-ca/verify-full には sslrootcert が必要です".to_string()),
        },
        _ => Ok(TlsMode::System),
    }
}

/// PEM ファイルから証明書をすべて読み込む（Dashboard 配布の CA など複数入りに対応）
#[cfg(not(target_arch = "wasm32"))]
fn load_pem_certs(path: &str) -> Result<Vec<native_tls::Certificate>, String> {
    let bytes = std::fs::read(path)
        .map_err(|e| format!("sslrootcert の CA ファイルを読み込めませんでした: {e}"))?;
    let text = String::from_utf8(bytes)
        .map_err(|_| "sslrootcert の CA ファイルが PEM 形式ではありません".to_string())?;
    const BEGIN: &str = "-----BEGIN CERTIFICATE-----";
    const END: &str = "-----END CERTIFICATE-----";
    let mut certs = Vec::new();
    let mut rest = text.as_str();
    while let Some(start) = rest.find(BEGIN) {
        let after_begin = start + BEGIN.len();
        let end = rest[after_begin..]
            .find(END)
            .map(|i| after_begin + i + END.len())
            .ok_or_else(|| "sslrootcert の CA ファイルが PEM 形式ではありません".to_string())?;
        certs.push(
            native_tls::Certificate::from_pem(&rest.as_bytes()[start..end])
                .map_err(|e| format!("sslrootcert の CA 証明書を読み込めませんでした: {e}"))?,
        );
        rest = &rest[end..];
    }
    if certs.is_empty() {
        return Err("sslrootcert の CA ファイルに証明書が含まれていません".to_string());
    }
    Ok(certs)
}

#[cfg(not(target_arch = "wasm32"))]
fn build_tls_connector(mode: &TlsMode) -> Result<postgres_native_tls::MakeTlsConnector, String> {
    let mut builder = native_tls::TlsConnector::builder();
    match mode {
        TlsMode::System => {}
        TlsMode::WithExtraCa(path) => {
            for cert in load_pem_certs(path)? {
                builder.add_root_certificate(cert);
            }
        }
        TlsMode::Unverified => {
            builder.danger_accept_invalid_certs(true);
        }
    }
    let connector = builder
        .build()
        .map_err(|e| format!("TLS コネクタの初期化に失敗しました: {e}"))?;
    Ok(postgres_native_tls::MakeTlsConnector::new(connector))
}

#[cfg(not(target_arch = "wasm32"))]
pub fn connect_pool_lazy(database_url: &str, max_connections: u32) -> Result<Db, String> {
    let database_url = database_url.trim();
    validate_database_url(database_url)?;
    let tls_mode = tls_mode_from_url(database_url)?;
    let sanitized_url = sanitize_database_url(database_url)?;
    let config = sanitized_url
        .parse::<tokio_postgres::Config>()
        .map_err(|e| format!("DB 接続設定の解析に失敗しました: {e}"))?;
    let tls = build_tls_connector(&tls_mode)?;
    let manager = deadpool_postgres::Manager::from_config(
        config,
        tls,
        deadpool_postgres::ManagerConfig {
            // sqlx の test_before_acquire に近い「使う前に生存確認」を選ぶ
            recycling_method: deadpool_postgres::RecyclingMethod::Verified,
        },
    );
    let pool = deadpool_postgres::Pool::builder(manager)
        .max_size(max_connections as usize)
        .runtime(deadpool_postgres::Runtime::Tokio1)
        .build()
        .map_err(|e| format!("DB pool の初期化に失敗しました: {e}"))?;
    Ok(Db {
        inner: Arc::new(DbInner::Pool(pool)),
    })
}

#[cfg(target_arch = "wasm32")]
struct WorkerDb {
    connection_string: String,
    host: String,
    port: u16,
    // クライアントは Db 単位（= リクエスト単位）で使い回す。Socket は生成した
    // リクエストの I/O コンテキストに紐付くため、isolate 永続のキャッシュは
    // できない。Client は wasm では Clone できないため Rc で共有する
    client:
        worker::send::SendWrapper<std::cell::RefCell<Option<std::rc::Rc<tokio_postgres::Client>>>>,
}

#[cfg(target_arch = "wasm32")]
impl Db {
    /// Hyperdrive バインディングから DB ハンドルを作る。接続自体は初回クエリ時に張る
    pub fn from_hyperdrive(hyperdrive: &worker::Hyperdrive) -> Self {
        Db {
            inner: Arc::new(DbInner::Worker(WorkerDb {
                connection_string: hyperdrive.connection_string(),
                host: hyperdrive.host(),
                port: hyperdrive.port(),
                client: worker::send::SendWrapper::new(std::cell::RefCell::new(None)),
            })),
        }
    }
}

#[cfg(target_arch = "wasm32")]
mod worker_db {
    use super::{DbError, WorkerDb};
    use std::rc::Rc;
    use tokio_postgres::Client;

    pub(super) async fn connect(
        connection_string: &str,
        host: &str,
        port: u16,
    ) -> Result<Client, DbError> {
        let socket = worker::Socket::builder()
            .secure_transport(worker::SecureTransport::StartTls)
            .connect(host, port)
            .map_err(|e| DbError::Other(format!("socket connect 失敗: {e}")))?;
        let config = connection_string
            .parse::<tokio_postgres::Config>()
            .map_err(|e| DbError::Other(format!("Hyperdrive 接続文字列の解析失敗: {e}")))?;
        let (client, connection) = config
            .connect_raw(socket, worker::postgres_tls::PassthroughTls)
            .await
            .map_err(DbError::Pg)?;
        wasm_bindgen_futures::spawn_local(async move {
            if let Err(e) = connection.await {
                worker::console_error!("DB connection task error: {e:?}");
            }
        });
        Ok(client)
    }

    impl WorkerDb {
        /// 同一リクエスト内で共有するクライアントを返す。
        /// トランザクション用途では `fresh_client` を使う
        pub(super) async fn client(&self) -> Result<Rc<Client>, DbError> {
            if let Some(client) = self.client.borrow().as_ref()
                && !client.is_closed()
            {
                return Ok(client.clone());
            }
            let client = Rc::new(connect(&self.connection_string, &self.host, self.port).await?);
            *self.client.borrow_mut() = Some(client.clone());
            Ok(client)
        }

        /// トランザクション用の独立接続（共有クライアント上で BEGIN すると
        /// 他クエリと混線するため、tx は接続を分ける）
        pub(super) async fn fresh_client(&self) -> Result<Client, DbError> {
            connect(&self.connection_string, &self.host, self.port).await
        }
    }
}

impl Db {
    pub async fn close(&self) {
        match &*self.inner {
            #[cfg(not(target_arch = "wasm32"))]
            DbInner::Pool(pool) => pool.close(),
            #[cfg(target_arch = "wasm32")]
            DbInner::Worker(_) => {}
        }
    }

    /// トランザクションを開始する。`BEGIN` は batch_execute で送り、
    /// commit/rollback は `Tx` が担う
    pub async fn begin(&self) -> Result<Tx, DbError> {
        match &*self.inner {
            #[cfg(not(target_arch = "wasm32"))]
            DbInner::Pool(pool) => {
                let obj = pool
                    .get()
                    .await
                    .map_err(|e| DbError::Other(format!("pool からの接続取得に失敗: {e}")))?;
                let guard = TxGuard::new(TxConn::Native(obj));
                guard.conn().batch_execute("BEGIN").await?;
                Ok(Tx {
                    conn: Some(guard.disarm()),
                })
            }
            #[cfg(target_arch = "wasm32")]
            DbInner::Worker(db) => {
                let client = db.fresh_client().await?;
                let guard = TxGuard::new(TxConn::Wasm(client));
                guard.conn().batch_execute("BEGIN").await?;
                Ok(Tx {
                    conn: Some(guard.disarm()),
                })
            }
        }
    }
}

impl Executor for Db {
    fn query_rows<'a>(
        &'a self,
        sql: &'a str,
        params: &'a [Bind],
    ) -> DbFuture<'a, Result<Vec<Row>, DbError>> {
        db_future(async move {
            match &*self.inner {
                #[cfg(not(target_arch = "wasm32"))]
                DbInner::Pool(pool) => {
                    let obj = pool
                        .get()
                        .await
                        .map_err(|e| DbError::Other(format!("pool からの接続取得に失敗: {e}")))?;
                    obj.query_typed(sql, &typed_params(params))
                        .await
                        .map_err(|e| db_err("query", sql, e))
                }
                #[cfg(target_arch = "wasm32")]
                DbInner::Worker(db) => {
                    let client = db.client().await?;
                    client
                        .query_typed(sql, &typed_params(params))
                        .await
                        .map_err(|e| db_err("query", sql, e))
                }
            }
        })
    }

    fn execute<'a>(
        &'a self,
        sql: &'a str,
        params: &'a [Bind],
    ) -> DbFuture<'a, Result<u64, DbError>> {
        db_future(async move {
            match &*self.inner {
                #[cfg(not(target_arch = "wasm32"))]
                DbInner::Pool(pool) => {
                    let obj = pool
                        .get()
                        .await
                        .map_err(|e| DbError::Other(format!("pool からの接続取得に失敗: {e}")))?;
                    obj.execute_typed(sql, &typed_params(params))
                        .await
                        .map_err(|e| db_err("execute", sql, e))
                }
                #[cfg(target_arch = "wasm32")]
                DbInner::Worker(db) => {
                    let client = db.client().await?;
                    client
                        .execute_typed(sql, &typed_params(params))
                        .await
                        .map_err(|e| db_err("execute", sql, e))
                }
            }
        })
    }

    fn batch_execute<'a>(&'a self, sql: &'a str) -> DbFuture<'a, Result<(), DbError>> {
        db_future(async move {
            match &*self.inner {
                #[cfg(not(target_arch = "wasm32"))]
                DbInner::Pool(pool) => {
                    let obj = pool
                        .get()
                        .await
                        .map_err(|e| DbError::Other(format!("pool からの接続取得に失敗: {e}")))?;
                    obj.batch_execute(sql)
                        .await
                        .map_err(|e| db_err("batch", sql, e))
                }
                #[cfg(target_arch = "wasm32")]
                DbInner::Worker(db) => {
                    let client = db.client().await?;
                    client
                        .batch_execute(sql)
                        .await
                        .map_err(|e| db_err("batch", sql, e))
                }
            }
        })
    }
}

impl Executor for &Db {
    fn query_rows<'a>(
        &'a self,
        sql: &'a str,
        params: &'a [Bind],
    ) -> DbFuture<'a, Result<Vec<Row>, DbError>> {
        (*self).query_rows(sql, params)
    }

    fn execute<'a>(
        &'a self,
        sql: &'a str,
        params: &'a [Bind],
    ) -> DbFuture<'a, Result<u64, DbError>> {
        (*self).execute(sql, params)
    }

    fn batch_execute<'a>(&'a self, sql: &'a str) -> DbFuture<'a, Result<(), DbError>> {
        (*self).batch_execute(sql)
    }
}

/// トランザクション。`BEGIN`/`COMMIT`/`ROLLBACK` は batch_execute で実装している
/// （deadpool/tokio-postgres の Transaction は借用ベースで所有できないため）。
/// commit せず drop された場合は非同期で ROLLBACK を送る
pub struct Tx {
    conn: Option<TxConn>,
}

enum TxConn {
    #[cfg(not(target_arch = "wasm32"))]
    Native(deadpool_postgres::Object),
    #[cfg(target_arch = "wasm32")]
    Wasm(tokio_postgres::Client),
}

impl TxConn {
    async fn batch_execute(&self, sql: &str) -> Result<(), DbError> {
        match self {
            #[cfg(not(target_arch = "wasm32"))]
            TxConn::Native(obj) => obj
                .batch_execute(sql)
                .await
                .map_err(|e| db_err("batch", sql, e)),
            #[cfg(target_arch = "wasm32")]
            TxConn::Wasm(client) => client
                .batch_execute(sql)
                .await
                .map_err(|e| db_err("batch", sql, e)),
        }
    }

    /// 開いたかもしれないトランザクションを ROLLBACK で確実に閉じてから
    /// 接続を手放す（ホストはプール返却、wasm は drop）。spawn できない
    /// 場合は接続をプールから切り離して破棄し、サーバー側でロールバックさせる
    fn rollback_and_release(self) {
        match self {
            #[cfg(not(target_arch = "wasm32"))]
            TxConn::Native(obj) => {
                if let Ok(handle) = tokio::runtime::Handle::try_current() {
                    handle.spawn(async move {
                        if let Err(e) = obj.batch_execute("ROLLBACK").await {
                            tracing::error!(error = %e, "DB ロールバック失敗");
                        }
                        drop(obj);
                    });
                } else {
                    drop(deadpool_postgres::Object::take(obj));
                }
            }
            #[cfg(target_arch = "wasm32")]
            TxConn::Wasm(client) => {
                wasm_bindgen_futures::spawn_local(async move {
                    if let Err(e) = client.batch_execute("ROLLBACK").await {
                        worker::console_error!("DB ロールバック失敗: {e}");
                    }
                    drop(client);
                });
            }
        }
    }
}

/// BEGIN/COMMIT/ROLLBACK の await 中に呼び出し Future がキャンセルされると、
/// 開いたトランザクションを残した接続がそのままプールへ戻り、次の借用者の
/// クエリが他人のトランザクション内で実行される。完了確認まで接続を握って
/// おくガード。Drop 時は ROLLBACK を送ってから接続を手放す
struct TxGuard(Option<TxConn>);

impl TxGuard {
    fn new(conn: TxConn) -> Self {
        TxGuard(Some(conn))
    }

    fn conn(&self) -> &TxConn {
        self.0.as_ref().expect("トランザクションの接続がありません")
    }

    /// トランザクション状態が確定した接続を取り出してガードを解除する
    fn disarm(mut self) -> TxConn {
        self.0.take().expect("トランザクションの接続がありません")
    }
}

impl Drop for TxGuard {
    fn drop(&mut self) {
        if let Some(conn) = self.0.take() {
            conn.rollback_and_release();
        }
    }
}

impl Tx {
    pub async fn commit(mut self) -> Result<(), DbError> {
        let conn = self
            .conn
            .take()
            .expect("トランザクションの接続がありません");
        let guard = TxGuard::new(conn);
        guard.conn().batch_execute("COMMIT").await?;
        drop(guard.disarm());
        Ok(())
    }

    /// 明示ロールバック。`Drop` の非同期 ROLLBACK と違い完了を待てる
    pub async fn rollback(mut self) -> Result<(), DbError> {
        let conn = self
            .conn
            .take()
            .expect("トランザクションの接続がありません");
        let guard = TxGuard::new(conn);
        guard.conn().batch_execute("ROLLBACK").await?;
        drop(guard.disarm());
        Ok(())
    }
}

impl Drop for Tx {
    fn drop(&mut self) {
        // 接続が残っている = commit/rollback 未完了で drop された = ロールバック要。
        // 接続をプールへ戻す前に ROLLBACK を送る（送れない環境では接続ごと破棄される）
        if let Some(conn) = self.conn.take() {
            conn.rollback_and_release();
        }
    }
}

macro_rules! impl_executor_for_tx {
    ($ty:ty) => {
        impl Executor for $ty {
            fn query_rows<'a>(
                &'a self,
                sql: &'a str,
                params: &'a [Bind],
            ) -> DbFuture<'a, Result<Vec<Row>, DbError>> {
                db_future(async move {
                    match self
                        .conn
                        .as_ref()
                        .expect("トランザクションの接続がありません")
                    {
                        #[cfg(not(target_arch = "wasm32"))]
                        TxConn::Native(obj) => obj
                            .query_typed(sql, &typed_params(params))
                            .await
                            .map_err(|e| db_err("query", sql, e)),
                        #[cfg(target_arch = "wasm32")]
                        TxConn::Wasm(client) => client
                            .query_typed(sql, &typed_params(params))
                            .await
                            .map_err(|e| db_err("query", sql, e)),
                    }
                })
            }

            fn execute<'a>(
                &'a self,
                sql: &'a str,
                params: &'a [Bind],
            ) -> DbFuture<'a, Result<u64, DbError>> {
                db_future(async move {
                    match self
                        .conn
                        .as_ref()
                        .expect("トランザクションの接続がありません")
                    {
                        #[cfg(not(target_arch = "wasm32"))]
                        TxConn::Native(obj) => obj
                            .execute_typed(sql, &typed_params(params))
                            .await
                            .map_err(|e| db_err("execute", sql, e)),
                        #[cfg(target_arch = "wasm32")]
                        TxConn::Wasm(client) => client
                            .execute_typed(sql, &typed_params(params))
                            .await
                            .map_err(|e| db_err("execute", sql, e)),
                    }
                })
            }

            fn batch_execute<'a>(&'a self, sql: &'a str) -> DbFuture<'a, Result<(), DbError>> {
                db_future(async move {
                    self.conn
                        .as_ref()
                        .expect("トランザクションの接続がありません")
                        .batch_execute(sql)
                        .await
                })
            }
        }
    };
}

impl_executor_for_tx!(&Tx);
impl_executor_for_tx!(&mut Tx);

#[cfg(not(target_arch = "wasm32"))]
use url::Url;

/// `tokio_postgres::Config` へ渡す前に `channel_binding`・`sslrootcert` の
/// query parameter を除去し、`sslmode=verify-ca/verify-full` を `require` に
/// 読み替える（tokio-postgres が解釈できない値を先に正規化する。検証の有無は
/// `tls_mode_from_url` が決め、ここでは TLS を必須にすることだけ伝える）。
/// 上記がなければ入力文字列をそのまま返す。
/// 保持する query pair は raw 文字列を再利用し percent-encoding を変換しない。
#[cfg(not(target_arch = "wasm32"))]
fn sanitize_database_url(url: &str) -> Result<String, String> {
    let parsed = Url::parse(url).map_err(|_| "DATABASE_URL の解析に失敗しました".to_string())?;

    if !parsed.query_pairs().any(|(k, v)| {
        k == "channel_binding"
            || k == "sslrootcert"
            || (k == "sslmode" && (v == "verify-ca" || v == "verify-full"))
    }) {
        return Ok(url.to_string());
    }

    // raw pair と decoded pair を zip し、decoded key が除去・読替対象の raw pair を処理する
    let filtered_query = parsed
        .query()
        .map(|q| {
            let raw_pairs: Vec<&str> = q.split('&').collect();
            let decoded_pairs: Vec<_> = parsed.query_pairs().collect();
            raw_pairs
                .iter()
                .zip(decoded_pairs.iter())
                .filter_map(|(raw, (k, v))| {
                    if k == "channel_binding" || k == "sslrootcert" {
                        None
                    } else if k == "sslmode" && (v == "verify-ca" || v == "verify-full") {
                        Some("sslmode=require")
                    } else {
                        Some(*raw)
                    }
                })
                .collect::<Vec<_>>()
                .join("&")
        })
        .unwrap_or_default();

    let fragment = parsed
        .fragment()
        .map(|f| format!("#{f}"))
        .unwrap_or_default();
    let base = url.find('?').map_or(url, |pos| &url[..pos]);
    if filtered_query.is_empty() {
        Ok(format!("{base}{fragment}"))
    } else {
        Ok(format!("{base}?{filtered_query}{fragment}"))
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn validate_database_url(url: &str) -> Result<(), String> {
    if url.is_empty() {
        return Err("DATABASE_URL が空です".to_string());
    }
    let lower = url.to_lowercase();
    if !lower.starts_with("postgres://") && !lower.starts_with("postgresql://") {
        return Err(
            "DATABASE_URL のスキームが不正です（postgres:// または postgresql:// が必要です）"
                .to_string(),
        );
    }
    Ok(())
}

#[cfg(all(test, not(target_arch = "wasm32")))]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_connect_pool_lazy() {
        let database_url = "postgresql://user:password@localhost/test_db";
        assert!(connect_pool_lazy(database_url, 5).is_ok());
        assert!(connect_pool_lazy(database_url, 10).is_ok());
    }

    #[tokio::test]
    async fn test_connect_pool_lazy_invalid_url() {
        let err = connect_pool_lazy("", 5).unwrap_err();
        assert!(err.contains("空"), "空URLを拒否すること: {err}");
        let err = connect_pool_lazy("   ", 5).unwrap_err();
        assert!(err.contains("空"), "空白のみは空として拒否: {err}");

        let err = connect_pool_lazy("/relative", 5).unwrap_err();
        assert!(
            !err.contains("/relative"),
            "相対URLをエラーに含めない: {err}"
        );

        let err = connect_pool_lazy("http://example.com/db", 5).unwrap_err();
        assert!(
            !err.contains("example.com"),
            "エラーにホスト名を含めない: {err}"
        );
    }

    #[tokio::test]
    async fn test_connect_pool_lazy_sanitizes_channel_binding() {
        let url = "postgresql://user:password@localhost/test_db?channel_binding=require";
        assert!(
            connect_pool_lazy(url, 5).is_ok(),
            "channel_binding 付きでも pool 作成できる"
        );
    }

    #[test]
    fn test_validate_database_url_valid() {
        assert!(validate_database_url("postgres://user:password@localhost/db").is_ok());
        assert!(validate_database_url("postgresql://user:password@localhost/db").is_ok());
        assert!(validate_database_url("POSTGRES://localhost/db").is_ok());
    }

    #[test]
    fn test_sanitize_database_url_removes_only_channel_binding_param() {
        // channel_binding だけを除去し、query が空になる
        let url = "postgres://localhost/db?channel_binding=require";
        let sanitized = sanitize_database_url(url).unwrap();
        let parsed = url::Url::parse(&sanitized).unwrap();
        let params: Vec<_> = parsed.query_pairs().collect();
        assert!(
            params.iter().all(|(k, _)| k != "channel_binding"),
            "channel_binding が残っている"
        );
        assert!(
            parsed.query().is_none_or(|q| q.is_empty()),
            "query が空でない: {:?}",
            parsed.query()
        );
    }

    #[test]
    fn test_sanitize_database_url_preserves_other_params() {
        // sslmode など他の query parameter は保持する
        let url = "postgres://localhost/db?sslmode=require&channel_binding=require";
        let sanitized = sanitize_database_url(url).unwrap();
        let parsed = url::Url::parse(&sanitized).unwrap();
        let params: Vec<_> = parsed.query_pairs().collect();
        assert!(
            params.iter().any(|(k, _)| k == "sslmode"),
            "sslmode が除去されている"
        );
        assert!(
            params.iter().all(|(k, _)| k != "channel_binding"),
            "channel_binding が残っている"
        );
    }

    #[test]
    fn test_sanitize_database_url_preserves_percent_encoding() {
        // %20 が + などに変換されず raw encoding のまま保持される
        let url =
            "postgres://localhost/db?application_name=shoken%20backend&channel_binding=require";
        let sanitized = sanitize_database_url(url).unwrap();
        assert!(
            sanitized.contains("application_name=shoken%20backend"),
            "%20 が + などに変換されている"
        );
        assert!(
            !sanitized.contains("channel_binding"),
            "channel_binding が残っている"
        );
    }

    #[test]
    fn test_sanitize_database_url_no_change_without_channel_binding() {
        // channel_binding がない URL は入力文字列をそのまま返す
        let url = "postgres://localhost/db?sslmode=require&application_name=shoken%20backend";
        let sanitized = sanitize_database_url(url).unwrap();
        assert!(sanitized == url, "入力文字列が変化した");
    }

    #[test]
    fn test_sanitize_database_url_removes_percent_encoded_key() {
        // decoded key が channel_binding になる percent-encoded raw key も除去される
        // %5F は '_' なので channel%5Fbinding は channel_binding に decode される
        let url = "postgres://localhost/db?channel%5Fbinding=require&sslmode=require";
        let sanitized = sanitize_database_url(url).unwrap();
        assert!(
            !sanitized.contains("channel"),
            "percent-encoded channel_binding key が残っている"
        );
        assert!(
            sanitized.contains("sslmode=require"),
            "sslmode が除去されている"
        );
    }

    #[test]
    fn test_sanitize_database_url_preserves_fragment() {
        // fragment は channel_binding 除去後も保持される
        let url = "postgres://localhost/db?sslmode=require&channel_binding=require#frag";
        let sanitized = sanitize_database_url(url).unwrap();
        assert!(sanitized.ends_with("#frag"), "fragment が消えている");
        assert!(
            !sanitized.contains("channel_binding"),
            "channel_binding が残っている"
        );
        assert!(
            sanitized.contains("sslmode=require"),
            "sslmode が除去されている"
        );
    }

    #[test]
    fn test_sanitize_database_url_invalid_url_error_safe() {
        // invalid URL のエラーに URL 本体・user・password・host を含まない
        let err = sanitize_database_url("not-a-valid-url").unwrap_err();
        assert!(
            !err.contains("not-a-valid-url"),
            "エラーに URL 値を含めない: {err}"
        );
    }

    #[test]
    fn test_sanitize_database_url_removes_sslrootcert() {
        // sslrootcert は tokio-postgres が解釈できないため除去する
        let url = "postgres://localhost/db?sslmode=require&sslrootcert=/tmp/ca.crt";
        let sanitized = sanitize_database_url(url).unwrap();
        assert!(
            !sanitized.contains("sslrootcert"),
            "sslrootcert が残っている"
        );
        assert!(
            sanitized.contains("sslmode=require"),
            "sslmode が除去されている"
        );
    }

    #[test]
    fn test_sanitize_database_url_rewrites_verify_sslmode() {
        // tokio-postgres が解釈できない verify-ca/verify-full は require に読み替える
        for mode in ["verify-ca", "verify-full"] {
            let url = format!("postgres://localhost/db?sslmode={mode}&application_name=a#frag");
            let sanitized = sanitize_database_url(&url).unwrap();
            assert!(
                sanitized.contains("sslmode=require"),
                "読み替えられていない: {sanitized}"
            );
            assert!(!sanitized.contains(mode), "元の値が残っている");
            assert!(
                sanitized.contains("application_name=a"),
                "他の parameter が消えている"
            );
            assert!(sanitized.ends_with("#frag"), "fragment が消えている");
        }
    }

    #[test]
    fn test_tls_mode_from_url_defaults_to_system() {
        // 未指定・prefer・disable は従来通りシステムルートで検証する
        for url in [
            "postgres://localhost/db",
            "postgres://localhost/db?sslmode=prefer",
            "postgres://localhost/db?sslmode=disable",
        ] {
            assert_eq!(tls_mode_from_url(url).unwrap(), TlsMode::System);
        }
    }

    #[test]
    fn test_tls_mode_from_url_require_skips_verification() {
        // require は libpq と同じく暗号化のみで検証はしない
        let mode = tls_mode_from_url("postgres://localhost/db?sslmode=require").unwrap();
        assert_eq!(mode, TlsMode::Unverified);
    }

    #[test]
    fn test_tls_mode_from_url_verify_needs_rootcert() {
        // verify-ca/verify-full は sslrootcert なしでは構築できない
        for mode in ["verify-ca", "verify-full"] {
            let err =
                tls_mode_from_url(&format!("postgres://localhost/db?sslmode={mode}")).unwrap_err();
            assert!(err.contains("sslrootcert"), "案内がない: {err}");
        }
        let mode = tls_mode_from_url(
            "postgres://localhost/db?sslmode=verify-full&sslrootcert=/tmp/ca.crt",
        )
        .unwrap();
        assert_eq!(mode, TlsMode::WithExtraCa("/tmp/ca.crt".to_string()));
    }

    #[test]
    fn test_tls_mode_from_url_last_sslmode_wins() {
        // 複数指定は後勝ち（tokio-postgres の上書きと同じ向き）
        let mode =
            tls_mode_from_url("postgres://localhost/db?sslmode=disable&sslmode=require").unwrap();
        assert_eq!(mode, TlsMode::Unverified);
    }

    #[test]
    fn test_connect_pool_lazy_verify_without_rootcert_fails() {
        // sslrootcert なしの verify-full はプール構築前に失敗する
        let err =
            connect_pool_lazy("postgresql://localhost/db?sslmode=verify-full", 1).unwrap_err();
        assert!(err.contains("sslrootcert"), "案内がない: {err}");
        assert!(!err.contains("localhost"), "ホストを漏らさない: {err}");
    }

    #[test]
    fn test_build_tls_connector_system_and_unverified() {
        assert!(build_tls_connector(&TlsMode::System).is_ok());
        assert!(build_tls_connector(&TlsMode::Unverified).is_ok());
    }

    #[test]
    fn test_build_tls_connector_with_extra_ca() {
        // 存在しない CA ファイルは失敗する
        let err = match build_tls_connector(&TlsMode::WithExtraCa(
            "/tmp/shoken-test-ca-missing-1586.pem".to_string(),
        )) {
            Ok(_) => panic!("存在しない CA ファイルで構築できた"),
            Err(err) => err,
        };
        assert!(err.contains("sslrootcert"), "案内がない: {err}");

        // PEM でない中身は失敗する
        let bad =
            std::env::temp_dir().join(format!("shoken-test-ca-bad-{}.pem", std::process::id()));
        std::fs::write(&bad, "not a certificate").unwrap();
        let err =
            match build_tls_connector(&TlsMode::WithExtraCa(bad.to_string_lossy().into_owned())) {
                Ok(_) => panic!("PEM でない中身で構築できた"),
                Err(err) => err,
            };
        assert!(err.contains("sslrootcert"), "案内がない: {err}");
        let _ = std::fs::remove_file(&bad);

        // 正しい PEM はシステムルートに足して構築できる（使い捨ての自己署名 CA）
        let good =
            std::env::temp_dir().join(format!("shoken-test-ca-good-{}.pem", std::process::id()));
        std::fs::write(&good, TEST_CA_PEM).unwrap();
        assert!(
            build_tls_connector(&TlsMode::WithExtraCa(good.to_string_lossy().into_owned())).is_ok()
        );
        let _ = std::fs::remove_file(&good);
    }

    /// 使い捨ての自己署名テスト CA（openssl で生成、有効期限 2 日）。
    /// `Certificate::from_pem` の解析に通ることだけを見る
    const TEST_CA_PEM: &str = "-----BEGIN CERTIFICATE-----\n\
        MIIDEzCCAfugAwIBAgIUIBz8scoCMzzPsh3HSvpELBwAMF4wDQYJKoZIhvcNAQEL\n\
        BQAwGTEXMBUGA1UEAwwOc2hva2VuLXRlc3QtY2EwHhcNMjYxMDI4MjMzOTA5WhcN\n\
        MjYxMDMwMjMzOTA5WjAZMRcwFQYDVQQDDA5zaG9rZW4tdGVzdC1jYTCCASIwDQYJ\n\
        KoZIhvcNAQEBBQADggEPADCCAQoCggEBAKoEaGxyCdyxdx5O+Cd9p3Ys/ER1RYio\n\
        w5G+5NZ+o8Vxp47bk/N60hfJTQGz3Bc3iuuh+w/Rgm2i+owSNYbHiqViEb2BkgIF\n\
        Em4Y96kFpOvfMwAjd0LlGn+aTQoKU7lqdqMvqYxFdySGgtmad3P1bsCofpJfWOdS\n\
        IvxfGp5goH06Crts6tWM7A8j2F4vr3cln1UgJeHT91UKByJPES0TwANz8rxaki9M\n\
        HbgFJ+wYkdpDkkwdyCbQx7QKFF28hDBMwdThHD+CvM5tu4DlRQ8MfIV/GxtIXlOX\n\
        lMVXMkOg1zG5HWxoEMfcALEtZMukHWUt7ggbOcDfq2pUtwdFzvm8A0UCAwEAAaNT\n\
        MFEwHQYDVR0OBBYEFNGgY9wHQEWMw9DTqZPvFoLw6Q8YMB8GA1UdIwQYMBaAFNGg\n\
        Y9wHQEWMw9DTqZPvFoLw6Q8YMA8GA1UdEwEB/wQFMAMBAf8wDQYJKoZIhvcNAQEL\n\
        BQADggEBADWjU80ah5BcOZbUGjHNeyIQ8mgeNMn05bOQaTNfaF4ZjGal+dgj1cse\n\
        JKdbdreB0AEKppCEXdnqqmeGsamSzSk9dXtl7GqLTx0kTxa5FnVWR9kd/LYxwBp0\n\
        WbPQM5abzNTOU7TAA258V/liOVhYyHhsrSHbTOzz8RVW+JYH5jNLKsRbRWZ02SSs\n\
        TU//hxMb9s/E0EQZDZl2jGxEb56v46DJKktyn8j2ojiFytO5pbghrMCxpyjiLd1r\n\
        rmOdokpUgEQ+Wq6CGpi/eq0nGNKhttl93bTloFtwl9RXjt6ulxfWQRW2p1gNkli+\n\
        AduEulXqlyfrNc8aTQd0G7sfCdCHPMo=\n\
        -----END CERTIFICATE-----\n";
}
