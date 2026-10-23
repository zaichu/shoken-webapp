use leptos::ev;
use leptos::prelude::*;

/// features/ では <button> を直書きしない。種類は enum の props で受け、
/// 誤った組み合わせを型で防ぐ。見た目の変更は各 variant のクラスで行う。
/// ボタンは主(Primary=塗り)・副(Secondary=枠線)・無効(disabled prop=薄い・押せない)の3種。

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ButtonVariant {
    /// 紺塗りの主ボタン
    Primary(ButtonSize),
    /// 枠線の副ボタン
    Secondary(ButtonSize),
    /// 紺のヘッダー上の枠線ボタン(PC・スマホの上部帯)
    Header(ButtonSize),
    /// 赤塗りの危険確定ボタン(確認モーダル)
    DangerSolid(ButtonSize),
    /// 文字だけの静かな操作(絞り込みの解除など)
    Ghost,
    /// 読み込みエラーの再読み込み(独自ベース)
    Retry,
    /// メニュー項目(role="menuitem" と一緒に使う)
    MenuItem,
    /// 銘柄名のコピー(文字そのものがボタン)
    CopyName,
    /// 資産管理テーブルの並び替え列見出し
    AssetSortColumn,
    /// 資産管理スマホ下部帯の並べ替え順トグル(「高い順 ↓」)
    AssetMobileOrder,
    /// 検索条件の適用中チップ(「口座: 特定 ✕」)
    ConditionChip,
    /// 「条件を足す」ボタン(＋口座 など)
    AddCondition,
    /// 条件小窓の選択肢(aria-pressed で選択中を示す)
    ConditionOption,
    /// 検索条件の一括解除(「すべてクリア」)
    SearchClear,
    /// 設定画面の少数択一(金額単位・自動ログアウト)。aria-pressed で選択中を示す
    Segmented,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ButtonSize {
    /// 行内の小さな操作(検索条件の解除など)
    Xs,
    /// 小さめの操作
    Sm,
    /// 標準
    Md,
}

const BASE: &str = "inline-flex items-center justify-center rounded-md font-bold transition-[background-color,border-color,color,box-shadow,transform] focus:outline-none focus:ring-2 focus:ring-accent-bright/50 focus:ring-offset-2 disabled:cursor-not-allowed disabled:opacity-60";

fn sized(variant: &'static str, size: ButtonSize) -> String {
    format!("{BASE} {variant} {}", size.class())
}

impl ButtonVariant {
    fn classes(self) -> String {
        match self {
            Self::Primary(size) => sized(
                "border border-brand bg-brand text-text-inverse shadow-edge-lit hover:bg-brand-deep active:bg-brand-deep",
                size,
            ),
            Self::Header(size) => sized(
                "border border-brand-border bg-transparent text-text-inverse hover:bg-brand-deep",
                size,
            ),
            Self::Secondary(size) => sized(
                "border border-border-strong bg-surface text-text hover:border-border-bold hover:bg-surface-sunken",
                size,
            ),
            Self::DangerSolid(size) => sized(
                // ink・negative 塗り面はダークで明転するため、上の文字は page 色にする(text-inverse は白固定)
                "border border-negative bg-negative text-page shadow-sm hover:border-negative-strong hover:bg-negative-strong",
                size,
            ),
            Self::Ghost => "text-sm text-text-deep hover:underline".to_string(),
            Self::Retry => {
                "inline-flex min-h-11 items-center justify-center rounded-md border border-ink bg-ink px-4 text-sm font-bold text-page transition-colors hover:bg-ink-hover focus:outline-none focus-visible:ring-2 focus-visible:ring-accent-bright/50 focus-visible:ring-offset-2".to_string()
            }
            Self::MenuItem => {
                "w-full px-3 py-2 text-left text-sm font-semibold hover:bg-surface-raised"
                    .to_string()
            }
            Self::CopyName => "copyable-name".to_string(),
            Self::AssetSortColumn => "asset-th-sort".to_string(),
            Self::AssetMobileOrder => "asset-mobile-order".to_string(),
            Self::ConditionChip => "active-condition-chip".to_string(),
            Self::AddCondition => "add-condition-button".to_string(),
            Self::ConditionOption => "condition-option".to_string(),
            Self::SearchClear => "receipts-search-clear".to_string(),
            Self::Segmented => "segmented-option".to_string(),
        }
    }
}

impl ButtonSize {
    fn class(self) -> &'static str {
        match self {
            Self::Xs => "px-2 py-1 text-xs max-sm:min-h-11",
            Self::Sm => "px-3 py-1.5 text-sm max-sm:min-h-11",
            Self::Md => "px-4 py-2 text-sm max-sm:min-h-11",
        }
    }
}

