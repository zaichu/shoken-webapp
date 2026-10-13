use leptos::prelude::*;

/// 小さな状態バッジ(span)。押せるバッジは choice::Chip を使う。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BadgeVariant {
    /// 絞り込み適用中
    Accent,
    /// プレビュー中の表示
    AccentFlat,
    /// 保有銘柄の緑
    Positive,
}

impl BadgeVariant {
    fn class(self) -> &'static str {
        match self {
            Self::Accent => "filter-badge",
            Self::AccentFlat => {
                "shrink-0 rounded bg-accent-softer px-1.5 py-0.5 text-xs font-bold text-accent-text"
            }
            Self::Positive => {
                "inline-flex items-center rounded bg-positive-softer px-1.5 py-0.5 text-xs font-medium text-positive"
            }
        }
    }
}

#[component]
pub fn Badge(variant: BadgeVariant, children: Children) -> impl IntoView {
    view! { <span class=variant.class()>{children()}</span> }
}

/// 口座区分のバッジ。文字に NISA を含むものだけ琥珀配色にする
#[component]
pub fn AccountBadge(#[prop(into)] label: String) -> impl IntoView {
    let class = if is_nisa_account(&label) {
        "account-badge account-badge-nisa"
    } else {
        "account-badge"
    };
    view! { <span class=class>{label}</span> }
}

fn is_nisa_account(label: &str) -> bool {
    label.contains("NISA")
}

#[component]
pub fn CodeBadge(
    #[prop(optional)] flush: bool,
    #[prop(into, optional)] class: Signal<String>,
    #[prop(optional)] testid: Option<&'static str>,
    children: Children,
) -> impl IntoView {
    let base = if flush {
        "code-badge-flush"
    } else {
        "code-badge"
    };
    view! {
        <span
            class=move || {
                let extra = class.get();
                if extra.is_empty() {
                    base.to_string()
                } else {
                    format!("{base} {extra}")
                }
            }
            data-testid=testid
        >
            {children()}
        </span>
    }
}

#[cfg(test)]
mod tests;
