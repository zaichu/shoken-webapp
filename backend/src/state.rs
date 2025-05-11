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
