//! cron 実行の処理時間ログの形式と計測。ログに出すのは処理名・件数・経過 ms だけ。

/// cron で順に回す処理。ログ名は固定値だけを使い、呼び出し側が任意文字列を渡せないようにする
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CronStep {
    DividendCache,
    StockPrice,
    FundPrice,
    StockMaster,
}

impl CronStep {
    pub fn name(self) -> &'static str {
        match self {
            Self::DividendCache => "dividend_cache",
            Self::StockPrice => "stock_price",
            Self::FundPrice => "fund_price",
            Self::StockMaster => "stock_master",
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
mod tests {
    use super::*;

    #[test]
    fn step_names_are_fixed_ascii() {
        let steps = [
            CronStep::DividendCache,
            CronStep::StockPrice,
            CronStep::FundPrice,
            CronStep::StockMaster,
        ];
        assert_eq!(
            steps.map(CronStep::name),
            [
                "dividend_cache",
                "stock_price",
                "fund_price",
                "stock_master"
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
    fn timer_elapsed_is_bounded() {
        let timer = CronTimer::start();
        assert!(timer.elapsed_ms() < 60_000);
    }
}
