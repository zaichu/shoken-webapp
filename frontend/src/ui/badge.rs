use leptos::prelude::*;

/// 小さな状態バッジ(span)。押せるバッジは choice::Chip を使う。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BadgeVariant {
    /// .filter-badge(絞り込み適用中)
    Accent,
    /// プレビュー中の表示
    AccentFlat,
    /// 情報(絞り込み中: N/M件)
    Info,
    /// 保有銘柄の緑
    Positive,
    /// CSV 保存結果の丸いバッジ
    Neutral,
    /// 保存結果の補助バッジ
    Muted,
    /// 保存結果の注意バッジ
    Warn,
    /// 参照チップ(ファイル選択)
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
            Self::Neutral => {
                "inline-flex items-center rounded-full border border-border-subtle bg-surface px-2.5 py-1 text-xs font-semibold text-text-soft"
            }
            Self::Muted => {
                "inline-flex items-center rounded-full border border-border-subtle bg-surface px-2.5 py-1 text-xs font-medium text-text-muted"
            }
            Self::Warn => {
                "inline-flex items-center rounded-full border border-accent-border bg-surface px-2.5 py-1 text-xs font-medium text-accent-deep"
            }
            Self::File => "file-chip",
        }
    }
}

#[component]
pub fn Badge(
    variant: BadgeVariant,
    #[prop(into, optional)] class: Signal<String>,
    #[prop(optional)] testid: Option<&'static str>,
    children: Children,
) -> impl IntoView {
    view! {
        <span
            class=move || {
                let extra = class.get();
                if extra.is_empty() {
                    variant.class().to_string()
                } else {
                    format!("{} {extra}", variant.class())
                }
            }
            data-testid=testid
        >
            {children()}
        </span>
    }
}

/// 銘柄コードのバッジ(.code-badge)
#[component]
pub fn CodeBadge(
    #[prop(into, optional)] class: Signal<String>,
    #[prop(optional)] testid: Option<&'static str>,
    children: Children,
) -> impl IntoView {
    view! {
        <span
            class=move || {
                let extra = class.get();
                if extra.is_empty() {
                    "code-badge".to_string()
                } else {
                    format!("code-badge {extra}")
                }
            }
            data-testid=testid
        >
            {children()}
        </span>
    }
}
