//! 保有銘柄の識別色。`style/input.css` の `--color-holding-*` トークンに対応する
//! ユーティリティを表示位置から返す。Tailwind が静的に検出できるよう完全なクラス名で保持する。

const BARS: [&str; 20] = [
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
    "bg-holding-11",
    "bg-holding-12",
    "bg-holding-13",
    "bg-holding-14",
    "bg-holding-15",
    "bg-holding-16",
    "bg-holding-17",
    "bg-holding-18",
    "bg-holding-19",
    "bg-holding-20",
];

const BANDS: [&str; 20] = [
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
    "border-l-holding-11",
    "border-l-holding-12",
    "border-l-holding-13",
    "border-l-holding-14",
    "border-l-holding-15",
    "border-l-holding-16",
    "border-l-holding-17",
    "border-l-holding-18",
    "border-l-holding-19",
    "border-l-holding-20",
];

pub(crate) fn holding_bar_class(position: usize) -> &'static str {
    BARS[position % BARS.len()]
}

pub(crate) fn holding_band_class(position: usize) -> &'static str {
    BANDS[position % BANDS.len()]
}

#[cfg(test)]
mod tests;
