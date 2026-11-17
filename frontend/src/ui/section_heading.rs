use leptos::prelude::*;

/// 設定画面の節見出し。見た目はこの部品だけで決め、呼び出し側で変えない
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum SectionHeadingTone {
    #[default]
    Default,
    Danger,
}

#[component]
pub fn SectionHeading(
    id: &'static str,
    #[prop(optional)] tone: SectionHeadingTone,
    children: Children,
) -> impl IntoView {
    let class = match tone {
        SectionHeadingTone::Danger => "text-heading font-bold text-negative mb-4",
        SectionHeadingTone::Default => "text-heading font-bold mb-4",
    };
    view! {
        <h2 id=id class=class>
            {children()}
        </h2>
    }
}
