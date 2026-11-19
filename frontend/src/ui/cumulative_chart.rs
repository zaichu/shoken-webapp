//! 受取の累計の折れ線グラフ。ホームの配当カードで使う。
//! 今年の累計(実線・アクセントの青)と前年の累計(点線・灰)を重ねる。
//! 前年は12月まで、今年は今月まで。縦軸の目盛り・吹き出し・
//! 読み上げ用の表までを部品が持つ。目盛りの段数は axis_ticks が決め、
//! 線の高さは目盛りの上限にそろえる。

use crate::features::asset_balance;
use crate::ui::amount::{Amount, AmountKind};
use crate::ui::format;
use crate::ui::labels;
use leptos::prelude::*;

/// 縦軸の目盛り(0 を含むきりのよい 3〜4 本)。上限が最大値を覆う最小の段になる
pub fn axis_ticks(max: f64) -> Vec<f64> {
    if !max.is_finite() || max <= 0.0 {
        return vec![0.0, 5_000.0, 10_000.0];
    }
    let min_step = if max >= 10_000.0 { 10_000.0 } else { 0.0 };
    let mut best: Option<(f64, Vec<f64>)> = None;
    for intervals in [2u32, 3u32] {
        let step = nice_step(max / f64::from(intervals), min_step);
        let top = step * f64::from(intervals);
        let ticks: Vec<f64> = (0..=intervals)
            .map(|index| step * f64::from(index))
            .collect();
        best = Some(match best {
            Some((best_top, best_ticks)) if best_top < top => (best_top, best_ticks),
            _ => (top, ticks),
        });
    }
    best.map_or_else(|| vec![0.0], |(_, ticks)| ticks)
}

fn nice_step(raw: f64, min_step: f64) -> f64 {
    let magnitude = 10f64.powf(raw.log10().floor()).max(1.0);
    [1.0, 2.0, 5.0, 10.0]
        .into_iter()
        .map(|factor| factor * magnitude)
        .find(|step| *step >= raw && *step >= min_step)
        .unwrap_or(10.0 * magnitude)
}

/// 月別入金の累計(running total)。有限でない月は 0 として足す
pub fn cumulative_sums(monthly: &[f64]) -> Vec<f64> {
    let mut total = 0.0;
    monthly
        .iter()
        .map(|value| {
            if value.is_finite() {
                total += value;
            }
            total
        })
        .collect()
}

/// 前年同期の点の差 = 今年の累計 - 前年の累計。前年なし(None)のときは None
pub fn yoy_difference(current_cum: f64, previous_cum: Option<f64>) -> Option<f64> {
    previous_cum.map(|previous| current_cum - previous)
}

/// 読み上げ表の今年の欄。未来の月はまだ受取がないため None(「—」で出す)
pub fn table_current_text(month: u32, current_month: u32, cumulative: &[f64]) -> Option<String> {
    if month == 0 || month > current_month {
        return None;
    }
    cumulative
        .get(month as usize - 1)
        .map(|value| format::format_currency(*value))
}

/// 読み上げ表の月の受取の欄。未来の月はまだ受取がないため None(「—」で出す)
pub fn table_monthly_text(month: u32, current_month: u32, monthly: &[f64]) -> Option<String> {
    if month == 0 || month > current_month {
        return None;
    }
    monthly
        .get(month as usize - 1)
        .map(|value| format::format_currency(*value))
}

fn x_percent(month: u32) -> f64 {
    f64::from(month.saturating_sub(1)) / 11.0 * 100.0
}

fn y_percent(value: f64, top: f64) -> f64 {
    if top <= 0.0 || !value.is_finite() {
        return 100.0;
    }
    ((1.0 - value / top) * 100.0).clamp(0.0, 100.0)
}

fn line_points(cumulative: &[f64], top: f64) -> String {
    cumulative
        .iter()
        .enumerate()
        .map(|(index, value)| {
            let month = u32::try_from(index + 1).unwrap_or(u32::MAX);
            format!("{:.2},{:.2}", x_percent(month), y_percent(*value, top))
        })
        .collect::<Vec<_>>()
        .join(" ")
}

