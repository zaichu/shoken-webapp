//! 受取の累計の折れ線グラフ。ホームの配当カードで使う。
//! 今年の累計(実線・アクセントの青)と前年の累計(点線・灰)を重ねる。
//! 前年は12月まで、今年は今月まで。縦軸の目盛り・吹き出し・
//! 読み上げ用の表までを部品が持つ。目盛りの段数は axis_ticks が決め、
//! 線の高さは目盛りの上限にそろえる。

use crate::ui::amount::Amount;
use crate::ui::format;
use crate::ui::labels;
use leptos::prelude::*;
use wasm_bindgen::JsCast;
use wasm_bindgen::closure::Closure;

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

fn area_points(cumulative: &[f64], top: f64) -> String {
    if cumulative.is_empty() {
        return String::new();
    }
    let mut points = line_points(cumulative, top);
    let end_x = x_percent(u32::try_from(cumulative.len()).unwrap_or(u32::MAX));
    points.push_str(&format!(" {end_x:.2},100.00 0.00,100.00"));
    points
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

/// タップ位置から最も近い月を返す。x は描画域の左端からの距離、width は描画域の幅。
/// 12等分の境界で分け、描画域の外は端に寄せる。選べる月は 1..=current_month のため、
/// 未来月の領域を押したときは今月に寄せる
pub fn month_for_position(x: f64, width: f64, current_month: u32) -> u32 {
    let max = current_month.clamp(1, 12);
    if !x.is_finite() || !width.is_finite() || width <= 0.0 {
        return 1;
    }
    let scaled = (x / width * 12.0).floor();
    if scaled < 0.0 {
        return 1;
    }
    (1..=max)
        .find(|month| scaled < f64::from(*month))
        .unwrap_or(max)
}

/// クリック後の選択。ホバー可能なPCではホバーが既に同じ月を選んでいるため、
/// 同じ月のクリックでも表示を維持する。タッチでは同月タップで開閉する
pub fn next_selected(current: Option<u32>, month: u32, hover: bool) -> Option<u32> {
    if hover {
        return Some(month);
    }
    if current == Some(month) {
        None
    } else {
        Some(month)
    }
}

/// 吹き出しの横位置。端の月はグラフの内側に寄せる。
/// 狭い画面では中央の月でも吹き出しが柱より広いため、左の3か月を左寄せにする。
/// 640未満は Tailwind の max-sm と同じ境目
pub const fn tip_edge(month: u32, narrow: bool) -> &'static str {
    if month == 1 || (narrow && month <= 3) {
        "left"
    } else if month >= 10 {
        "right"
    } else {
        ""
    }
}

/// 吹き出しを点の下に出すか。点の少し上を基本にし、上の丸より上に
/// 吹き出しが入らないときは下の丸の少し下に出す。
/// 今年・前年の丸をどちらも隠さないよう、上下の丸の両方を基準にする
pub fn tip_below(current_top: f64, previous_top: Option<f64>) -> bool {
    let highest = previous_top.map_or(current_top, |previous| current_top.min(previous));
    highest < 50.0
}

/// スライダーのキー操作で月を1つ動かす。範囲外は端に寄せる
pub fn step_month(month: u32, delta: i32, current_month: u32) -> u32 {
    let max = current_month.clamp(1, 12);
    let next = i64::from(month) + i64::from(delta);
    if next <= 1 {
        return 1;
    }
    if next >= i64::from(max) {
        return max;
    }
    u32::try_from(next).unwrap_or(max)
}

// 月の x 座標。押せる帯・目盛りの文字は12等分の帯の中心にいるため、
// 線と丸も同じ帯中心から出す(全幅に広げると端で半月ずれる)
fn x_percent(month: u32) -> f64 {
    (f64::from(month.saturating_sub(1)) + 0.5) / 12.0 * 100.0
}

