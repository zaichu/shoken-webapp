#![cfg(target_os = "linux")]
#![allow(clippy::unwrap_used, clippy::expect_used)]

use backend::db::{self, Bind, Db, Executor, connect_pool_lazy, run_migrations};
use shared::value::UserId;
use std::fs;
use std::io::Write;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::Duration;
use testcontainers::{ImageExt, runners::AsyncRunner};
use testcontainers_modules::postgres::Postgres;
use time::Date;
use tokio::time::{sleep, timeout};
use uuid::Uuid;

struct Workdir {
    path: PathBuf,
}

impl Workdir {
    fn new() -> Self {
        let path = std::env::temp_dir().join(format!("shoken-db-backup-{}", Uuid::new_v4()));
        fs::create_dir_all(&path).unwrap();
        Self { path }
    }

    fn file(&self, name: &str) -> PathBuf {
        self.path.join(name)
    }
}

impl Drop for Workdir {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.path);
    }
}

fn run(key: &str, args: &[&str]) -> std::process::Output {
    Command::new(key)
        .args(args)
        .output()
        .unwrap_or_else(|_| panic!("テストに必要な道具がありません: {key}"))
}

fn require_pg_dump_17() {
    let out = run(
        "docker",
        &["run", "--rm", "postgres:17", "pg_dump", "--version"],
    );
    let version = String::from_utf8_lossy(&out.stdout);
    assert!(
        out.status.success() && version.contains(" 17."),
        "pg_dump 17 が必要です: {version}"
    );
}

fn generate_recipient(workdir: &Workdir) -> PathBuf {
    let identity = workdir.file("test-backup.key");
    let keygen = run("age-keygen", &["-o", identity.to_str().unwrap()]);
    assert!(
        keygen.status.success(),
        "age-keygen が失敗しました: {}",
        String::from_utf8_lossy(&keygen.stderr)
    );
    let key_text = fs::read_to_string(&identity).unwrap();
    let public_key = key_text
        .lines()
        .find_map(|line| line.strip_prefix("# public key: "))
        .expect("age-keygen の出力に公開鍵がありません")
        .trim()
        .to_string();
    let recipient = workdir.file("recipient.txt");
    fs::write(&recipient, format!("{public_key}\n")).unwrap();
    recipient
}

async fn connect_with_retry(database_url: &str) -> Db {
    let pool = connect_pool_lazy(database_url, 4).expect("プールの作成に失敗");
    for _ in 0..20 {
        let attempt = timeout(
            Duration::from_secs(2),
            db::query("SELECT 1", vec![]).execute(&pool),
        )
        .await;
        if matches!(attempt, Ok(Ok(_))) {
            return pool;
        }
        sleep(Duration::from_millis(500)).await;
    }
    panic!("Postgres に接続できませんでした");
}

async fn seed_dummy(pool: &Db) -> Uuid {
    run_migrations(pool).await.expect("マイグレーションに失敗");
    db::query("CREATE SCHEMA backup_probe", vec![])
        .execute(pool)
        .await
        .expect("別スキーマの作成に失敗");
    db::query(
        "CREATE TABLE backup_probe.secret_note (id bigint PRIMARY KEY, body text NOT NULL)",
        vec![],
    )
    .execute(pool)
    .await
    .expect("別スキーマのテーブル作成に失敗");
    db::query(
        "INSERT INTO backup_probe.secret_note (id, body) VALUES ($1, $2)",
        vec![Bind::from(1_i64), Bind::from("probe-secret-1596")],
    )
    .execute(pool)
    .await
    .expect("別スキーマへの投入に失敗");
    let user_id = Uuid::new_v4();
    db::query(
        "INSERT INTO users (id, google_id, email) VALUES ($1, $2, $3)",
        vec![
            Bind::from(user_id),
            Bind::from("backup-test-user-001"),
            Bind::from("backup-test-001@example.com"),
        ],
    )
    .execute(pool)
    .await
    .expect("ダミーユーザーの投入に失敗");
    for (code, name) in [
        ("9996", "架空バックアップ銘柄A"),
        ("9997", "架空バックアップ銘柄B"),
    ] {
        db::query(
            "INSERT INTO stock (date, code, name, market_category) VALUES ($1, $2, $3, $4)",
            vec![
                Bind::from(
                    Date::from_calendar_date(2026, time::Month::try_from(10).unwrap(), 29).unwrap(),
                ),
                Bind::from(code),
                Bind::from(name),
                Bind::from("プライム"),
            ],
        )
        .execute(pool)
        .await
        .expect("ダミー銘柄の投入に失敗");
    }
    user_id
}

