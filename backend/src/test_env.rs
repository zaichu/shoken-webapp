#[rustfmt::skip]
use {std::env, tokio::sync::Mutex};
pub static ENV_MUTEX: Mutex<()> = Mutex::const_new(());
#[rustfmt::skip]
pub struct EnvGuard { key: &'static str, previous: Option<String> }
#[rustfmt::skip]
impl EnvGuard { pub fn set(key: &'static str, value: Option<&str>) -> Self { let previous = env::var(key).ok(); match value { Some(v) => env::set_var(key, v), None => env::remove_var(key) }; Self { key, previous } } }
#[rustfmt::skip]
impl Drop for EnvGuard { fn drop(&mut self) { match &self.previous { Some(v) => env::set_var(self.key, v), None => env::remove_var(self.key) } } }
