#[cfg(not(target_arch = "wasm32"))]
#[tokio::main]
async fn main() -> Result<(), String> {
    dotenvy::dotenv().ok();

    let database_url = migration_database_url(
        std::env::var("MIGRATION_DATABASE_URL").ok(),
        std::env::var("DATABASE_URL").ok(),
    )?;
    let pool = backend::db::connect_pool_lazy(&database_url, 1)
        .map_err(|e| format!("DB 接続設定の構築に失敗しました: {e}"))?;
    backend::db::run_migrations(&pool)
        .await
        .map_err(|e| format!("マイグレーション失敗: {e}"))?;
    println!("マイグレーション完了");
    Ok(())
}

fn migration_database_url(
    migration_database_url: Option<String>,
    database_url: Option<String>,
) -> Result<String, String> {
    migration_database_url
        .or(database_url)
        .ok_or_else(|| "MIGRATION_DATABASE_URL または DATABASE_URL が未設定です".to_string())
}

#[cfg(test)]
mod tests {
    use super::migration_database_url;

    #[test]
    fn prefers_migration_url() {
        assert_eq!(
            migration_database_url(Some("migration".into()), Some("app".into())).unwrap(),
            "migration"
        );
    }

    #[test]
    fn falls_back_to_database_url() {
        assert_eq!(
            migration_database_url(None, Some("app".into())).unwrap(),
            "app"
        );
    }
}

// worker-build の wasm ビルドでは bin も対象になるため、ネイティブ専用コードを cfg で閉じて空の main を置く
#[cfg(target_arch = "wasm32")]
fn main() {}
