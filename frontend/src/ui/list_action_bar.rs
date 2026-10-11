use crate::ui::button::{Button, ButtonSize, ButtonVariant, LinkButton};
use leptos::prelude::*;
use wasm_bindgen::JsCast;

/// 適用中の検索条件。文字ボタンを押すとその条件だけ解除される
#[derive(Clone)]
pub struct AppliedCondition {
    pub label: String,
    pub on_remove: Callback<()>,
}

/// 一覧ページの下部操作帯。検索入力・消去・条件の入口と適用中条件の解除を1か所にまとめる
#[component]
pub fn ListActionBar(
    /// 文字絞り込みの入力値(変更は即時に反映される)
    query: RwSignal<String>,
    /// 入力欄の placeholder と aria-label
    #[prop(into)]
    placeholder: String,
    /// 「N / M件」など検索結果の件数表示。空文字なら出さない
    #[prop(into, optional)]
    count_text: Option<Signal<String>>,
    /// 条件ページへの入口([検索条件])
    #[prop(optional)]
    conditions_href: Option<&'static str>,
    /// 適用中の条件(文字ボタンで個別解除)
    #[prop(optional)]
    conditions: Option<Signal<Vec<AppliedCondition>>>,
    /// 全条件をまとめて解除する操作。条件が1つでもあれば「全解除」を出す
    #[prop(optional)]
    on_clear_all: Option<Callback<()>>,
    #[prop(optional)] input_testid: Option<&'static str>,
    #[prop(optional)] clear_testid: Option<&'static str>,
    #[prop(optional)] count_testid: Option<&'static str>,
) -> impl IntoView {
    let label = placeholder.clone();
    view! {
        <div class="list-action-bar" role="search">
            {conditions
                .map(|conditions| {
                    view! {
                        <div class="list-action-bar-conditions">
                            {move || {
                                let applied = conditions.get();
                                (!applied.is_empty()).then(|| {
                                    let items = applied
                                        .iter()
                                        .map(|condition| {
                                            let text = condition.label.clone();
                                            let aria = format!("{}を解除", condition.label);
                                            let on_remove = condition.on_remove;
                                            view! {
                                                <Button
                                                    variant=ButtonVariant::SecondarySoft(ButtonSize::Xs)
                                                    aria_label=aria
                                                    on_click=move |_| on_remove.run(())
                                                >
                                                    {text}
                                                </Button>
                                            }
                                        })
                                        .collect_view();
                                    let clear_all = on_clear_all.map(|on_clear_all| {
                                        view! {
                                            <Button
                                                variant=ButtonVariant::Ghost
                                                on_click=move |_| on_clear_all.run(())
                                            >
                                                "全解除"
                                            </Button>
                                        }
                                    });
                                    view! { {items} {clear_all} }
                                })
                            }}
                        </div>
                    }
                })}
            <div class="flex items-stretch gap-2">
                <div class="search-input-frame flex-1">
                    <input
                        type="search"
                        class="asset-search-input"
                        placeholder=placeholder
                        aria-label=label
                        autocomplete="off"
                        enterkeyhint="search"
                        data-testid=input_testid
                        prop:value=move || query.get()
                        // IME 変換中の確定前の文字では絞り込まない。確定は compositionend で受ける
                        on:input=move |ev| {
                            if ev.unchecked_ref::<web_sys::InputEvent>().is_composing() {
                                return;
                            }
                            query.set(event_target_value(&ev));
                        }
                        on:compositionend=move |ev| query.set(event_target_value(&ev))
                    />
                    {move || {
                        (!query.get().is_empty()).then(|| {
                            view! {
                                <Button
                                    variant=ButtonVariant::SecondarySoft(ButtonSize::Xs)
                                    class="shrink-0 self-center".to_string()
                                    aria_label="検索を解除".to_string()
                                    testid=clear_testid.unwrap_or_default()
                                    on_click=move |_| query.set(String::new())
                                >
                                    "消去"
                                </Button>
                            }
                        })
                    }}
                </div>
                {conditions_href.map(|href| {
                    view! {
                        <LinkButton
                            variant=ButtonVariant::Secondary(ButtonSize::Md)
                            href=href
                        >
                            "検索条件"
                        </LinkButton>
                    }
                })}
            </div>
            {count_text.map(|count_text| {
                view! {
                    {move || {
                        let text = count_text.get();
                        (!text.is_empty()).then(|| {
                            view! {
                                <p class="asset-search-count" data-testid=count_testid>
                                    {text}
                                </p>
                            }
                        })
                    }}
                }
            })}
        </div>
    }
}
