/// env var 操作を伴うテスト全体で共有するユーティリティ
///
/// `std::env::set_var` / `remove_var` はプロセス全体に影響するため、
/// crate 内の env テストは必ずここの `ENV_MUTEX` を取得してから実行する。
use std::env;
use tokio::sync::Mutex;

pub static ENV_MUTEX: Mutex<()> = Mutex::const_new(());

/// env var を操作し、Drop 時に元の値へ自動復元するガード
pub struct EnvGuard {
    key: &'static str,
    previous: Option<String>,
}

impl EnvGuard {
    pub fn set(key: &'static str, value: Option<&str>) -> Self {
        let previous = env::var(key).ok();
        match value {
            Some(v) => env::set_var(key, v),
            None => env::remove_var(key),
        }
        Self { key, previous }
    }
}

impl Drop for EnvGuard {
    fn drop(&mut self) {
        match &self.previous {
            Some(v) => env::set_var(self.key, v),
            None => env::remove_var(self.key),
        }
    }
}
