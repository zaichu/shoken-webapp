use crate::csv_input;
use crate::ui::labels;
use leptos::prelude::*;

/// 小さな状態バッジ(span)。押せるバッジは choice::Chip を使う。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BadgeVariant {
    /// 保有銘柄の緑
    Positive,
    /// 情報の青
    Info,
    /// 注意の青紫
    Accent,
    /// 失敗・危険の赤
    Danger,
    /// 口座の特定
    AccountSpecific,
    /// 口座の一般・対応表にない中立
    AccountGeneral,
    /// 口座のNISA系
    AccountNisa,
    /// 口座の旧NISA
    AccountOldNisa,
    /// 銘柄コードの札
    Code,
    /// 背景なしの銘柄コード
    CodeFlush,
    /// タブ横の件数
    Count,
}

impl BadgeVariant {
    fn class(self) -> &'static str {
        match self {
            Self::Positive => "badge badge-positive",
            Self::Info => "badge badge-info",
            Self::Accent => "badge badge-accent",
            Self::Danger => "badge badge-danger",
            Self::AccountSpecific => "badge badge-account badge-account-specific",
            Self::AccountGeneral => "badge badge-account",
            Self::AccountNisa => "badge badge-account badge-account-nisa",
            Self::AccountOldNisa => "badge badge-account badge-account-old-nisa",
            Self::Code => "badge badge-code",
            Self::CodeFlush => "badge badge-code-flush",
            Self::Count => "badge badge-count",
        }
    }
}

#[component]
pub fn Badge(
    variant: BadgeVariant,
    #[prop(into, optional)] class: Signal<String>,
    #[prop(into, optional)] testid: String,
    children: Children,
) -> impl IntoView {
    let testid_attr = testid.clone();
    view! {
        <span
            class=move || {
                let extra = class.get();
                let base = variant.class();
                if extra.is_empty() {
                    base.to_string()
                } else {
                    format!("{base} {extra}")
                }
            }
            data-testid=move || (!testid_attr.is_empty()).then(|| testid_attr.clone())
        >
            {children()}
        </span>
    }
}

/// 口座区分の表示名。証券会社 CSV 由来の長い名称を短い札向けの表記に直す。
/// 対応表にない文字列はそのまま返す
pub fn account_display_name(value: &str) -> &str {
    match value {
        csv_input::ACCOUNT_SPECIFIC | labels::ACCOUNT_SPECIFIC => labels::ACCOUNT_SPECIFIC,
        csv_input::ACCOUNT_GENERAL | labels::ACCOUNT_GENERAL => labels::ACCOUNT_GENERAL,
        csv_input::ACCOUNT_NISA | csv_input::ACCOUNT_NEW_NISA | csv_input::ACCOUNT_NISA_SHORT => {
            csv_input::ACCOUNT_NISA_SHORT
        }
        labels::ACCOUNT_OLD_NISA => labels::ACCOUNT_OLD_NISA,
        csv_input::ACCOUNT_NISA_GROWTH => labels::ACCOUNT_NISA_GROWTH,
        csv_input::ACCOUNT_NISA_TSUMITATE => labels::ACCOUNT_NISA_TSUMITATE,
        _ => value,
    }
}

/// 口座区分のバッジ。配色は input.css の --color-badge-* トークンで決まる。
/// 対応表にない口座名は一般と同じ中立色にする
#[component]
pub fn AccountBadge(#[prop(into)] label: String) -> impl IntoView {
    let label = account_display_name(&label).to_string();
    let variant = account_badge_variant(&label);
    view! { <Badge variant=variant>{label}</Badge> }
}

/// 口座名の配色。連結した口座名(「特定・一般」)は単独の口座名ではないので
/// 一般と同じ中立色になる
fn account_badge_variant(label: &str) -> BadgeVariant {
    match label {
        labels::ACCOUNT_SPECIFIC => BadgeVariant::AccountSpecific,
        labels::ACCOUNT_OLD_NISA => BadgeVariant::AccountOldNisa,
        _ if label.contains("NISA") => BadgeVariant::AccountNisa,
        _ => BadgeVariant::AccountGeneral,
    }
}

#[component]
pub fn CodeBadge(
    #[prop(optional)] flush: bool,
    #[prop(into, optional)] class: Signal<String>,
    #[prop(optional)] testid: Option<&'static str>,
    children: Children,
) -> impl IntoView {
    let variant = if flush {
        BadgeVariant::CodeFlush
    } else {
        BadgeVariant::Code
    };
    view! {
        <Badge variant=variant class=class testid=testid.unwrap_or_default()>
            {children()}
        </Badge>
    }
}

#[cfg(test)]
mod tests;
