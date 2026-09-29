//! 保有銘柄の識別色。`style/input.css` の `--color-holding-*` トークンに対応する
//! ユーティリティを表示位置から返す。Tailwind が静的に検出できるよう完全なクラス名で保持する。

const BARS: [&str; 10] = [
    "bg-holding-1",
    "bg-holding-2",
    "bg-holding-3",
    "bg-holding-4",
    "bg-holding-5",
    "bg-holding-6",
    "bg-holding-7",
    "bg-holding-8",
    "bg-holding-9",
    "bg-holding-10",
];

const BANDS: [&str; 10] = [
    "border-l-holding-1",
    "border-l-holding-2",
    "border-l-holding-3",
    "border-l-holding-4",
    "border-l-holding-5",
    "border-l-holding-6",
    "border-l-holding-7",
    "border-l-holding-8",
    "border-l-holding-9",
    "border-l-holding-10",
];

/// 構成比バー・凡例ドットの塗り色。
pub(crate) fn holding_bar_class(position: usize) -> &'static str {
    BARS[position % BARS.len()]
}

/// 保有カード左端の帯に使うボーダー色。
pub(crate) fn holding_band_class(position: usize) -> &'static str {
    BANDS[position % BANDS.len()]
}

#[cfg(test)]
mod tests;