async fn table_counts(pool: &Db) -> Vec<(String, i64)> {
    let tables = db::query(
        "SELECT table_name FROM information_schema.tables \
         WHERE table_schema = 'public' AND table_type = 'BASE TABLE' ORDER BY table_name",
        vec![],
    )
    .fetch_all::<(String,), _>(pool)
    .await
    .expect("テーブル一覧の取得に失敗");
    let mut counts = Vec::with_capacity(tables.len());
    for (table,) in tables {
        let count = db::query_scalar::<i64>(format!("SELECT COUNT(*) FROM \"{table}\""), vec![])
            .fetch_one(pool)
            .await
            .expect("件数の取得に失敗");
        counts.push((table, count));
    }
    counts
}

fn count_for(counts: &[(String, i64)], table: &str) -> i64 {
    counts
        .iter()
        .find_map(|(name, count)| (*name == table).then_some(*count))
        .unwrap_or(0)
}

fn docker_tool_wrapper(parent: &Path, tool: &str) -> PathBuf {
    let bin_dir = parent.join("bin");
    fs::create_dir_all(&bin_dir).expect("pg_dump wrapper のディレクトリ作成に失敗");
    let wrapper = bin_dir.join(tool);
    fs::write(
        &wrapper,
        format!(
            "#!/bin/bash\nargs=()\nfor arg in \"$@\"; do args+=(\"${{arg/127.0.0.1:/host.docker.internal:}}\"); done\nexec docker run --rm --add-host=host.docker.internal:host-gateway postgres:17 {tool} \"${{args[@]}}\"\n"
        ),
    )
    .expect("pg_dump wrapper の作成に失敗");
    let mut permissions = fs::metadata(&wrapper).unwrap().permissions();
    permissions.set_mode(0o755);
    fs::set_permissions(&wrapper, permissions).expect("pg_dump wrapper の実行権限設定に失敗");
    wrapper
}

