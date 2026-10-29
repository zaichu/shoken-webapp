#![cfg(target_os = "linux")]

use backend::db::{self, Bind, Db, connect_pool_lazy, run_migrations};
use chrono::NaiveDate;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::Duration;
use testcontainers::{ImageExt, runners::AsyncRunner};
use testcontainers_modules::postgres::Postgres;
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

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .to_path_buf()
}

fn run(key: &str, args: &[&str]) -> std::process::Output {
    Command::new(key)
        .args(args)
        .output()
        .unwrap_or_else(|_| panic!("テストに必要な道具がありません: {key}"))
}

fn require_pg_dump_17() {
    let out = run("pg_dump", &["--version"]);
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

async fn seed_dummy(pool: &Db) {
    run_migrations(pool).await.expect("マイグレーションに失敗");
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
                Bind::from(NaiveDate::from_ymd_opt(2026, 10, 29).unwrap()),
                Bind::from(code),
                Bind::from(name),
                Bind::from("プライム"),
            ],
        )
        .execute(pool)
        .await
        .expect("ダミー銘柄の投入に失敗");
    }
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

fn run_backup(pgurl: &str, recipient: &Path, out: &Path) -> std::process::Output {
    Command::new("bash")
        .arg(repo_root().join("scripts/db-backup.sh"))
        .arg(out)
        .env("PGURL", pgurl)
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
    psql.stdin
        .as_mut()
        .unwrap()
        .write_all(&gunzipped.stdout)
        .expect("リストア入力の書き込みに失敗");
    let restored = psql.wait_with_output().expect("リストアの完了待機に失敗");
    assert!(
        restored.status.success(),
        "リストアに失敗しました: {}",
        String::from_utf8_lossy(&restored.stderr)
    );
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
    seed_dummy(&source_pool).await;
    let expected = table_counts(&source_pool).await;
    assert_eq!(count_for(&expected, "users"), 1);
    assert_eq!(count_for(&expected, "stock"), 2);

    let backup = run_backup(&source_url, &recipient, &encrypted);
    assert!(
        backup.status.success(),
        "バックアップに失敗しました: {}",
        String::from_utf8_lossy(&backup.stderr)
    );
    let stored = fs::read(&encrypted).expect("暗号化ファイルがありません");
    assert!(!stored.is_empty());
    assert!(
        !String::from_utf8_lossy(&stored).contains("架空バックアップ銘柄A"),
        "暗号化ファイルに平文が残っています"
    );

    let dest = Postgres::default().with_tag("17").start().await.unwrap();
    let dest_port = dest.get_host_port_ipv4(5432).await.unwrap();
    let dest_url = format!("postgres://postgres:postgres@127.0.0.1:{dest_port}/postgres");
    connect_with_retry(&dest_url).await;
    restore_to(&dest_url, &encrypted, &identity);
    let dest_pool = connect_with_retry(&dest_url).await;
    assert_eq!(table_counts(&dest_pool).await, expected);
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
