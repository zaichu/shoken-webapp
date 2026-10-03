use leptos::prelude::*;

/// 小さな状態バッジ(span)。押せるバッジは choice::Chip を使う。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BadgeVariant {
    /// 絞り込み適用中
    Accent,
    /// プレビュー中の表示
    AccentFlat,
    /// 情報(絞り込み中: N/M件)
    Info,
    /// 保有銘柄の緑
    Positive,
    /// ファイル選択の参照チップ
    File,
}

impl BadgeVariant {
    fn class(self) -> &'static str {
        match self {
            Self::Accent => "filter-badge",
            Self::AccentFlat => {
                "shrink-0 rounded bg-accent-softer px-1.5 py-0.5 text-xs font-bold text-accent-text"
            }
            Self::Info => {
                "inline-flex items-center rounded-md border border-info-border bg-info-soft px-3 py-1 text-sm font-bold text-info"
            }
            Self::Positive => {
                "inline-flex items-center rounded bg-positive-softer px-1.5 py-0.5 text-xs font-medium text-positive"
            }
            Self::File => "file-chip",
        }
    }
}

#[component]
pub fn Badge(variant: BadgeVariant, children: Children) -> impl IntoView {
    view! { <span class=variant.class()>{children()}</span> }
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
