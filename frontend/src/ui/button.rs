use leptos::ev;
use leptos::prelude::*;

/// features/ では <button> を直書きしない。種類は enum の props で受け、
/// 誤った組み合わせを型で防ぐ。見た目の変更は各 variant のクラスで行う。

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ButtonVariant {
    /// 黒塗りの主ボタン
    Primary(ButtonSize),
    /// 枠線の副ボタン
    Secondary(ButtonSize),
    /// 文字色を落とした副ボタン(絞り込みの解除など)
    SecondarySoft(ButtonSize),
    /// 赤塗りの危険確定ボタン(確認モーダル)
    DangerSolid(ButtonSize),
    /// 文字だけの静かな危険操作(格下げした削除導線)
    DangerGhost,
    /// サイトヘッダーの反転ボタン(ログイン・メニュー)
    Header(ButtonSize),
    /// 文字だけの静かな操作(絞り込みの解除など)
    Ghost,
    /// 一覧の開閉(収益グラフの全表示など)
    Quiet,
    /// AI プロンプトのコピーなど細い枠の全幅ボタン
    Prompt,
    /// ログイン画面の輪郭ボタン
    Login,
    /// 検索フォームの送信ボタン(submit と一緒に使う)
    SearchSubmit,
    /// 読み込みエラーの再読み込み(独自ベース)
    Retry,
    /// メニュー項目(role="menuitem" と一緒に使う)
    MenuItem,
    /// メニューの危険な項目(role="menuitem" と一緒に使う)
    MenuItemDanger,
    /// 銘柄名のコピー(文字そのものがボタン)
    CopyName,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ButtonSize {
    /// 行内の小さな操作(検索条件の解除など)
    Xs,
    /// 小さめの操作
    Sm,
    /// 標準
    Md,
    /// レールに縦積みする全幅
    Fill,
}

const BASE: &str = "inline-flex items-center justify-center rounded-md font-bold transition-[background-color,border-color,color,box-shadow,transform] focus:outline-none focus:ring-2 focus:ring-accent-bright/50 focus:ring-offset-2 disabled:cursor-not-allowed disabled:opacity-60";

fn sized(variant: &'static str, size: ButtonSize) -> String {
    format!("{BASE} {variant} {}", size.class())
}

impl ButtonVariant {
    fn classes(self) -> String {
        match self {
            Self::Primary(size) => sized(
                "border border-ink bg-ink text-text-inverse shadow-edge-lit hover:bg-ink-hover active:bg-ink",
                size,
            ),
            Self::Secondary(size) => sized(
                "border border-border-strong bg-surface text-text hover:border-border-bold hover:bg-surface-sunken",
                size,
            ),
            Self::SecondarySoft(size) => sized(
                "border border-border-strong bg-surface text-text-soft hover:border-border-bold hover:bg-surface-sunken",
                size,
            ),
            Self::DangerSolid(size) => sized(
                "border border-negative bg-negative text-text-inverse shadow-sm hover:border-negative-strong hover:bg-negative-strong",
                size,
            ),
            Self::DangerGhost => {
                "inline-flex w-full items-center justify-center rounded-md px-3 py-2 text-sm font-semibold text-negative transition-colors hover:bg-negative-soft focus:outline-none focus-visible:ring-2 focus-visible:ring-negative/50 focus-visible:ring-offset-2 disabled:cursor-not-allowed disabled:opacity-60 max-sm:min-h-11".to_string()
            }
            Self::Header(size) => sized(
                "no-print border border-text-inverse/70 text-text-inverse hover:bg-surface hover:text-ink",
                size,
            ),
            Self::Ghost => "text-sm text-text-deep hover:underline".to_string(),
            Self::Quiet => "chart-toggle-button".to_string(),
            Self::Prompt => "review-prompt-button".to_string(),
            Self::Login => "login-button".to_string(),
            Self::SearchSubmit => "search-submit".to_string(),
            Self::Retry => {
                "inline-flex min-h-11 items-center justify-center rounded-md border border-ink bg-ink px-4 text-sm font-bold text-text-inverse transition-colors hover:bg-ink-hover focus:outline-none focus-visible:ring-2 focus-visible:ring-accent-bright/50 focus-visible:ring-offset-2".to_string()
            }
            Self::MenuItem => {
                "w-full px-3 py-2 text-left text-sm font-semibold hover:bg-surface-raised"
                    .to_string()
            }
            Self::MenuItemDanger => {
                "w-full px-3 py-2 text-left text-sm font-bold text-negative hover:bg-negative/10"
                    .to_string()
            }
            Self::CopyName => "copyable-name".to_string(),
        }
    }
}

impl ButtonSize {
    fn class(self) -> &'static str {
        match self {
            Self::Xs => "px-2 py-1 text-xs max-sm:min-h-11",
            Self::Sm => "px-3 py-1.5 text-sm max-sm:min-h-11",
            Self::Md => "px-4 py-2 text-sm max-sm:min-h-11",
            Self::Fill => "h-11 w-full px-3 text-sm max-sm:min-h-11",
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
            tabindex=tabindex.map(|index| move || index.get())
            data-testid=testid
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
    href: &'static str,
    children: Children,
) -> impl IntoView {
    let classes = move || button_classes(variant, &class.get());
    view! { <a href=href class=classes>{children()}</a> }
}

/// アイコンのみのボタン
#[component]
pub fn IconButton(
    #[prop(into)] aria_label: Signal<String>,
    #[prop(into, optional)] disabled: Option<Signal<bool>>,
    on_click: impl Fn(ev::MouseEvent) + 'static,
    children: Children,
) -> impl IntoView {
    view! {
        <button
            type="button"
            class="modal-close-button"
            aria-label=move || aria_label.get()
            disabled=move || disabled.is_some_and(|disabled| disabled.get())
            on:click=on_click
        >
            {children()}
        </button>
    }
}

#[cfg(test)]
mod tests;
