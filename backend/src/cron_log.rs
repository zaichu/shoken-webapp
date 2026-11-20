//! cron 実行の処理時間ログの形式と計測。ログに出すのは処理名・件数・経過 ms だけ。

use time::OffsetDateTime;

/// cron で順に回す処理。ログ名は固定値だけを使い、呼び出し側が任意文字列を渡せないようにする
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CronStep {
    DividendCache,
    StockPrice,
    FundPrice,
    StockMaster,
    LoginEvents,
    OperationLogs,
}

impl CronStep {
    pub const fn name(self) -> &'static str {
        match self {
            Self::DividendCache => "dividend_cache",
            Self::StockPrice => "stock_price",
            Self::FundPrice => "fund_price",
            Self::StockMaster => "stock_master",
            Self::LoginEvents => "login_events",
            Self::OperationLogs => "operation_logs",
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
    OffsetDateTime::from_unix_timestamp_nanos(scheduled_ms as i128 * 1_000_000)
        .is_ok_and(|t| t.minute().is_multiple_of(10))
}

/// 接続ロール・共有表の RLS・限定書き込み関数の所有者・event trigger を 1 行ずつ返す。
/// 出すのはロール名・表名・関数名・真偽値・件数だけ
/// anon・authenticated・service_role が public の表・ビュー・関数・シーケンスに持つ権限の
/// 件数もロールごとに 1 行出す。無いロールは 0 件にし、名前指定の has_* 呼び出しはしない
/// (ロール不在で検査自体が落ちるため pg_roles 経由で oid を解決する)。
/// 末尾の range_check_violations は 0119 の CHECK 制約(dividend_per_share_cache。#2112 で VALIDATE 予定)と
/// #2112 で追加予定の shares > 0 制約に先立つ違反件数だけを出す(値・利用者は出さない)。
/// 個人表への直接参照は権限のないロールで検査自体が落ちるため、
/// 件数の取得は SECURITY DEFINER の range_check_violation_counts_v2 に閉じ込める
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
UNION ALL
SELECT pg_catalog.format('role_memberships roles=%s',
           COALESCE((WITH RECURSIVE membership(role_oid) AS (
               SELECT m.roleid FROM pg_catalog.pg_auth_members m
               WHERE m.member = (SELECT oid FROM pg_catalog.pg_roles WHERE rolname = current_user)
               UNION
               SELECT m.roleid FROM pg_catalog.pg_auth_members m
               JOIN membership ON membership.role_oid = m.member
           ) SELECT pg_catalog.string_agg(r.rolname, ',' ORDER BY r.rolname)
           FROM membership JOIN pg_catalog.pg_roles r ON r.oid = membership.role_oid), '-'))
UNION ALL
SELECT CASE WHEN o.oid IS NULL
       THEN pg_catalog.format('table_owner %s owner=- member=false can_set=false', t.name)
       ELSE pg_catalog.format('table_owner %s owner=%s member=%s can_set=%s', t.name, o.rolname,
           pg_catalog.pg_has_role(current_user, o.oid, 'MEMBER')::text,
           pg_catalog.pg_has_role(current_user, o.oid, 'SET')::text) END
FROM (VALUES ('users'), ('sessions'), ('dividends'), ('domestic_stocks'), ('mutualfunds'),
             ('asset_balances'), ('fund_balances'), ('csv_import_history'), ('login_events'),
             ('login_notify_tokens'), ('admin_login_views')) AS t(name)
LEFT JOIN pg_catalog.pg_class c
    ON c.relname = t.name AND c.relnamespace = 'public'::pg_catalog.regnamespace
LEFT JOIN pg_catalog.pg_roles o ON o.oid = c.relowner
UNION ALL
SELECT CASE WHEN o.oid IS NULL THEN 'role_owner postgres member=- can_set=-'
       ELSE pg_catalog.format('role_owner %s member=%s can_set=%s', 'postgres',
           pg_catalog.pg_has_role(current_user, o.oid, 'MEMBER')::text,
           pg_catalog.pg_has_role(current_user, o.oid, 'SET')::text) END
FROM (SELECT 1) AS dummy
LEFT JOIN pg_catalog.pg_roles o ON o.rolname = 'postgres'
UNION ALL
SELECT pg_catalog.format('default_privilege_grants role=%s table=%s view=%s function=%s sequence=%s',
           v.rolname,
           CASE WHEN r.oid IS NULL THEN 0 ELSE (SELECT pg_catalog.count(*)
               FROM pg_catalog.pg_class c JOIN pg_catalog.pg_namespace n ON n.oid = c.relnamespace
               WHERE n.nspname = 'public' AND c.relkind IN ('r', 'p')
                 AND (pg_catalog.has_table_privilege(r.oid, c.oid, 'SELECT')
                   OR pg_catalog.has_table_privilege(r.oid, c.oid, 'INSERT')
                   OR pg_catalog.has_table_privilege(r.oid, c.oid, 'UPDATE')
                   OR pg_catalog.has_table_privilege(r.oid, c.oid, 'DELETE')
                   OR pg_catalog.has_table_privilege(r.oid, c.oid, 'TRUNCATE')
                   OR pg_catalog.has_table_privilege(r.oid, c.oid, 'REFERENCES')
                   OR pg_catalog.has_table_privilege(r.oid, c.oid, 'TRIGGER'))) END,
           CASE WHEN r.oid IS NULL THEN 0 ELSE (SELECT pg_catalog.count(*)
               FROM pg_catalog.pg_class c JOIN pg_catalog.pg_namespace n ON n.oid = c.relnamespace
               WHERE n.nspname = 'public' AND c.relkind IN ('v', 'm')
                 AND (pg_catalog.has_table_privilege(r.oid, c.oid, 'SELECT')
                   OR pg_catalog.has_table_privilege(r.oid, c.oid, 'INSERT')
                   OR pg_catalog.has_table_privilege(r.oid, c.oid, 'UPDATE')
                   OR pg_catalog.has_table_privilege(r.oid, c.oid, 'DELETE')
                   OR pg_catalog.has_table_privilege(r.oid, c.oid, 'TRUNCATE')
                   OR pg_catalog.has_table_privilege(r.oid, c.oid, 'REFERENCES')
                   OR pg_catalog.has_table_privilege(r.oid, c.oid, 'TRIGGER'))) END,
           CASE WHEN r.oid IS NULL THEN 0 ELSE (SELECT pg_catalog.count(*)
               FROM pg_catalog.pg_proc p
               WHERE p.pronamespace = 'public'::pg_catalog.regnamespace
                 AND pg_catalog.has_function_privilege(r.oid, p.oid, 'EXECUTE')) END,
           CASE WHEN r.oid IS NULL THEN 0 ELSE (SELECT pg_catalog.count(*)
               FROM pg_catalog.pg_class c JOIN pg_catalog.pg_namespace n ON n.oid = c.relnamespace
               WHERE n.nspname = 'public' AND c.relkind = 'S'
                 AND (pg_catalog.has_sequence_privilege(r.oid, c.oid, 'USAGE')
                   OR pg_catalog.has_sequence_privilege(r.oid, c.oid, 'SELECT')
                   OR pg_catalog.has_sequence_privilege(r.oid, c.oid, 'UPDATE'))) END)
FROM (VALUES ('anon'), ('authenticated'), ('service_role')) AS v(rolname)
LEFT JOIN pg_catalog.pg_roles r ON r.rolname = v.rolname
UNION ALL
SELECT pg_catalog.format('range_check_violations %s', public.range_check_violation_counts_v2())
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
            CronStep::OperationLogs,
        ];
        assert_eq!(
            steps.map(CronStep::name),
            [
                "dividend_cache",
                "stock_price",
                "fund_price",
                "stock_master",
                "login_events",
                "operation_logs"
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
            CronStep::OperationLogs,
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
            time::Date::from_calendar_date(2026, time::Month::try_from(11).unwrap(), 17)
                .unwrap()
                .with_hms(u8::try_from(h).unwrap(), u8::try_from(m).unwrap(), 0)
                .unwrap()
                .assume_utc()
                .unix_timestamp_nanos() as f64
                / 1_000_000.0
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

    #[test]
    fn db_access_state_tolerates_missing_postgres_role() {
        assert!(
            !DB_ACCESS_STATE_SQL.contains("pg_has_role(current_user, 'postgres'"),
            "名前指定の pg_has_role はロール不在で例外になるため pg_roles 経由で解決する"
        );
        for fragment in [
            "has_table_privilege('anon'",
            "has_table_privilege('authenticated'",
            "has_table_privilege('service_role'",
            "has_function_privilege('anon'",
            "has_function_privilege('authenticated'",
            "has_function_privilege('service_role'",
            "has_sequence_privilege('anon'",
            "has_sequence_privilege('authenticated'",
            "has_sequence_privilege('service_role'",
        ] {
            assert!(
                !DB_ACCESS_STATE_SQL.contains(fragment),
                "名前指定の has_* はロール不在で例外になるため pg_roles 経由の oid で調べる: {fragment}"
            );
        }
    }

    #[test]
    fn db_access_state_reports_default_privilege_grants_without_personal_data() {
        assert!(
            DB_ACCESS_STATE_SQL.contains("default_privilege_grants"),
            "Supabase 3 ロールの権限件数行を出す"
        );
        for role in ["anon", "authenticated", "service_role"] {
            assert!(
                DB_ACCESS_STATE_SQL.contains(role),
                "ロールごとに件数を出す: {role}"
            );
        }
        for kind in ["table=", "view=", "function=", "sequence="] {
            assert!(
                DB_ACCESS_STATE_SQL.contains(kind),
                "種類ごとに件数を出す: {kind}"
            );
        }
        for fragment in [
            "user_id",
            "email",
            "token_hash",
            "device_hash",
            "security_code",
        ] {
            assert!(
                !DB_ACCESS_STATE_SQL.contains(fragment),
                "件数だけを出し値・利用者は出さない: {fragment}"
            );
        }
    }

    #[test]
    fn db_access_state_reports_range_check_violations_without_personal_data() {
        // #2112 の VALIDATE 前に本番の違反件数を cron ログで確かめるための行
        assert!(
            DB_ACCESS_STATE_SQL.contains("range_check_violations"),
            "CHECK 違反の件数行を出す"
        );
        for fragment in [
            "user_id",
            "email",
            "token_hash",
            "device_hash",
            "security_code",
        ] {
            assert!(
                !DB_ACCESS_STATE_SQL.contains(fragment),
                "件数だけを出し値・利用者は出さない: {fragment}"
            );
        }
    }
}
