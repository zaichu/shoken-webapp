use shuttle_runtime::SecretStore;
use sqlx::PgPool;

#[derive(Clone)]
pub struct AppState {
    pub pool: PgPool,
    pub _secrets: SecretStore,
}
