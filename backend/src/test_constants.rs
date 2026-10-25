//! 共有テスト定数
//!
//! 単体テスト・統合テストの両方から参照できる定数を定義する。

/// PostgreSQL コンテナのタグ。本番(Neon)と同じ系統の 16-alpine を固定で使用する。
pub const POSTGRES_TAG: &str = "16-alpine";
