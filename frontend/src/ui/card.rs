use leptos::prelude::*;

/// カード状の面。variant が見た目(枠・角丸・影・背景)を決め、
/// 余白や並びなどのレイアウトは class で追加する。

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CardVariant {
    /// フォーム・一覧の外枠
    Panel,
    /// ログイン画面のカード
    Login,
    /// 明細表の外枠
    Table,
    /// 開閉つきの帯(section 要素で出す)
    Collapsible,
    /// ホームのリンクカード(a 要素で出す)
    Feature,
    /// 白く浮いた外枠(空の状態の包み)
    Shell,
    /// 薄い外枠(資産管理の空状態)
    Soft,
    /// 明細カード(スマホの行カード)
    Item,
    /// グループの外枠(月ごとの明細)
    Group,
    /// 保有カード(PC 一覧)
    Holding,
    /// 内側のくぼんだ箱(配当情報の小枠)
    Sunken,
    /// カード内の集計ストリップ(grid 等は class で)
    Strip,
    /// 破線の小さい箱(保有内訳の「その他」)
    DashedCompact,
    /// ホームの概要タイル
    Tile,
    /// グループのラベル行(開閉なし)
    GroupLabel,
}

impl CardVariant {
    fn class(self) -> String {
        let class = match self {
            Self::Panel => "panel-card",
            Self::Login => "login-card",
            Self::Table => "table-card",
            Self::Collapsible => "collapsible-card",
            Self::Feature => "feature-card",
            Self::Shell => {
                "overflow-hidden rounded-xl border border-ink/10 bg-surface/95 shadow-elevation-2"
            }
            Self::Soft => {
                "overflow-hidden rounded-xl border border-ink/10 bg-surface/90 shadow-card"
            }
            Self::Item => "rounded-lg border border-border-strong bg-surface",
            Self::Group => "overflow-hidden rounded-lg border border-border-subtle",
            Self::Holding => "rounded-lg border border-ink/10 bg-surface shadow-sm",
            Self::Sunken => "rounded-lg bg-surface-sunken px-4 py-3",
            Self::Strip => "overflow-hidden rounded-md bg-surface-sunken",
            Self::DashedCompact => {
                "rounded-lg border border-dashed border-border-strong bg-surface-sunken p-3"
            }
            Self::Tile => "rounded-xl border border-ink/10 bg-surface px-4 py-4 shadow-sm",
            Self::GroupLabel => "flex min-h-11 items-center rounded-lg bg-surface-raised px-3 py-2",
        };
        class.to_string()
    }

    fn section_tag(self) -> bool {
        matches!(self, Self::Collapsible)
    }
}

/// KPI の値の色分け(マイナスだけ赤文字)
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StatTone {
    /// 中立
    Neutral,
    /// マイナス
    Loss,
}

#[component]
pub fn Card(
    variant: CardVariant,
    #[prop(into, optional)] class: Signal<String>,
    #[prop(optional)] href: Option<String>,
    #[prop(optional)] testid: Option<&'static str>,
    #[prop(optional)] id: Option<String>,
    #[prop(optional)] aria_label: Option<&'static str>,
    #[prop(optional)] role: Option<&'static str>,
    children: Children,
) -> impl IntoView {
    let classes = move || {
        let extra = class.get();
        if extra.is_empty() {
            variant.class()
        } else {
            format!("{} {extra}", variant.class())
        }
    };
    if let Some(href) = href {
        return view! {
            <a href=href class=classes data-testid=testid id=id aria-label=aria_label role=role>
                {children()}
            </a>
        }
        .into_any();
    }
    if variant.section_tag() {
        return view! {
            <section class=classes data-testid=testid id=id aria-label=aria_label role=role>
                {children()}
            </section>
        }
        .into_any();
    }
    view! {
        <div class=classes data-testid=testid id=id aria-label=aria_label role=role>
            {children()}
        </div>
    }
    .into_any()
}

#[cfg(test)]
mod tests;
