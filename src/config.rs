use std::env;

pub struct Config {
    pub addr: String,
}

pub fn get_config() -> Config {
    let port = env::var("PORT").unwrap_or_else(|_| "8080".to_string());
    let addr = format!("0.0.0.0:{}", port);

    Config { addr }
}
