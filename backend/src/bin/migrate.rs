#[cfg(not(target_arch = "wasm32"))]
#[tokio::main]
async fn main() -> Result<(), String> {
    dotenvy::dotenv().ok();

    let database_url = std::env::var("MIGRATION_DATABASE_URL")
        .or_else(|_| std::env::var("DATABASE_URL"))
        .map_err(|_| "MIGRATION_DATABASE_URL または DATABASE_URL が未設定です".to_string())?;
    let pool = backend::db::connect_pool_lazy(&database_url, 1)
        .map_err(|e| format!("DB 接続設定の構築に失敗しました: {e}"))?;
    backend::db::run_migrations(&pool)
        .await
        .map_err(|e| format!("マイグレーション失敗: {e}"))?;
    println!("マイグレーション完了");
    Ok(())
}

// worker-build の wasm ビルドでは bin も対象になるため、ネイティブ専用コードを cfg で閉じて空の main を置く
#[cfg(target_arch = "wasm32")]
fn main() {}