fn hover_capable() -> bool {
    web_sys::window()
        .and_then(|window| {
            window
                .match_media("(hover: hover) and (pointer: fine)")
                .ok()
                .flatten()
        })
        .is_some_and(|query| query.matches())
}

// Tailwind の max-sm と同じ境目(640px 未満)で吹き出しの寄せを切り替える
fn narrow_screen() -> bool {
    web_sys::window()
        .and_then(|window| window.inner_width().ok())
        .and_then(|width| width.as_f64())
        .is_some_and(|width| width < 640.0)
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
    let chart_ref = NodeRef::<leptos::html::Div>::new();
    let plot_ref = NodeRef::<leptos::html::Div>::new();
    let dismiss = move || {
        if selected.get_untracked().is_some() {
            selected.set(None);
        }
    };
    // 吹き出しが開いている間だけ、ページ全体に1つずつ検知を付ける。
    // 描画域の内側は押した位置の月の切替えに任せ、閉じたら外す
    Effect::new(move |_| {
        if selected.get().is_none() {
            return;
        }
        let Some(document) = web_sys::window().and_then(|window| window.document()) else {
            return;
        };
        let on_outside = Closure::wrap(Box::new(move |event: web_sys::PointerEvent| {
            let inside = chart_ref
                .get_untracked()
                .zip(
                    event
                        .target()
                        .and_then(|target| target.dyn_into::<web_sys::Node>().ok()),
                )
                .is_some_and(|(chart, target)| chart.contains(Some(&target)));
            if !inside {
                dismiss();
            }
        }) as Box<dyn FnMut(_)>);
        let _ = document.add_event_listener_with_callback_and_bool(
            "pointerdown",
            on_outside.as_ref().unchecked_ref(),
            true,
        );
        // scroll はバブルしないため document のキャプチャ段で拾う
        let on_scroll = Closure::wrap(Box::new(dismiss) as Box<dyn FnMut()>);
        let _ = document.add_event_listener_with_callback_and_bool(
            "scroll",
            on_scroll.as_ref().unchecked_ref(),
            true,
        );
        let handles = send_wrapper::SendWrapper::new((document, on_outside, on_scroll));
        on_cleanup(move || {
            let (document, on_outside, on_scroll) = &*handles;
            let _ = document.remove_event_listener_with_callback_and_bool(
                "pointerdown",
                on_outside.as_ref().unchecked_ref(),
                true,
            );
            let _ = document.remove_event_listener_with_callback_and_bool(
                "scroll",
                on_scroll.as_ref().unchecked_ref(),
                true,
            );
        });
    });
    let current_label = labels::cumulative_year_label(current_year);
    let previous_label = labels::cumulative_year_label(previous_year);
    let current_values = &current_cum[..(current_month as usize).min(current_cum.len())];
    let current_line = line_points(current_values, top);
    let current_area = area_points(current_values, top);
    let previous_line = previous_cum.as_ref().map(|sums| line_points(sums, top));
    let current_end = current_values.last().copied().unwrap_or(0.0);
    let previous_end = previous_cum
        .as_ref()
        .and_then(|sums| sums.get(current_month as usize - 1).copied());
    let current_end_text = format::format_currency(current_end);
    let previous_end_text = previous_end.map(format::format_currency);
    let previous_end_label = labels::cumulative_previous_label(current_month);
    // スライダーの読み上げ文(「6月 配当金 ¥… / 累計 ¥…」)の材料。月ごとに先に作り、
    // 選んだ月で引く。未来月は選べないため今月まで
    let slider_texts: Vec<(String, String, Option<String>)> = (1..=current_month)
        .map(|month| {
            let monthly =
                format::format_currency(current.get(month as usize - 1).copied().unwrap_or(0.0));
            let cumulative = format::format_currency(
                current_cum.get(month as usize - 1).copied().unwrap_or(0.0),
            );
            let previous = previous_cum
                .as_ref()
                .and_then(|sums| sums.get(month as usize - 1).copied())
                .map(format::format_currency);
            (monthly, cumulative, previous)
        })
        .collect();

    let months: Vec<u32> = (1..=12).collect();
    // 前年の丸は未来月にも置く(前年の線は12か月あるため)。
    // 横は帯の中心で、線の x_percent と一致する
    let previous_dot = |month: u32| {
        previous_cum
            .as_ref()
            .and_then(|sums| sums.get(month as usize - 1).copied())
            .map(|value| {
                let top_pos = y_percent(value, top);
                view! {
                    <span
                        class="home-cum-dot home-cum-dot-previous"
                        style:top=format!("{top_pos}%")
                    />
                }
            })
    };
    view! {
        <div class="home-cum-layout" node_ref=chart_ref>
            <div class="home-cum-summary-axis" aria-hidden="true"></div>
            <div class="home-cum-summary-main">
                <div class="home-cum-summary">
                    <span class="home-cum-summary-current">
                        <span class="home-cum-summary-current-dot" aria-hidden="true">{"●"}</span>{" "}{labels::CUMULATIVE_THIS_YEAR}{" "}<Amount text=current_end_text.clone() />
                    </span>
                    {previous_end_text.map(|text| view! {
                        <span class="home-cum-summary-previous">
                            <span aria-hidden="true">{"○"}</span>{" "}{previous_end_label}{" "}<Amount text=text />
                        </span>
                    })}
                </div>
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
                <div class="home-cum-main">
                <div
                    class="home-chart-plot"
                    node_ref=plot_ref
                    tabindex="0"
                    role="slider"
                    aria-label=aria_label.clone()
                    aria-valuemin="1"
                    aria-valuemax=current_month.to_string()
                    aria-valuenow=move || selected.get().unwrap_or(current_month).to_string()
                    aria-valuetext=move || {
                        let month = selected.get().unwrap_or(current_month);
                        slider_texts.get(month as usize - 1).map_or_default(
                            |(monthly, cumulative, previous)| {
                                labels::cumulative_slider_valuetext(
                                    month,
                                    monthly,
                                    cumulative,
                                    previous.as_deref(),
                                )
                            },
                        )
                    }
                    aria-orientation="horizontal"
                    on:click=move |event| {
                        let Some(plot) = plot_ref.get() else { return };
                        let rect = plot.get_bounding_client_rect();
                        let month = month_for_position(
                            f64::from(event.client_x()) - rect.left(),
                            rect.width(),
                            current_month,
                        );
                        let hover = hover_capable();
                        selected.update(|state| {
                            *state = next_selected(*state, month, hover);
                        });
                    }
                    on:pointermove=move |event| {
                        if !hover_capable() {
                            return;
                        }
                        let Some(plot) = plot_ref.get() else { return };
                        let rect = plot.get_bounding_client_rect();
                        let month = month_for_position(
                            f64::from(event.client_x()) - rect.left(),
                            rect.width(),
                            current_month,
                        );
                        selected.update(|state| {
                            if *state != Some(month) {
                                *state = Some(month);
                            }
                        });
                    }
                    on:pointerleave=move |_| {
                        if hover_capable() {
                            dismiss();
                        }
                    }
                    on:keydown=move |event| {
                        let key = event.key();
                        if key == "Escape" {
                            event.prevent_default();
                            dismiss();
                            return;
                        }
                        // div のスライダーはボタンと違い Enter/Space の既定動作が無いため、
                        // ここで吹き出しの開閉を受け持つ(主要な操作は Enter/Space で操作できること)
                        if key == "Enter" || key == " " {
                            event.prevent_default();
                            selected.update(|state| {
                                *state = if state.is_some() {
                                    None
                                } else {
                                    Some(state.unwrap_or(current_month))
                                };
                            });
                            return;
                        }
                        let base = selected.get_untracked().unwrap_or(current_month);
                        let next = match key.as_str() {
                            "ArrowLeft" | "ArrowDown" => Some(step_month(base, -1, current_month)),
                            "ArrowRight" | "ArrowUp" => {
                                Some(step_month(base, 1, current_month))
                            }
                            "Home" => Some(1),
                            "End" => Some(current_month),
                            _ => None,
                        };
                        if let Some(month) = next {
                            event.prevent_default();
                            selected.set(Some(month));
                        }
                    }
                >
                <div class="home-chart-grid" aria-hidden="true">
                    {ticks
                        .iter()
                        .rev()
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
                    <defs>
                        <linearGradient id="home-cum-area-gradient" x1="0" y1="0" x2="0" y2="1">
                            <stop offset="0%" stop-color="var(--color-chart-1)" stop-opacity="0.18" />
                            <stop offset="100%" stop-color="var(--color-chart-1)" stop-opacity="0" />
                        </linearGradient>
                    </defs>
                    <polygon
                        class="home-cum-area-current"
                        fill="url(#home-cum-area-gradient)"
                        points=current_area
                    />
                    {previous_line
                        .map(|points| view! {
                            <polyline class="home-cum-line-previous" points=points />
                        })}
                    <polyline class="home-cum-line-current" points=current_line />
                </svg>
                <div class="home-cum-cols" aria-hidden="true">
                    {months
                    .iter()
                    .map(|month| {
                        let month = *month;
                        if month > current_month {
                            // 未来の月も同じ幅の枠だけ置く。線の12か月座標と
                            // 点と月ラベルをずらさないため
                            return view! {
                                <div class="home-cum-col">{previous_dot(month)}</div>
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
                        let previous_top = previous_value.map(|value| y_percent(value, top));
                        let current_text = format::format_currency(current_value);
                        let previous_text = previous_value.map(format::format_currency);
                        let month_label = labels::receipt_month_label(month);
                        let current_cell = current_label.clone();
                        let previous_cell = previous_label.clone();
                        let dot_top = y_percent(current_value, top);
                        // 吹き出しは点の近くに出す。上の丸の少し上を基本にし、
                        // 上に入らないときは下の丸の少し下に出す。
                        // 端の月は吹き出しを内側に寄せる
                        let edge = tip_edge(month, narrow_screen());
                        let tip_below = tip_below(dot_top, previous_top);
                        // 上の丸も下の丸も隠さないよう、上下それぞれの丸を基準にする
                        let tip_anchor = previous_top.map_or(dot_top, |previous| {
                            if tip_below {
                                dot_top.max(previous)
                            } else {
                                dot_top.min(previous)
                            }
                        });
                        let tip_top =
                            tip_below.then(|| format!("calc({tip_anchor:.2}% + 12px)"));
                        let tip_bottom = if tip_below {
                            "auto".to_string()
                        } else {
                            format!("calc({:.2}% + 8px)", 100.0 - tip_anchor)
                        };
                        let is_current = month == current_month;
                        view! {
                            <div
                                class="home-cum-col"
                                data-selected=move || (selected.get() == Some(month)).to_string()
                                data-current=is_current.to_string()
                                data-edge=edge
                            >
                                {previous_dot(month)}
                                <span
                                    class="home-cum-dot home-cum-dot-current"
                                    style:top=format!("{dot_top}%")
                                />
                                <div
                                    class="home-cum-tip"
                                    aria-hidden="true"
                                    style:top=tip_top
                                    style:bottom=tip_bottom
                                >
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
                <div class="home-chart-month-labels" aria-hidden="true">
                    {months
                        .iter()
                        .map(|month| {
                            let is_current = *month == current_month;
                            view! {
                                <span class="home-chart-month-label" data-current=is_current.to_string()>
                                    {month.to_string()}<span
                                        class="home-chart-month-unit"
                                        aria-hidden="true"
                                    >
                                        {labels::MONTH}
                                    </span>
                                </span>
                            }
                        })
                        .collect_view()}
                </div>
                </div>
            </div>
        </div>
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
