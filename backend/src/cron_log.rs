//! cron 実行の処理時間ログの形式と計測。ログに出すのは処理名・件数・経過 ms だけ。

use chrono::{DateTime, Timelike};

/// cron で順に回す処理。ログ名は固定値だけを使い、呼び出し側が任意文字列を渡せないようにする
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CronStep {
    DividendCache,
    StockPrice,
    FundPrice,
    StockMaster,
    LoginEvents,
}

impl CronStep {
    pub const fn name(self) -> &'static str {
        match self {
            Self::DividendCache => "dividend_cache",
            Self::StockPrice => "stock_price",
            Self::FundPrice => "fund_price",
            Self::StockMaster => "stock_master",
            Self::LoginEvents => "login_events",
        }
    }
}

/// 完了ログの形式。件数と計測済みの経過 ms だけを含める
pub fn format_cron_step(step: CronStep, count: usize, elapsed_ms: u64) -> String {
    format!("cron {}: {count}件 {elapsed_ms}ms", step.name())
}

/// 何もしなかった回を1行に抑えるためのログ
pub fn format_cron_noop(elapsed_ms: u64) -> String {
    format!("cron noop: {elapsed_ms}ms")
}

/// 接続ロールと共有表の RLS 状態を毎分出すと多いため、10 分に 1 回だけ出す
#[expect(clippy::cast_possible_truncation)]
pub fn should_log_db_access_at(scheduled_ms: f64) -> bool {
    DateTime::from_timestamp_millis(scheduled_ms as i64)
        .is_some_and(|t| t.minute().is_multiple_of(10))
}

/// 接続ロール・共有表の RLS・限定書き込み関数の所有者・event trigger を 1 行ずつ返す。
/// 出すのはロール名・表名・関数名・真偽値・件数だけ
pub const DB_ACCESS_STATE_SQL: &str = r"
SELECT pg_catalog.format('session current_user=%s rolbypassrls=%s rolsuper=%s',
           r.rolname, r.rolbypassrls::text, r.rolsuper::text)
FROM pg_catalog.pg_roles r WHERE r.rolname = current_user
UNION ALL
SELECT CASE WHEN c.oid IS NULL THEN pg_catalog.format('table %s missing', t.name)
       ELSE pg_catalog.format('table %s relrowsecurity=%s relforcerowsecurity=%s policies=%s select=%s',
           t.name, c.relrowsecurity::text, c.relforcerowsecurity::text,
           (SELECT pg_catalog.count(*) FROM pg_catalog.pg_policy p WHERE p.polrelid = c.oid),
           pg_catalog.has_table_privilege(c.oid, 'SELECT')::text) END
FROM pg_catalog.unnest(ARRAY['dividend_per_share_cache', 'market_data_provider_rate_control',
         'stock', 'security_prices', 'fund_prices', 'fund_codes']) AS t(name)
LEFT JOIN pg_catalog.pg_class c
    ON c.relname = t.name AND c.relnamespace = 'public'::pg_catalog.regnamespace
UNION ALL
SELECT pg_catalog.format('function %s owner_bypassrls=%s', f.proname, o.rolbypassrls::text)
FROM pg_catalog.pg_proc f JOIN pg_catalog.pg_roles o ON o.oid = f.proowner
WHERE f.pronamespace = 'public'::pg_catalog.regnamespace
  AND f.proname IN ('dividend_cache_enqueue_v2', 'dividend_cache_apply_success_v2',
      'dividend_cache_apply_error_v2', 'rate_control_acquire_jquants_v2',
      'rate_control_cooldown_v2', 'stock_master_claim_v2', 'stock_master_postpone_v2',
      'stock_master_replace_v2')
UNION ALL
SELECT pg_catalog.format('event_triggers count=%s names=%s', pg_catalog.count(*),
           COALESCE(pg_catalog.string_agg(e.evtname::text, ',' ORDER BY e.evtname), '-'))
