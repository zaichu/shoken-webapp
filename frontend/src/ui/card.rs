use leptos::prelude::*;

/// カード状の面。variant が見た目(枠・角丸・影・背景)を決め、
/// 余白や並びなどのレイアウトは class で追加する。

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CardVariant {
    /// .panel-card(フォーム・一覧の外枠)
    Panel,
    /// .login-card
    Login,
    /// .table-card(明細表の外枠)
    Table,
    /// .collapsible-card(開閉つきの帯。section 要素で出す)
    Collapsible,
    /// .feature-card(ホームのリンクカード。a 要素で出す)
    Feature,
    /// .rail-panel(ユーティリティレールの外枠)
    Rail,
    /// 白く浮いた外枠(空の状態の包み)
    Shell,
    /// 資産サマリーの外枠
    Summary,
    /// 薄い外枠(資産管理の空状態)
    Soft,
    /// 明細カード(スマホの行カード)
    Item,
    /// グループの外枠(月ごとの明細)
    Group,
    /// 保有カード(PC 一覧)
    Holding,
    /// KPI のマス
    Stat,
    /// 取引明細 KPI の小さいマス(色は class で付ける)
    StatSmall,
    /// 内側のくぼんだ箱(配当情報の小枠)
    Sunken,
    /// カード内の集計ストリップ(grid 等は class で)
    Strip,
    /// 検索ヒントの薄い箱
    Hint,
    /// 破線の箱(ホームの CSV 案内)
    Dashed,
    /// 破線の小さい箱(保有内訳の「その他」)
    DashedCompact,
    /// ホームの手順ステップ
    Step,
    /// ホームの概要タイル
    Tile,
    /// グループのラベル行(開閉なし)
    GroupLabel,
}

impl CardVariant {
    fn class(self) -> &'static str {
        match self {
            Self::Panel => "panel-card",
            Self::Login => "login-card",
            Self::Table => "table-card",
            Self::Collapsible => "collapsible-card",
            Self::Feature => "feature-card",
            Self::Rail => "rail-panel",
            Self::Shell => {
                "overflow-hidden rounded-xl border border-ink/10 bg-surface/95 shadow-elevation-2"
            }
            Self::Summary => {
                "rounded-xl border border-ink/10 bg-surface/95 px-5 py-5 shadow-summary-card"
            }
            Self::Soft => "overflow-hidden rounded-xl border border-ink/10 bg-surface/90 shadow-card",
            Self::Item => "rounded-lg border border-border-strong bg-surface",
            Self::Group => "overflow-hidden rounded-lg border border-border-subtle",
            Self::Holding => "rounded-lg border border-ink/10 bg-surface shadow-sm",
            Self::Stat => "rounded-lg border border-ink/10 bg-surface px-4 py-4 shadow-sm",
            Self::StatSmall => "rounded-lg border px-3.5 py-3",
            Self::Sunken => "rounded-lg bg-surface-sunken px-4 py-3",
            Self::Strip => "overflow-hidden rounded-md bg-surface-sunken",
            Self::Hint => "rounded-xl border border-ink/10 bg-surface-sunken/80 p-4",
            Self::Dashed => {
                "rounded-xl border border-dashed border-border-strong bg-surface-raised/70 px-4 py-4"
            }
            Self::DashedCompact => {
                "rounded-lg border border-dashed border-border-strong bg-surface-sunken p-3"
            }
            Self::Step => "rounded-lg border border-ink/10 bg-surface/75 px-3 py-3",
            Self::Tile => "rounded-xl border border-ink/10 bg-surface px-4 py-4 shadow-sm",
            Self::GroupLabel => {
                "flex min-h-11 items-center rounded-lg bg-surface-raised px-3 py-2"
            }
        }
    }

    fn section_tag(self) -> bool {
        matches!(self, Self::Collapsible | Self::Summary)
    }
}

#[component]
pub fn Card(
    variant: CardVariant,
    #[prop(into, optional)] class: Signal<String>,
    #[prop(optional)] href: Option<String>,
    #[prop(optional)] testid: Option<&'static str>,
    #[prop(optional)] id: Option<String>,
    #[prop(optional)] aria_label: Option<&'static str>,
    children: ChildrenFn,
) -> impl IntoView {
    let classes = move || {
        let extra = class.get();
        if extra.is_empty() {
            variant.class().to_string()
        } else {
            format!("{} {extra}", variant.class())
        }
    };
    if let Some(href) = href {
        return view! {
            <a href=href class=classes data-testid=testid>
                {children()}
            </a>
        }
        .into_any();
    }
    if variant.section_tag() {
        return view! {
            <section class=classes data-testid=testid id=id aria-label=aria_label>
                {children()}
            </section>
        }
        .into_any();
    }
    view! {
        <div class=classes data-testid=testid id=id aria-label=aria_label>
            {children()}
        </div>
    }
    .into_any()
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SectionHeaderVariant {
    /// 帯の中の見出し(保有内訳)。h3
    Band,
    /// カード内の上段見出し(資産サマリー)。h2
    Card,
    /// 下線だけの見出し(集計情報)。h2
    Divider,
}

impl SectionHeaderVariant {
    fn frame(self) -> &'static str {
        match self {
            Self::Band => "summary-section-header",
            Self::Card => {
                "flex flex-col gap-3 border-b border-ink/10 pb-4 sm:flex-row sm:items-end sm:justify-between"
            }
            Self::Divider => "flex items-start justify-between gap-3 border-b border-ink/10 pb-2.5",
        }
    }

    fn heading(self) -> &'static str {
        match self {
            Self::Band => "text-sm font-black text-text-strong",
            Self::Card | Self::Divider => "text-sm font-black text-ink",
        }
    }
}

/// 節見出し。trailing に右側の操作(バッジ・解除ボタンなど)を置く
#[component]
pub fn SectionHeader(
    variant: SectionHeaderVariant,
    #[prop(optional)] testid: Option<&'static str>,
    #[prop(optional)] trailing: Option<AnyView>,
    children: ChildrenFn,
) -> impl IntoView {
    let frame = variant.frame();
    let heading = variant.heading();
    view! {
        <div class=frame data-testid=testid>
            <div>
                {if variant == SectionHeaderVariant::Band {
                    view! { <h3 class=heading>{children()}</h3> }.into_any()
                } else {
                    view! { <h2 class=heading>{children()}</h2> }.into_any()
                }}
            </div>
            {trailing}
        </div>
    }
}
