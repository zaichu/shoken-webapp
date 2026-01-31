use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

pub fn init_tracing() {
    tracing_subscriber::registry()
        .with(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "backend=info,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_module_compilation() {
        // ロギング初期化はグローバル状態を変更するため、
        // 複数テストでの呼び出しは避ける
        // このテストはモジュールのコンパイルを確認する
        assert!(true);
    }
}
