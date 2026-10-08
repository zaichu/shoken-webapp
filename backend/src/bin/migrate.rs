#[cfg(not(target_arch = "wasm32"))]
#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();

    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL が未設定です");
    let pool =
        backend::db::connect_pool_lazy(&database_url, 1).expect("DB 接続設定の構築に失敗しました");
    backend::db::run_migrations(&pool)
        .await
        .expect("マイグレーション失敗");
    println!("マイグレーション完了");
}

// worker-build の wasm ビルドでは bin も対象になるため、ネイティブ専用コードを cfg で閉じて空の main を置く
#[cfg(target_arch = "wasm32")]
fn main() {}