FROM pg_catalog.pg_event_trigger e
";

/// BulkTimer と同じ方式の計測。wasm では Instant が未対応のため epoch ミリ秒で保持する
pub struct CronTimer {
    #[cfg(not(target_arch = "wasm32"))]
    start: std::time::Instant,
    #[cfg(target_arch = "wasm32")]
    start_ms: u64,
}

impl CronTimer {
    pub fn start() -> Self {
        Self {
            #[cfg(not(target_arch = "wasm32"))]
            start: std::time::Instant::now(),
            #[cfg(target_arch = "wasm32")]
            start_ms: worker::Date::now().as_millis(),
        }
    }

    pub fn elapsed_ms(&self) -> u64 {
        #[cfg(not(target_arch = "wasm32"))]
        return self
            .start
            .elapsed()
            .as_millis()
            .try_into()
            .unwrap_or(u64::MAX);
        #[cfg(target_arch = "wasm32")]
        return worker::Date::now()
            .as_millis()
            .saturating_sub(self.start_ms);
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;

    #[test]
    fn step_names_are_fixed_ascii() {
        let steps = [
            CronStep::DividendCache,
            CronStep::StockPrice,
            CronStep::FundPrice,
            CronStep::StockMaster,
            CronStep::LoginEvents,
        ];
        assert_eq!(
            steps.map(CronStep::name),
            [
                "dividend_cache",
                "stock_price",
                "fund_price",
                "stock_master",
                "login_events"
            ]
        );
        for name in steps.map(CronStep::name) {
            assert!(
                name.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'_'),
                "step name must stay grep-safe: {name}"
            );
        }
    }

    #[test]
    fn step_log_format() {
        assert_eq!(
            format_cron_step(CronStep::DividendCache, 3, 42),
            "cron dividend_cache: 3件 42ms"
        );
        assert_eq!(
            format_cron_step(CronStep::StockPrice, 0, 0),
            "cron stock_price: 0件 0ms"
        );
        assert_eq!(
            format_cron_step(CronStep::StockMaster, 10000, 123456),
            "cron stock_master: 10000件 123456ms"
        );
    }

    #[test]
    fn noop_log_format() {
        assert_eq!(format_cron_noop(5), "cron noop: 5ms");
        assert_eq!(format_cron_noop(0), "cron noop: 0ms");
    }

    #[test]
    fn step_log_carries_no_personal_data() {
        for step in [
            CronStep::DividendCache,
            CronStep::StockPrice,
            CronStep::FundPrice,
            CronStep::StockMaster,
            CronStep::LoginEvents,
        ] {
            let log = format_cron_step(step, usize::MAX, u64::MAX);
            for fragment in [
                "@", "?", "=", "code=", "user", "key", "token", "http", "/", ".",
            ] {
                assert!(
                    !log.contains(fragment),
                    "log must not carry identifiers: {log}"
                );
            }
        }
        let noop = format_cron_noop(u64::MAX);
        assert!(!noop.contains('@'));
    }

    #[test]
    #[expect(clippy::cast_precision_loss)]
    fn db_access_log_runs_every_ten_minutes() {
        let at = |h: u32, m: u32| {
            chrono::NaiveDate::from_ymd_opt(2026, 11, 17)
                .unwrap()
                .and_hms_opt(h, m, 0)
                .unwrap()
                .and_utc()
                .timestamp_millis() as f64
        };
        assert!(should_log_db_access_at(at(12, 0)));
        assert!(should_log_db_access_at(at(12, 50)));
        assert!(!should_log_db_access_at(at(12, 1)));
        assert!(!should_log_db_access_at(at(12, 9)));
        assert_eq!(
            (0..60)
                .filter(|m| should_log_db_access_at(at(3, *m)))
                .count(),
            6
        );
    }

    #[test]
    fn timer_elapsed_is_bounded() {
        let timer = CronTimer::start();
        assert!(timer.elapsed_ms() < 60_000);
    }
}
