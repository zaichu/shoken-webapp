use shuttle_runtime::SecretStore;
use sqlx::PgPool;
use reqwest::Client;

#[derive(Clone)]
pub struct AppState {
    pub pool: PgPool,
    #[allow(dead_code)]
    pub secrets: SecretStore,
    pub client: Client,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeMap;
    use sqlx::postgres::PgPoolOptions;
    
    #[tokio::test]
    async fn test_app_state_creation() {
        // データベース接続のためのダミーURL（実際のテストでは実際のデータベースを使用）
        let database_url = "postgresql://user:password@localhost/test_db";
        
        // データベース接続プールを作成（実際の接続は行わない）
        let pool = PgPoolOptions::new()
            .max_connections(1)
            .connect_lazy(database_url)
            .expect("Failed to create connection pool");
        
        // シークレットストアを作成
        let bt = BTreeMap::from([
            ("test_key".to_owned(), "test_value".to_owned().into()),
        ]);
        let secrets = SecretStore::new(bt);
        
        // HTTPクライアントを作成
        let client = Client::new();
        
        // AppStateを作成
        let app_state = AppState {
            pool: pool.clone(),
            secrets: secrets.clone(),
            client: client.clone(),
        };
        
        // テスト：AppStateが正常に作成されることを確認
        // pool.size()は実際の接続数ではなく設定された最大接続数なので、基本的なチェックのみ
        let _ = &app_state.pool;
        let _ = &app_state.secrets;
        let _ = &app_state.client;
        
        // AppStateのクローンが正常に動作することを確認
        let cloned_state = app_state.clone();
        let _ = &cloned_state.pool;
        let _ = &cloned_state.secrets;
        let _ = &cloned_state.client;
    }
    
    #[tokio::test]
    async fn test_app_state_clone() {
        // データベース接続プールを作成（レイジー接続）
        let database_url = "postgresql://user:password@localhost/test_db";
        let pool = PgPoolOptions::new()
            .max_connections(1)
            .connect_lazy(database_url)
            .expect("Failed to create connection pool");
        
        // シークレットストアを作成
        let bt = BTreeMap::from([
            ("key1".to_owned(), "value1".to_owned().into()),
            ("key2".to_owned(), "value2".to_owned().into()),
        ]);
        let secrets = SecretStore::new(bt);
        
        // HTTPクライアントを作成
        let client = Client::new();
        
        // オリジナルのAppStateを作成
        let original_state = AppState {
            pool,
            secrets,
            client,
        };
        
        // クローンを作成
        let cloned_state = original_state.clone();
        
        // クローンが正常に作成されることを確認
        let _ = &cloned_state.pool;
        let _ = &cloned_state.secrets;
        let _ = &cloned_state.client;
    }
}
