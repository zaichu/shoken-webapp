use crate::ui::button::{Button, ButtonVariant};
use leptos::prelude::*;

/// 3択程度の少数選択を横一列の押しボタン群で選ぶ。選択中は aria-pressed で示す
#[component]
pub fn Segmented(
    /// (値, ラベル) の選択肢
    options: &'static [(&'static str, &'static str)],
    /// 現在の値
    #[prop(into)]
    value: Signal<String>,
    /// 変更時に新しい値を渡す
    on_change: impl Fn(String) + Clone + 'static,
    #[prop(optional)] aria_label: &'static str,
    #[prop(optional)] testid: Option<&'static str>,
) -> impl IntoView {
    view! {
        <div class="segmented" role="group" aria-label=aria_label data-testid=testid>
            {options
                .iter()
                .map(|&(option_value, label)| {
                    let on_change = on_change.clone();
                    view! {
                        <Button
                            variant=ButtonVariant::Segmented
                            aria_pressed=Signal::derive(move || {
                                if value.get() == option_value {
                                    "true".to_string()
                                } else {
                                    "false".to_string()
                                }
                            })
                            on_click=move |_| on_change(option_value.to_string())
                        >
                            {label}
                        </Button>
                    }
                })
                .collect_view()}
        </div>
    }
}
