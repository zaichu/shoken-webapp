use crate::ui::amount::Amount;
use crate::ui::state::Skeleton;
use leptos::prelude::*;

/// 圧縮集計欄の1項目。value が None のときは取得中としてスケルトンを出す
#[derive(Clone)]
pub struct SummaryItem {
    pub label: &'static str,
    pub value: Option<String>,
    /// 値の右に添える補足(損益率など)
    pub note: Option<String>,
    /// マイナス値。符号と色の両方で示す
    pub negative: bool,
}

impl SummaryItem {
    pub fn new(label: &'static str, value: Option<String>) -> Self {
        Self {
            label,
            value,
            note: None,
            negative: false,
        }
    }
}

/// ページ上部に置く圧縮した集計。ラベルと値を段組みの1行で並べる
#[component]
pub fn SummaryStrip(#[prop(into)] items: Signal<Vec<SummaryItem>>) -> impl IntoView {
    view! {
        <dl class="summary-strip">
            {move || {
                items
                    .get()
                    .into_iter()
                    .map(|item| {
                        view! {
                            <div class="summary-strip-item">
                                <dt class="text-xs font-medium text-text-muted">{item.label}</dt>
                                <dd
                                    class="mt-0.5 flex flex-wrap items-baseline gap-x-2 text-base font-bold text-ink"
                                    data-negative=item.negative.then_some("true")
                                >
                                    {match item.value {
                                        Some(value) => {
                                            view! {
                                                <Amount class="break-all" text=value negative=item.negative />
                                                {item.note.map(|note| {
                                                    view! {
                                                        <span class="whitespace-nowrap text-sm font-bold">
                                                            {format!("（{note}）")}
                                                        </span>
                                                    }
                                                })}
                                            }
                                                .into_any()
                                        }
                                        None => view! { <Skeleton class="h-6 w-24" /> }.into_any(),
                                    }}
                                </dd>
                            </div>
                        }
                    })
                    .collect_view()
            }}
        </dl>
    }
}