#[component]
#[expect(
    clippy::needless_pass_by_value,
    reason = "Leptos component props are owned by the view"
)]
pub fn CumulativeChart(
    /// 今年12か月分の入金(月別)。1月から順に12件
    current: Vec<f64>,
    /// 前年12か月分の入金(月別)。取得失敗のとき None で線と差を出さない
    previous: Option<Vec<f64>>,
    current_year: u32,
    previous_year: u32,
    #[prop(into)] aria_label: String,
    /// 今月(1..=12)。今年の線と押せる点はここまで
    current_month: u32,
) -> impl IntoView {
    let current_month = current_month.clamp(1, 12);
    let current_cum = cumulative_sums(&current);
    let previous_cum = previous.as_ref().map(|monthly| cumulative_sums(monthly));
    let max = current_cum
        .iter()
        .take(current_month as usize)
        .chain(previous_cum.iter().flat_map(|sums| sums.iter()))
        .filter(|value| value.is_finite())
        .fold(0.0f64, |a, b| a.max(*b));
    let ticks = axis_ticks(max);
    let top = ticks.last().copied().unwrap_or(0.0);
    let selected = RwSignal::new(None::<u32>);
    let current_label = labels::cumulative_year_label(current_year);
    let previous_label = labels::cumulative_year_label(previous_year);
    let current_line = line_points(
        &current_cum[..(current_month as usize).min(current_cum.len())],
        top,
    );
    let previous_line = previous_cum.as_ref().map(|sums| line_points(sums, top));
    let end_diff = current_cum
        .get(current_month as usize - 1)
        .copied()
        .and_then(|current| {
            yoy_difference(
                current,
                previous_cum
                    .as_ref()
                    .and_then(|sums| sums.get(current_month as usize - 1).copied()),
            )
        });
    let end_diff_text =
        end_diff.map(|value| asset_balance::format_signed_valuation_amount(Some(value)));
    let end_diff_negative =
        end_diff.is_some_and(|value| asset_balance::is_negative_valuation(Some(value)));
    let end_diff_positive =
        end_diff.is_some_and(|value| asset_balance::is_positive_valuation(Some(value)));

    let months: Vec<u32> = (1..=12).collect();
    view! {
        <div class="home-cum-legend" aria-hidden="true">
            <span class="home-cum-legend-item">
                <span class="home-cum-key home-cum-key-solid"></span>
                {current_label.clone()}
            </span>
            {previous_cum.is_some().then(|| view! {
                <span class="home-cum-legend-item">
                    <span class="home-cum-key home-cum-key-dash"></span>
                    {previous_label.clone()}
                </span>
            })}
        </div>
        <div class="home-chart">
            <div class="home-cum-axis" aria-hidden="true">
                {ticks
                    .iter()
                    .rev()
                    .map(|tick| {
                        let position = if top > 0.0 { (1.0 - tick / top) * 100.0 } else { 100.0 };
                        view! {
                            <span class="home-cum-tick" style:top=format!("{position}%")>
                                {labels::axis_yen_short(*tick)}
                            </span>
                        }
                    })
                    .collect_view()}
            </div>
            <div class="home-chart-plot">
                <div class="home-chart-grid" aria-hidden="true">
                    {ticks
                        .iter()
                        .map(|tick| {
                            let position = if top > 0.0 {
                                (1.0 - tick / top) * 100.0
                            } else {
                                100.0
                            };
                            view! {
                                <span class="home-chart-grid-line" style:top=format!("{position}%") />
                            }
                        })
                        .collect_view()}
                </div>
                <svg
                    class="home-cum-lines"
                    viewBox="0 0 100 100"
                    preserveAspectRatio="none"
                    aria-hidden="true"
                >
                    {previous_line
                        .map(|points| view! {
                            <polyline class="home-cum-line-previous" points=points />
                        })}
                    <polyline class="home-cum-line-current" points=current_line />
                </svg>
                <div class="home-cum-cols">
                    {months
                    .iter()
                    .map(|month| {
                        let month = *month;
                        if month > current_month {
                            // 未来の月も同じ幅の枠だけ置く。線の12か月座標と
                            // 点・押せる範囲・月ラベルをずらさないため
                            return view! {
                                <div class="home-cum-col" aria-hidden="true"></div>
                            }
                                .into_any();
                        }
                        let current_value = current_cum.get(month as usize - 1).copied().unwrap_or(0.0);
                        let monthly_text = format::format_currency(
                            current.get(month as usize - 1).copied().unwrap_or(0.0),
                        );
                        let receipt_label = labels::cumulative_month_receipt_label(month);
                        let previous_value = previous_cum
                            .as_ref()
                            .and_then(|sums| sums.get(month as usize - 1).copied());
                        let current_text = format::format_currency(current_value);
                        let previous_text = previous_value.map(format::format_currency);
                        let month_aria = labels::home_cumulative_month_aria(
                            month,
                            if month == current_month { labels::HOME_CURRENT_MONTH } else { "" },
                            current_year,
                            &monthly_text,
                            &current_text,
                            previous_value.map(|_| previous_year),
                            previous_text.as_deref(),
                        );
                        let month_label = labels::receipt_month_label(month);
                        let current_cell = current_label.clone();
                        let previous_cell = previous_label.clone();
                        let dot_top = y_percent(current_value, top);
                        let dot_left = x_percent(month);
                        let prev_dot = previous_value.map(|value| {
                            let top_pos = y_percent(value, top);
                            let left_pos = x_percent(month);
                            view! {
                                <span
                                    class="home-cum-dot home-cum-dot-previous"
                                    style:top=format!("{top_pos}%")
                                    style:left=format!("{left_pos}%")
                                />
                            }
                        });
                        view! {
                            <div
                                class="home-cum-col"
                                tabindex="0"
                                role="button"
                                aria-expanded=move || (selected.get() == Some(month)).to_string()
                                aria-label=month_aria
                                data-selected=move || (selected.get() == Some(month)).to_string()
                                on:click=move |_| {
                                    selected.update(|state| {
                                        *state = if *state == Some(month) { None } else { Some(month) };
                                    });
                                }
                                on:keydown=move |event| {
                                    if event.key() == "Enter" || event.key() == " " {
                                        event.prevent_default();
                                        selected.update(|state| {
                                            *state = if *state == Some(month) { None } else { Some(month) };
                                        });
                                    } else if event.key() == "Escape" {
                                        selected.set(None);
                                    }
                                }
                            >
                                {prev_dot}
                                <span
                                    class="home-cum-dot home-cum-dot-current"
                                    style:top=format!("{dot_top}%")
                                    style:left=format!("{dot_left}%")
                                />
                                <div class="home-cum-tip" aria-hidden="true">
                                    <p class="font-bold text-text">{month_label}</p>
                                    <dl class="mt-1 space-y-0.5">
                                        <div class="flex items-center gap-1.5">
                                            <dt class="text-text-muted">{receipt_label}</dt>
                                            <dd><Amount text=monthly_text /></dd>
                                        </div>
                                        <div class="flex items-center gap-1.5">
                                            <dt class="home-cum-key home-cum-key-solid"></dt>
                                            <dd>{current_cell}<Amount text=current_text /></dd>
                                        </div>
                                        {previous_text
                                            .map(|text| {
                                                let cell = previous_cell.clone();
                                                view! {
                                                    <div class="flex items-center gap-1.5">
                                                        <dt class="home-cum-key home-cum-key-dash"></dt>
                                                        <dd>{cell}<Amount text=text /></dd>
                                                    </div>
                                                }
                                            })}
                                    </dl>
                                </div>
                            </div>
                        }
                            .into_any()
                    })
                    .collect_view()}
                </div>
            </div>
        </div>
        <div class="home-chart-month-labels" aria-hidden="true">
            {months
                .iter()
                .map(|month| {
                    let is_current = *month == current_month;
                    view! {
                        <span class="home-chart-month-label" data-current=is_current.to_string()>
                            {labels::receipt_month_label(*month)}
                        </span>
                    }
                })
                .collect_view()}
        </div>
        {end_diff_text
            .map(|text| {
                view! {
                    <p class="home-cum-yoy" aria-hidden="true">
                        {labels::HOME_YOY_COMPARE_LABEL}
                        <Amount
                            kind=AmountKind::Delta
                            text=text
                            negative=end_diff_negative
                            positive=end_diff_positive
                        />
                    </p>
                }
            })}
        <table class="home-cum-table sr-only">
            <caption>{aria_label}</caption>
            <thead>
                <tr>
                    <th scope="col">{labels::MONTH}</th>
                    <th scope="col">{labels::CUMULATIVE_MONTHLY_RECEIPT}</th>
                    <th scope="col">{current_label}</th>
                    {previous_cum.is_some().then(|| view! {
                        <th scope="col">{previous_label.clone()}</th>
                    })}
                </tr>
            </thead>
            <tbody>
                {(1..=12u32)
                    .map(|month| {
                        let monthly_text = table_monthly_text(month, current_month, &current)
                            .unwrap_or_else(|| labels::MISSING.to_string());
                        let current_text = table_current_text(month, current_month, &current_cum)
                            .unwrap_or_else(|| labels::MISSING.to_string());
                        let previous_text = previous_cum
                            .as_ref()
                            .and_then(|sums| sums.get(month as usize - 1).copied())
                            .map(format::format_currency);
                        view! {
                            <tr>
                                <th scope="row">{labels::receipt_month_label(month)}</th>
                                <td>{monthly_text}</td>
                                <td>{current_text}</td>
                                {previous_text
                                    .map(|text| view! { <td>{text}</td> })}
                            </tr>
                        }
                    })
                    .collect_view()}
            </tbody>
        </table>
    }
}

#[cfg(test)]
mod tests;