fn dump_roles(admin_pgurl: &str, workdir: &Workdir) -> String {
    let wrapper = docker_tool_wrapper(&workdir.path, "pg_dumpall");
    let out = Command::new(&wrapper)
        .arg("--roles-only")
        .arg("-d")
        .arg(admin_pgurl)
        .output()
        .expect("pg_dumpall の起動に失敗");
    assert!(
        out.status.success(),
        "ロール書き出しに失敗: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    String::from_utf8_lossy(&out.stdout)
        .lines()
        .filter(|line| {
            !line.starts_with("CREATE ROLE postgres;") && !line.starts_with("ALTER ROLE postgres ")
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn apply_sql(dest_url: &str, sql: &str, context: &str) {
    let mut psql = Command::new("psql")
        .arg(dest_url)
        .arg("-v")
        .arg("ON_ERROR_STOP=1")
        .arg("-q")
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .spawn()
        .expect("psql の起動に失敗");
    let write_result = psql.stdin.as_mut().unwrap().write_all(sql.as_bytes());
    let restored = psql.wait_with_output().expect("適用の完了待機に失敗");
    if let Err(error) = write_result {
        assert!(
            restored.status.success(),
            "{context}の書き込みに失敗: {error}; {}",
            String::from_utf8_lossy(&restored.stderr)
        );
    }
    assert!(
        restored.status.success(),
        "{context}に失敗しました: {}",
        String::from_utf8_lossy(&restored.stderr)
    );
}

fn run_backup(pgurl: &str, recipient: &Path, out: &Path) -> std::process::Output {
    let workdir = recipient
        .parent()
        .expect("公開鍵の親ディレクトリがありません");
    let pg_dump = docker_tool_wrapper(workdir, "pg_dump");
    let bin_dir = pg_dump.parent().unwrap();
    let path = format!(
        "{}:{}",
        bin_dir.display(),
        std::env::var("PATH").unwrap_or_default()
    );
    Command::new("bash")
        .arg(
            Path::new(env!("CARGO_MANIFEST_DIR"))
                .parent()
                .unwrap()
                .join("scripts/db-backup.sh"),
        )
        .arg(out)
        .env("PATH", path)
        .env("DB_ADMIN_URL", pgurl)
        .env("RECIPIENT_FILE", recipient)
        .output()
        .expect("バックアップスクリプトの起動に失敗")
}

fn restore_to(dest_url: &str, encrypted: &Path, identity: &Path) {
    let workdir = encrypted.parent().unwrap();
    let dumped = workdir.join("dump.sql.gz");
    let decrypted = run(
        "age",
        &[
            "--decrypt",
            "--identity",
            identity.to_str().unwrap(),
            "--output",
            dumped.to_str().unwrap(),
            encrypted.to_str().unwrap(),
        ],
    );
    assert!(
        decrypted.status.success(),
        "復号に失敗しました: {}",
        String::from_utf8_lossy(&decrypted.stderr)
    );
    let gunzipped = run("gzip", &["-dc", dumped.to_str().unwrap()]);
    assert!(
        gunzipped.status.success(),
        "展開に失敗しました: {}",
        String::from_utf8_lossy(&gunzipped.stderr)
    );
    let sql = String::from_utf8_lossy(&gunzipped.stdout)
        .replace(
            "CREATE SCHEMA public;",
            "CREATE SCHEMA IF NOT EXISTS public;",
        )
        .replace("dump_owner", "postgres")
        .replace("backup_operator", "postgres")
        .replace("pg_database_owner", "postgres");
    apply_sql(dest_url, &sql, "リストア");
}

fn decrypted_plaintext(encrypted: &Path, identity: &Path, workdir: &Workdir) -> String {
    let dumped = workdir.file("inspect.sql.gz");
    let decrypted = run(
        "age",
        &[
            "--decrypt",
            "--identity",
            identity.to_str().unwrap(),
            "--output",
            dumped.to_str().unwrap(),
            encrypted.to_str().unwrap(),
        ],
    );
    assert!(
        decrypted.status.success(),
        "復号に失敗しました: {}",
        String::from_utf8_lossy(&decrypted.stderr)
    );
    let gunzipped = run("gzip", &["-dc", dumped.to_str().unwrap()]);
    assert!(
        gunzipped.status.success(),
        "展開に失敗しました: {}",
        String::from_utf8_lossy(&gunzipped.stderr)
    );
    String::from_utf8_lossy(&gunzipped.stdout).into_owned()
}

#[tokio::test]
#[ignore = "requires Docker to run Postgres containers"]
async fn backup_roundtrip_keeps_all_table_counts() {
    require_pg_dump_17();
    let workdir = Workdir::new();
    let recipient = generate_recipient(&workdir);
    let identity = workdir.file("test-backup.key");
    let encrypted = workdir.file("backup.sql.gz.age");

    let source = Postgres::default().with_tag("17").start().await.unwrap();
    let source_port = source.get_host_port_ipv4(5432).await.unwrap();
    let source_url = format!("postgres://postgres:postgres@127.0.0.1:{source_port}/postgres");
    let source_pool = connect_with_retry(&source_url).await;
    let source_admin = connect_with_retry(&source_url).await;
    let user_id = seed_dummy(&source_pool).await;
    source_pool
        .batch_execute(
            "CREATE ROLE backup_operator LOGIN PASSWORD 'backup-test' NOSUPERUSER BYPASSRLS; \
             GRANT USAGE ON SCHEMA public TO backup_operator; \
             GRANT SELECT ON ALL TABLES IN SCHEMA public TO backup_operator; \
             GRANT SELECT ON ALL SEQUENCES IN SCHEMA public TO backup_operator;",
        )
        .await
        .unwrap();
    source_admin
        .batch_execute(
            "ALTER TABLE users FORCE ROW LEVEL SECURITY; \
             ALTER TABLE sessions FORCE ROW LEVEL SECURITY; \
             ALTER TABLE dividends FORCE ROW LEVEL SECURITY; \
             ALTER TABLE domestic_stocks FORCE ROW LEVEL SECURITY; \
             ALTER TABLE mutualfunds FORCE ROW LEVEL SECURITY; \
             ALTER TABLE asset_balances FORCE ROW LEVEL SECURITY; \
             ALTER TABLE fund_balances FORCE ROW LEVEL SECURITY; \
             ALTER TABLE csv_import_history FORCE ROW LEVEL SECURITY; \
             ALTER TABLE login_events FORCE ROW LEVEL SECURITY; \
             ALTER TABLE login_notify_tokens FORCE ROW LEVEL SECURITY; \
             ALTER TABLE admin_login_views FORCE ROW LEVEL SECURITY;",
        )
        .await
        .unwrap();
    let expected = table_counts(&source_pool).await;
    assert_eq!(count_for(&expected, "users"), 1);
    assert_eq!(count_for(&expected, "stock"), 2);
    let probe_access: bool = db::query_scalar(
        "SELECT has_table_privilege('backup_operator', 'stock', 'SELECT')",
        vec![],
    )
    .fetch_one(&source_admin)
    .await
    .unwrap();
    assert!(
        probe_access,
        "テーブル所有者と異なる backup_operator に SELECT がある"
    );

    let backup = run_backup(
        &format!("postgres://backup_operator:backup-test@127.0.0.1:{source_port}/postgres"),
        &recipient,
        &encrypted,
    );
    assert!(
        backup.status.success(),
        "バックアップに失敗しました: {} / {}",
        String::from_utf8_lossy(&backup.stdout),
        String::from_utf8_lossy(&backup.stderr)
    );
    let stored = fs::read(&encrypted).expect("暗号化ファイルがありません");
    assert_ne!(stored, [] as [u8; 0]);
    assert!(
        !String::from_utf8_lossy(&stored).contains("架空バックアップ銘柄A"),
        "暗号化ファイルに平文が残っています"
    );
    let plain = decrypted_plaintext(&encrypted, &identity, &workdir);
    assert!(
        plain.contains("架空バックアップ銘柄A"),
        "public のデータが書き出しにありません"
    );
    assert!(
        !plain.contains("backup_probe") && !plain.contains("probe-secret-1596"),
        "public 外のスキーマが書き出しに含まれています"
    );

    let dest = Postgres::default().with_tag("17").start().await.unwrap();
    let dest_port = dest.get_host_port_ipv4(5432).await.unwrap();
    let dest_url = format!("postgres://postgres:postgres@127.0.0.1:{dest_port}/postgres");
    let dest_prep = connect_with_retry(&dest_url).await;
    for stmt in [
        "CREATE EXTENSION IF NOT EXISTS citext WITH SCHEMA public",
        "CREATE EXTENSION IF NOT EXISTS pg_trgm WITH SCHEMA public",
        "CREATE EXTENSION IF NOT EXISTS pgcrypto WITH SCHEMA public",
    ] {
        db::query(stmt, vec![])
            .execute(&dest_prep)
            .await
            .expect("復元前の拡張作成に失敗");
    }
    // dump はロールの定義を含まないため、先に書き出し元のロールだけを作る
    // (復元手順 docs/db-backup.md と同じ。土台の 0100 全体は app_enter が dump と重複するため使わない)。
    apply_sql(
        &dest_url,
        &dump_roles(&source_url, &workdir),
        "復元先のロール作成",
    );
    restore_to(&dest_url, &encrypted, &identity);
    let dest_pool = connect_with_retry(&dest_url).await;
    assert_eq!(table_counts(&dest_pool).await, expected);
    let worker_url = format!("postgres://app_worker:restore-test@127.0.0.1:{dest_port}/postgres");
    dest_prep
        .batch_execute("ALTER ROLE app_worker PASSWORD 'restore-test'")
        .await
        .expect("app_worker のパスワード設定に失敗");
    let worker = connect_with_retry(&worker_url).await;
    let mut tx = worker
        .begin_with_user(UserId::from(user_id))
        .await
        .expect("利用者 Tx の開始に失敗");
    let mine: i64 = db::query_scalar("SELECT COUNT(*) FROM users", vec![])
        .fetch_one(&mut tx)
        .await
        .expect("利用者の行の読み取りに失敗");
    assert_eq!(mine, 1, "復元した DB で利用者の行が読める");
    tx.rollback().await.expect("利用者 Tx の破棄に失敗");
    let probe_tables = db::query_scalar::<i64>(
        "SELECT COUNT(*) FROM information_schema.tables WHERE table_schema = 'backup_probe'"
            .to_string(),
        vec![],
    )
    .fetch_one(&dest_pool)
    .await
    .expect("別スキーマの有無の取得に失敗");
    assert_eq!(probe_tables, 0);
}

#[tokio::test]
#[ignore = "requires Docker to run Postgres containers"]
async fn migration_role_runs_all_migrations_and_force_rls_is_verified_separately() {
    let node = Postgres::default().with_tag("17").start().await.unwrap();
    let port = node.get_host_port_ipv4(5432).await.unwrap();
    let admin_url = format!("postgres://postgres:postgres@127.0.0.1:{port}/postgres");
    let admin = connect_with_retry(&admin_url).await;
    admin
        .batch_execute(
            // 土台 0100 が BYPASSRLS の限定関数所有ロールを作るため、本番の移行ロールと
            // 同じ条件にする(parity の owner モードと同一。NOBYPASSRLS ではロールを作れない)
            "CREATE ROLE migration_admin LOGIN PASSWORD 'test' NOSUPERUSER BYPASSRLS CREATEROLE; \
             GRANT CREATE ON DATABASE postgres TO migration_admin; \
             GRANT CREATE ON SCHEMA public TO migration_admin WITH GRANT OPTION; \
             CREATE ROLE app_restricted NOLOGIN; \
             GRANT app_restricted TO migration_admin WITH ADMIN TRUE, SET TRUE;",
        )
        .await
        .unwrap();
    let role_url = format!("postgres://migration_admin:test@127.0.0.1:{port}/postgres");
    let role_pool = connect_with_retry(&role_url).await;
    let role_name: String = db::query_scalar("SELECT current_user::text", vec![])
        .fetch_one(&role_pool)
        .await
        .unwrap();
    assert_eq!(role_name, "migration_admin");
    run_migrations(&role_pool)
        .await
        .expect("管理ロールによる全 migration 適用");

    let regular: i64 = db::query_scalar("SELECT COUNT(*) FROM users", vec![])
        .fetch_one(&admin)
        .await
        .unwrap();
    assert_eq!(regular, 0, "admin 接続では users が空である");

    let forced: i64 = db::query_scalar(
        "SELECT COUNT(*) FROM pg_class c JOIN pg_namespace n ON n.oid = c.relnamespace WHERE n.nspname = 'public' AND c.relkind = 'r' AND c.relforcerowsecurity",
        vec![],
    )
    .fetch_one(&role_pool)
    .await
    .unwrap();
    assert_eq!(forced, 0, "migration の全表で FORCE RLS は無効である");
    db::query(
        "INSERT INTO users (id, google_id, email) VALUES ($1, $2, $3)",
        vec![
            Bind::from(Uuid::new_v4()),
            Bind::from("force-policy-probe"),
            Bind::from("force-policy-probe@example.invalid"),
        ],
    )
    .execute(&role_pool)
    .await
    .unwrap();
    // FORCE RLS の検証は BYPASSRLS を持たない別ロールで行う。移行ロール自体は
    // 限定関数の所有ロールを作るため BYPASSRLS が要り、FORCE の影響を受けない。
    // 所有者を force_probe に移してから確かめる(非所有者の拒否は FORCE に依存しないため)。
    admin
        .batch_execute(
            "CREATE ROLE force_probe LOGIN PASSWORD 'test' NOSUPERUSER NOBYPASSRLS; \
             GRANT USAGE ON SCHEMA public TO force_probe; \
             ALTER TABLE users OWNER TO force_probe;",
        )
        .await
        .unwrap();
    let probe_pool = connect_with_retry(&format!(
        "postgres://force_probe:test@127.0.0.1:{port}/postgres"
    ))
    .await;
    probe_pool
        .batch_execute("ALTER TABLE users FORCE ROW LEVEL SECURITY")
        .await
        .unwrap();
    db::query(
        "INSERT INTO users (id, google_id, email) VALUES ($1, $2, $3)",
        vec![
            Bind::from(Uuid::new_v4()),
            Bind::from("admin-migration-probe"),
            Bind::from("admin-migration-probe@example.invalid"),
        ],
    )
    .execute(&probe_pool)
    .await
    .expect_err("RLS を FORCE した表への BYPASSRLS の無い所有者の INSERT は拒否される");
    db::query(
        "INSERT INTO users (id, google_id, email) VALUES ($1, $2, $3)",
        vec![
            Bind::from(Uuid::new_v4()),
            Bind::from("owner-force-probe"),
            Bind::from("owner-force-probe@example.invalid"),
        ],
    )
    .execute(&role_pool)
    .await
    .expect("BYPASSRLS の移行ロールは FORCE 下でも書ける");
    let force_enabled: bool = db::query_scalar(
        "SELECT relforcerowsecurity FROM pg_class WHERE relname = 'users'",
        vec![],
    )
    .fetch_one(&role_pool)
    .await
    .unwrap();
    assert!(force_enabled, "users の FORCE RLS 属性が有効になっている");

    let owner = Postgres::default().with_tag("17").start().await.unwrap();
    let owner_port = owner.get_host_port_ipv4(5432).await.unwrap();
    let owner_admin_url = format!("postgres://postgres:postgres@127.0.0.1:{owner_port}/postgres");
    let owner_admin = connect_with_retry(&owner_admin_url).await;
    owner_admin
        .batch_execute(
            "CREATE ROLE dump_owner LOGIN PASSWORD 'test' NOSUPERUSER BYPASSRLS CREATEROLE; \
             CREATE ROLE backup_operator LOGIN PASSWORD 'backup-test' NOSUPERUSER BYPASSRLS; \
             GRANT CREATE ON DATABASE postgres TO dump_owner; \
             GRANT CREATE ON SCHEMA public TO dump_owner WITH GRANT OPTION; \
             CREATE ROLE app_restricted NOLOGIN; \
             GRANT app_restricted TO dump_owner WITH ADMIN TRUE, SET TRUE; \
             GRANT USAGE ON SCHEMA public TO backup_operator; \
             ALTER DEFAULT PRIVILEGES FOR ROLE dump_owner IN SCHEMA public GRANT SELECT ON TABLES TO backup_operator; \
             ALTER DEFAULT PRIVILEGES FOR ROLE dump_owner IN SCHEMA public GRANT SELECT ON SEQUENCES TO backup_operator; \
             GRANT SELECT ON ALL TABLES IN SCHEMA public TO backup_operator; \
             GRANT SELECT ON ALL SEQUENCES IN SCHEMA public TO backup_operator;",
        )
        .await
        .unwrap();
    let owner_url = format!("postgres://dump_owner:test@127.0.0.1:{owner_port}/postgres");
    let owner_pool = connect_with_retry(&owner_url).await;
    run_migrations(&owner_pool)
        .await
        .expect("dump 用 DB の migration");
    db::query(
        "INSERT INTO users (id, google_id, email) VALUES ($1, $2, $3)",
        vec![
            Bind::from(Uuid::new_v4()),
            Bind::from("dump-force-probe"),
            Bind::from("dump-force-probe@example.invalid"),
        ],
    )
    .execute(&owner_pool)
    .await
    .unwrap();
    owner_admin
        .batch_execute(
            "ALTER TABLE users FORCE ROW LEVEL SECURITY; \
             ALTER TABLE sessions FORCE ROW LEVEL SECURITY; \
             ALTER TABLE dividends FORCE ROW LEVEL SECURITY; \
             ALTER TABLE domestic_stocks FORCE ROW LEVEL SECURITY; \
             ALTER TABLE mutualfunds FORCE ROW LEVEL SECURITY; \
             ALTER TABLE asset_balances FORCE ROW LEVEL SECURITY; \
             ALTER TABLE fund_balances FORCE ROW LEVEL SECURITY; \
             ALTER TABLE csv_import_history FORCE ROW LEVEL SECURITY; \
             ALTER TABLE login_events FORCE ROW LEVEL SECURITY; \
             ALTER TABLE login_notify_tokens FORCE ROW LEVEL SECURITY; \
             ALTER TABLE admin_login_views FORCE ROW LEVEL SECURITY;",
        )
        .await
        .unwrap();
    let workdir = Workdir::new();
    let recipient = generate_recipient(&workdir);
    let encrypted = workdir.file("restore.sql.gz.age");
    let backup_url =
        format!("postgres://backup_operator:backup-test@127.0.0.1:{owner_port}/postgres");
    let backup = run_backup(&backup_url, &recipient, &encrypted);
    assert!(
        backup.status.success(),
        "owner の FORCE 下 dump が失敗しました: {} / {}",
        String::from_utf8_lossy(&backup.stdout),
        String::from_utf8_lossy(&backup.stderr)
    );
    let target = Postgres::default().with_tag("17").start().await.unwrap();
    let target_port = target.get_host_port_ipv4(5432).await.unwrap();
    let target_admin_url = format!("postgres://postgres:postgres@127.0.0.1:{target_port}/postgres");
    let target_url = target_admin_url.clone();
    let target_pool = connect_with_retry(&target_admin_url).await;
    for stmt in [
        "CREATE EXTENSION IF NOT EXISTS citext WITH SCHEMA public",
        "CREATE EXTENSION IF NOT EXISTS pg_trgm WITH SCHEMA public",
        "CREATE EXTENSION IF NOT EXISTS pgcrypto WITH SCHEMA public",
    ] {
        db::query(stmt, vec![]).execute(&target_pool).await.unwrap();
    }
    apply_sql(
        &target_url,
        &dump_roles(&owner_admin_url, &workdir),
        "復元先のロール作成",
    );
    restore_to(&target_url, &encrypted, &workdir.file("test-backup.key"));
    let restored: i64 = db::query_scalar("SELECT COUNT(*) FROM users", vec![])
        .fetch_one(&target_pool)
        .await
        .unwrap();
    assert_eq!(
        restored, 1,
        "全表 backup の restore 後に FORCE 下の行が復元される"
    );
}

#[tokio::test]
#[ignore = "requires Docker to run Postgres containers"]
async fn missing_recipient_fails_without_plaintext() {
    require_pg_dump_17();
    let workdir = Workdir::new();
    let missing = workdir.file("no-such-recipient.txt");
    let out = workdir.file("backup.sql.gz.age");
    let fake_password = "fidpw-backup-1588";
    let fake_url = format!("postgres://backup-test:{fake_password}@localhost:1/backup-test");

    let backup = run_backup(&fake_url, &missing, &out);
    assert!(!backup.status.success(), "公開鍵が無いのに成功しています");
    assert!(!out.exists(), "失敗したのに出力が残っています");
    for entry in fs::read_dir(&workdir.path).unwrap() {
        let name = entry.unwrap().file_name().to_string_lossy().into_owned();
        assert!(
            !name.ends_with(".sql") && !name.ends_with(".sql.gz"),
            "平文の dump が残っています: {name}"
        );
    }
    let logged = format!(
        "{}{}",
        String::from_utf8_lossy(&backup.stdout),
        String::from_utf8_lossy(&backup.stderr)
    );
    assert!(
        !logged.contains(fake_password) && !logged.contains("postgres://"),
        "ログに接続情報が出ています"
    );
}
