/// 開発用として明示設定しうる環境名。
/// これ以外の値(未設定・不明値を含む)は安全側に本番として扱う
const DEVELOPMENT_ENV_VALUES: &[&str] = &["local", "dev", "development", "test"];

/// 実行環境の判定結果(fail-safe)
/// RUST_ENV / APP_ENV に設定された値がすべて開発用の値のときだけ `Development`。
/// 未設定・不明値・本番値との混在はすべて `Production` にし、環境変数の設定漏れや
/// 書き間違いでセキュリティ設定が緩まないようにする。
/// BACKEND_URL のスキームには依存しない
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum RuntimeEnv {
    #[default]
    Production,
    Development,
}

impl RuntimeEnv {
    /// RUST_ENV / APP_ENV の値から解決する。fail-safe のため
    /// 呼び出し側は取得した生の値をそのまま渡す
    pub fn resolve(rust_env: Option<&str>, app_env: Option<&str>) -> Self {
        let mut any_set = false;
        for value in [rust_env, app_env].into_iter().flatten() {
            if !DEVELOPMENT_ENV_VALUES.contains(&value) {
                return Self::Production;
            }
            any_set = true;
        }
        if any_set {
            Self::Development
        } else {
            Self::Production
        }
    }

    pub fn is_production(self) -> bool {
        matches!(self, Self::Production)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_runtime_env_resolve() {
        // (RUST_ENV, APP_ENV, expected)
        // fail-safe: 未設定・不明値・非開発値の混在はすべて本番扱い。
        // 設定値がすべて開発用の値のときだけ非本番。
        let cases = [
            (Some("production"), None, RuntimeEnv::Production),
            (None, Some("production"), RuntimeEnv::Production),
            (None, None, RuntimeEnv::Production),
            (
                Some("development"),
                Some("production"),
                RuntimeEnv::Production,
            ),
            (
                Some("production"),
                Some("development"),
                RuntimeEnv::Production,
            ),
            (Some("development"), Some("staging"), RuntimeEnv::Production),
            (Some("development"), None, RuntimeEnv::Development),
            (None, Some("local"), RuntimeEnv::Development),
            (None, Some("test"), RuntimeEnv::Development),
            (Some("dev"), Some("development"), RuntimeEnv::Development),
        ];
        for (rust_env, app_env, expected) in cases {
            assert_eq!(
                RuntimeEnv::resolve(rust_env, app_env),
                expected,
                "RUST_ENV={rust_env:?} APP_ENV={app_env:?}"
            );
        }
    }
}
