use leptos::prelude::*;

/// 小さな状態バッジ(span)。押せるバッジは choice::Chip を使う。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BadgeVariant {
    /// 保有銘柄の緑
    Positive,
}

impl BadgeVariant {
    fn class(self) -> &'static str {
        match self {
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

/// 口座区分の表示名。証券会社 CSV 由来の長い名称を短い札向けの表記に直す。
/// 対応表にない文字列はそのまま返す
pub fn account_display_name(value: &str) -> &str {
    match value {
        "特定口座" | "特定" => "特定",
        "一般口座" | "一般" => "一般",
        "NISA口座" | "新NISA" | "NISA" => "NISA",
        "旧NISA" => "旧NISA",
        "NISA成長投資枠" => "NISA成長",
        "NISAつみたて投資枠" => "NISAつみたて",
        _ => value,
    }
}

/// 口座区分のバッジ。配色は input.css の --color-badge-* トークンで決まる。
/// 対応表にない口座名は一般と同じ中立色にする
#[component]
pub fn AccountBadge(#[prop(into)] label: String) -> impl IntoView {
    let label = account_display_name(&label).to_string();
    let class = account_badge_class(&label);
    view! { <span class=class>{label}</span> }
}

/// 口座名ごとの配色。連結された口座名(「特定・一般」)は単独の口座ではないので
/// 一般と同じ中立色にする
fn account_badge_class(label: &str) -> &'static str {
    match label {
        "特定" => "account-badge account-badge-specific",
        "旧NISA" => "account-badge account-badge-old-nisa",
        _ if label.contains("NISA") => "account-badge account-badge-nisa",
        _ => "account-badge",
    }
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
