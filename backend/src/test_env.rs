use {std::env, tokio::sync::Mutex};
pub static ENV_MUTEX: Mutex<()> = Mutex::const_new(());
pub struct EnvGuard {
    key: &'static str,
    previous: Option<String>,
}

impl EnvGuard {
    pub fn set(key: &'static str, value: Option<&str>) -> Self {
        let previous = env::var(key).ok();

        match value {
            Some(v) => unsafe { env::set_var(key, v) },
            None => unsafe { env::remove_var(key) },
        };

        Self { key, previous }
    }
}

impl Drop for EnvGuard {
    fn drop(&mut self) {
        match &self.previous {
            Some(v) => unsafe { env::set_var(self.key, v) },
            None => unsafe { env::remove_var(self.key) },
        }
    }
}