fn button_classes(variant: ButtonVariant, extra: &str) -> String {
    let mut classes = variant.classes();
    if !extra.is_empty() {
        classes.push(' ');
        classes.push_str(extra);
    }
    classes
}

#[component]
pub fn Button(
    variant: ButtonVariant,
    #[prop(into, optional)] class: Signal<String>,
    #[prop(into, optional)] aria_label: Option<Signal<String>>,
    #[prop(into, optional)] disabled: Option<Signal<bool>>,
    #[prop(into, optional)] aria_disabled: Option<Signal<bool>>,
    #[prop(optional)] submit: bool,
    #[prop(optional)] testid: Option<&'static str>,
    #[prop(into, optional)] aria_hidden: Option<Signal<bool>>,
    #[prop(into, optional)] tabindex: Option<Signal<&'static str>>,
    #[prop(into, optional)] aria_expanded: Option<Signal<bool>>,
    #[prop(into, optional)] aria_pressed: Option<Signal<String>>,
    /// 開閉対象の要素 id(絞り込み欄の開閉ボタンなど)
    #[prop(optional)]
    aria_controls: Option<&'static str>,
    #[prop(optional)] node_ref: NodeRef<leptos::html::Button>,
    on_click: impl Fn(ev::MouseEvent) + 'static,
    children: Children,
) -> impl IntoView {
    let classes = move || button_classes(variant, &class.get());
    view! {
        <button
            type=if submit { "submit" } else { "button" }
            class=classes
            disabled=move || disabled.is_some_and(|disabled| disabled.get())
            aria-label=move || aria_label.map(|label| label.get())
            aria-hidden=aria_hidden.map(|hidden| move || hidden.get().to_string())
            aria-disabled=aria_disabled.map(|value| move || value.get().to_string())
            aria-expanded=aria_expanded.map(|expanded| move || expanded.get().to_string())
            aria-pressed=aria_pressed.map(|pressed| move || pressed.get())
            aria-controls=aria_controls
            tabindex=tabindex.map(|index| move || index.get())
            data-testid=testid
            node_ref=node_ref
            on:click=on_click
        >
            {children()}
        </button>
    }
}

/// 遷移だがボタンの見た目が必要なリンク(404 の「ホームに戻る」など)
#[component]
pub fn LinkButton(
    variant: ButtonVariant,
    #[prop(into, optional)] class: Signal<String>,
    #[prop(into)] href: String,
    /// 遷移前に実行する処理(下書きの適用など)。SPA 遷移の横取りより先に走る
    #[prop(optional)]
    on_click: Option<Callback<()>>,
    #[prop(into, optional)] aria_label: Option<Signal<String>>,
    children: Children,
) -> impl IntoView {
    let classes = move || button_classes(variant, &class.get());
    view! {
        <a
            href=href
            class=classes
            aria-label=move || aria_label.as_ref().map(|label| label.get())
            on:click=move |_| {
                if let Some(on_click) = on_click {
                    on_click.run(());
                }
            }
        >
            {children()}
        </a>
    }
}

#[cfg(test)]
mod tests;
