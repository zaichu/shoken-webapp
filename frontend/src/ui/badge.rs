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

/// 口座区分のバッジ。文字に NISA を含むものだけ琥珀配色にする
#[component]
pub fn AccountBadge(#[prop(into)] label: String) -> impl IntoView {
    let label = account_display_name(&label).to_string();
    let class = if is_nisa_account(&label) {
        "account-badge account-badge-nisa"
    } else {
        "account-badge"
    };
    view! { <span class=class>{label}</span> }
}

/// CSV の口座を1口座ずつに分ける。証券会社の CSV は複数口座を「特定・一般」のように
/// 中黒で連結するため、札を並べるときに分割する
pub fn account_labels(value: &str) -> Vec<&str> {
    value
        .split('・')
        .map(str::trim)
        .filter(|label| !label.is_empty())
        .collect()
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
