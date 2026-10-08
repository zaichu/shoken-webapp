//! PostgreSQL アクセス層。native は deadpool-postgres、wasm (Workers) は
//! Hyperdrive 経由の tokio-postgres を内部に持ち、サービス層は同一の API を使う。

mod bind;
mod builder;
#[cfg(not(target_arch = "wasm32"))]
mod migrate;
pub mod numeric;

pub use bind::Bind;
pub use builder::{QueryBuilder, Separated};
#[cfg(not(target_arch = "wasm32"))]
pub use migrate::run_migrations;
pub use numeric::Numeric;
pub use tokio_postgres::Row;
pub use tokio_postgres::error::SqlState;

use postgres_types::{FromSql, ToSql};
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

fn param_refs(params: &[Bind]) -> Vec<&(dyn ToSql + Sync)> {
    params.iter().map(|p| p as &(dyn ToSql + Sync)).collect()
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

/// コネクションプール（native）/ 共有クライアント（wasm）のハンドル。
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

#[cfg(not(target_arch = "wasm32"))]
pub fn connect_pool_lazy(database_url: &str, max_connections: u32) -> Result<Db, String> {
    let database_url = database_url.trim();
    validate_database_url(database_url)?;
    let sanitized_url = sanitize_database_url(database_url)?;
    let config = sanitized_url
        .parse::<tokio_postgres::Config>()
        .map_err(|e| format!("DB 接続設定の解析に失敗しました: {e}"))?;
    let tls = postgres_native_tls::MakeTlsConnector::new(
        native_tls::TlsConnector::new()
            .map_err(|e| format!("TLS コネクタの初期化に失敗しました: {e}"))?,
    );
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

    pub fn is_closed(&self) -> bool {
        match &*self.inner {
            #[cfg(not(target_arch = "wasm32"))]
            DbInner::Pool(pool) => pool.is_closed(),
            #[cfg(target_arch = "wasm32")]
            DbInner::Worker(_) => false,
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
                    done: false,
                })
            }
            #[cfg(target_arch = "wasm32")]
            DbInner::Worker(db) => {
                let client = db.fresh_client().await?;
                let guard = TxGuard::new(TxConn::Wasm(client));
                guard.conn().batch_execute("BEGIN").await?;
                Ok(Tx {
                    conn: Some(guard.disarm()),
                    done: false,
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
                    obj.query(sql, &param_refs(params))
                        .await
                        .map_err(DbError::Pg)
                }
                #[cfg(target_arch = "wasm32")]
                DbInner::Worker(db) => {
                    let client = db.client().await?;
                    client
                        .query(sql, &param_refs(params))
                        .await
                        .map_err(DbError::Pg)
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
                    obj.execute(sql, &param_refs(params))
                        .await
                        .map_err(DbError::Pg)
                }
                #[cfg(target_arch = "wasm32")]
                DbInner::Worker(db) => {
                    let client = db.client().await?;
                    client
                        .execute(sql, &param_refs(params))
                        .await
                        .map_err(DbError::Pg)
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
                    obj.batch_execute(sql).await.map_err(DbError::Pg)
                }
                #[cfg(target_arch = "wasm32")]
                DbInner::Worker(db) => {
                    let client = db.client().await?;
                    client.batch_execute(sql).await.map_err(DbError::Pg)
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
    done: bool,
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
            TxConn::Native(obj) => obj.batch_execute(sql).await.map_err(DbError::Pg),
            #[cfg(target_arch = "wasm32")]
            TxConn::Wasm(client) => client.batch_execute(sql).await.map_err(DbError::Pg),
        }
    }

    /// 開いたかもしれないトランザクションを ROLLBACK で確実に閉じてから
    /// 接続を手放す（native はプール返却、wasm は drop）。spawn できない
    /// 場合は接続をプールから切り離して破棄し、サーバー側でロールバックさせる
    fn rollback_and_release(self) {
        match self {
            #[cfg(not(target_arch = "wasm32"))]
            TxConn::Native(obj) => {
                if let Ok(handle) = tokio::runtime::Handle::try_current() {
                    handle.spawn(async move {
                        let _ = obj.batch_execute("ROLLBACK").await;
                        drop(obj);
                    });
                } else {
                    drop(deadpool_postgres::Object::take(obj));
                }
            }
            #[cfg(target_arch = "wasm32")]
            TxConn::Wasm(client) => {
                wasm_bindgen_futures::spawn_local(async move {
                    let _ = client.batch_execute("ROLLBACK").await;
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
        self.done = true;
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
        self.done = true;
        let guard = TxGuard::new(conn);
        guard.conn().batch_execute("ROLLBACK").await?;
        drop(guard.disarm());
        Ok(())
    }
}

impl Drop for Tx {
    fn drop(&mut self) {
        if self.done {
            return;
        }
        // commit なしで drop された = ロールバック要。
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
                            .query(sql, &param_refs(params))
                            .await
                            .map_err(DbError::Pg),
                        #[cfg(target_arch = "wasm32")]
                        TxConn::Wasm(client) => client
                            .query(sql, &param_refs(params))
                            .await
                            .map_err(DbError::Pg),
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
                            .execute(sql, &param_refs(params))
                            .await
                            .map_err(DbError::Pg),
                        #[cfg(target_arch = "wasm32")]
                        TxConn::Wasm(client) => client
                            .execute(sql, &param_refs(params))
                            .await
                            .map_err(DbError::Pg),
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

impl_executor_for_tx!(Tx);
impl_executor_for_tx!(&Tx);
impl_executor_for_tx!(&mut Tx);

// ---- native 起動時の接続リトライ（Fly の grace_period と整合させる） ----

#[cfg(not(target_arch = "wasm32"))]
use std::time::Duration;
#[cfg(not(target_arch = "wasm32"))]
use url::Url;

// 起動時 DB 接続の retry パラメータ。fly.toml の grace_period と整合すること
// 最大待機 = MAX_ATTEMPTS * CONNECT_TIMEOUT_SECS + (MAX_ATTEMPTS-1) * RETRY_DELAY_SECS = 2*15 + 1*3 = 33s < 40s
#[cfg(not(target_arch = "wasm32"))]
pub(crate) const MAX_ATTEMPTS: u32 = 2;
#[cfg(not(target_arch = "wasm32"))]
pub(crate) const CONNECT_TIMEOUT_SECS: u64 = 15;
#[cfg(not(target_arch = "wasm32"))]
pub(crate) const RETRY_DELAY_SECS: u64 = 3;

// Wait for an already-created pool to establish a startup connection with bounded retry.
// The pool is not recreated, so runtime state and migrations share the same handle.
#[cfg(not(target_arch = "wasm32"))]
pub async fn wait_for_pool_with_retry(pool: &Db) -> Result<(), String> {
    let connect_timeout = Duration::from_secs(CONNECT_TIMEOUT_SECS);
    let retry_delay = Duration::from_secs(RETRY_DELAY_SECS);

    for attempt in 1..=MAX_ATTEMPTS {
        let DbInner::Pool(inner_pool) = &*pool.inner;
        match tokio::time::timeout(connect_timeout, inner_pool.get()).await {
            Ok(Ok(connection)) => {
                drop(connection);
                return Ok(());
            }
            Ok(Err(e)) if is_transient_error(&e) => {
                tracing::warn!(
                    attempt,
                    max_attempts = MAX_ATTEMPTS,
                    "DB接続の一時的なエラー、リトライします"
                );
            }
            Ok(Err(_)) => {
                return Err("データベース接続の設定が不正です".to_string());
            }
            Err(_) => {
                tracing::warn!(
                    attempt,
                    max_attempts = MAX_ATTEMPTS,
                    "DB接続がタイムアウト、リトライします"
                );
            }
        }

        if attempt < MAX_ATTEMPTS {
            tokio::time::sleep(retry_delay).await;
        }
    }

    Err("データベースへの接続に失敗しました（リトライ上限超過）".to_string())
}

/// 一時的な接続失敗（タイムアウト・I/O）かどうか。認証失敗や DB 不存在など
/// 設定系のエラーはリトライしても治らないため対象外とする
#[cfg(not(target_arch = "wasm32"))]
fn is_transient_error(e: &deadpool_postgres::PoolError) -> bool {
    match e {
        deadpool_postgres::PoolError::Timeout(_) => true,
        deadpool_postgres::PoolError::Backend(e) => {
            let mut source: &dyn std::error::Error = e;
            loop {
                if source.is::<std::io::Error>() {
                    return true;
                }
                match source.source() {
                    Some(s) => source = s,
                    None => return false,
                }
            }
        }
        _ => false,
    }
}

/// `tokio_postgres::Config` へ渡す前に `channel_binding` query parameter を除去する。
/// `channel_binding` がない場合は入力文字列をそのまま返す。
/// 保持する query pair は raw 文字列を再利用し percent-encoding を変換しない。
#[cfg(not(target_arch = "wasm32"))]
fn sanitize_database_url(url: &str) -> Result<String, String> {
    let parsed = Url::parse(url).map_err(|_| "DATABASE_URL の解析に失敗しました".to_string())?;

    if !parsed.query_pairs().any(|(k, _)| k == "channel_binding") {
        return Ok(url.to_string());
    }

    // raw pair と decoded pair を zip し、decoded key が channel_binding の raw pair を除去する
    let filtered_query = parsed
        .query()
        .map(|q| {
            let raw_pairs: Vec<&str> = q.split('&').collect();
            let decoded_pairs: Vec<_> = parsed.query_pairs().collect();
            raw_pairs
                .iter()
                .zip(decoded_pairs.iter())
                .filter(|(_, (k, _))| k != "channel_binding")
                .map(|(raw, _)| *raw)
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

    /// Fly v196 起動直後に観測された ~10-15 秒の接続遅延をカバーできることを保証する。
    /// CONNECT_TIMEOUT_SECS が短すぎると attempt=1 で recoverable WARN が出る（v196 実測: attempt=1 max_attempts=3）。
    /// 上限は下の retry budget のコンパイル時検査が担う。
    const _: () = assert!(
        CONNECT_TIMEOUT_SECS >= 15,
        "CONNECT_TIMEOUT_SECS は Fly v196 実測遅延（~10-15s）をカバーするため 15 以上が必要"
    );

    /// 最大待機時間が fly.toml の grace_period を超えないことを保証する。
    /// grace_period を変更した場合はこの定数も更新すること。
    /// 最大待機 = MAX_ATTEMPTS * CONNECT_TIMEOUT_SECS + (MAX_ATTEMPTS-1) * RETRY_DELAY_SECS = 2*15 + 1*3 = 33s < 40s
    const GRACE_PERIOD_SECS: u64 = 40; // fly.toml [[http_service.checks]] grace_period と同期
    const MAX_WAIT_SECS: u64 =
        MAX_ATTEMPTS as u64 * CONNECT_TIMEOUT_SECS + (MAX_ATTEMPTS as u64 - 1) * RETRY_DELAY_SECS;
    const _: () = assert!(
        MAX_WAIT_SECS < GRACE_PERIOD_SECS,
        "最大待機時間が grace_period を超える"
    );
}
