use leptos::ev;
use leptos::prelude::*;

/// features/ では <button> を直書きしない。種類は enum の props で受け、
/// 誤った組み合わせを型で防ぐ。見た目の変更は各 variant のクラスで行う。

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ButtonVariant {
    /// 黒塗りの主ボタン(size と組み合わせる)
    Primary,
    /// 枠線の副ボタン(size と組み合わせる)
    Secondary,
    /// 赤枠の危険操作(size と組み合わせる)
    Danger,
    /// 文字だけの静かな操作(絞り込みの解除など)
    Ghost,
    /// 一覧の開閉(収益グラフの全表示など)。.chart-toggle-button
    Quiet,
    /// AI プロンプトのコピーなど細い枠の全幅ボタン。.review-prompt-button
    Prompt,
    /// ログイン画面の輪郭ボタン。.login-button
    Login,
    /// 検索フォームの送信ボタン。.search-submit(submit と一緒に使う)
    SearchSubmit,
    /// サイトヘッダーの反転ボタン(ログイン・メニュー)
    Header,
    /// 読み込みエラーの再読み込み(独自ベース)
    Retry,
    /// メニュー項目(role="menuitem" と一緒に使う)
    MenuItem,
    /// メニューの危険な項目(role="menuitem" と一緒に使う)
    MenuItemDanger,
    /// 銘柄名のコピー(文字そのものがボタン)。.copyable-name
    CopyName,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ButtonSize {
    /// px-2 py-1 text-xs(小さな解除ボタン)
    Xs,
    /// px-3 py-1.5 text-sm
    Sm,
    /// px-4 py-2 text-sm
    Md,
    /// h-11 px-5(検索フォームの送信など)
    Lg,
    /// h-11 w-full px-3(レールに縦積み)
    Fill,
    /// min-h-11 px-4(インラインの再読み込みなど)
    Block,
}

const BASE: &str = "inline-flex items-center justify-center rounded-md font-bold transition-[background-color,border-color,color,box-shadow,transform] focus:outline-none focus:ring-2 focus:ring-accent-bright/50 focus:ring-offset-2 disabled:cursor-not-allowed disabled:opacity-60";

impl ButtonVariant {
    /// 見た目クラス。composed() の variant は BASE + size と組み合わせ、
    /// それ以外は単体で完結する(size は使わない)
    fn class(self) -> &'static str {
        match self {
            Self::Primary => {
                "border border-ink bg-ink text-text-inverse shadow-edge-lit hover:bg-ink-hover active:bg-ink"
            }
            Self::Secondary => {
                "border border-border-strong bg-surface text-text hover:border-border-bold hover:bg-surface-sunken"
            }
            Self::Danger => {
                "border border-negative text-negative hover:bg-negative hover:text-text-inverse"
            }
            Self::Header => {
                "no-print border border-text-inverse/70 text-text-inverse hover:bg-surface hover:text-ink"
            }
            Self::Ghost => "text-sm text-text-deep hover:underline",
            Self::Quiet => "chart-toggle-button",
            Self::Prompt => "review-prompt-button",
            Self::Login => "login-button",
            Self::SearchSubmit => "search-submit",
            Self::Retry => {
                "inline-flex min-h-11 items-center justify-center rounded-md border border-ink bg-ink px-4 text-sm font-bold text-text-inverse transition-colors hover:bg-ink-hover focus:outline-none focus-visible:ring-2 focus-visible:ring-accent-bright/50 focus-visible:ring-offset-2"
            }
            Self::MenuItem => {
                "w-full px-3 py-2 text-left text-sm font-semibold hover:bg-surface-raised"
            }
            Self::MenuItemDanger => {
                "w-full px-3 py-2 text-left text-sm font-bold text-negative hover:bg-negative/10"
            }
            Self::CopyName => "copyable-name",
        }
    }

    fn composed(self) -> bool {
        matches!(
            self,
            Self::Primary | Self::Secondary | Self::Danger | Self::Header
        )
    }
}

impl ButtonSize {
    fn class(self) -> &'static str {
        match self {
            Self::Xs => "px-2 py-1 text-xs max-sm:min-h-11",
            Self::Sm => "px-3 py-1.5 text-sm max-sm:min-h-11",
            Self::Md => "px-4 py-2 text-sm max-sm:min-h-11",
            Self::Lg => "h-11 px-5 text-sm",
            Self::Fill => "h-11 w-full px-3 text-sm max-sm:min-h-11",
            Self::Block => "min-h-11 px-4 text-sm",
        }
    }
}

#[component]
pub fn Button(
    variant: ButtonVariant,
    #[prop(default = ButtonSize::Md)] size: ButtonSize,
    #[prop(into, optional)] class: Signal<String>,
    #[prop(into, optional)] aria_label: Option<Signal<String>>,
    #[prop(into, optional)] disabled: Option<Signal<bool>>,
    #[prop(into, optional)] aria_disabled: Option<Signal<bool>>,
    #[prop(optional)] submit: bool,
    #[prop(optional)] testid: Option<&'static str>,
    #[prop(into, optional)] aria_hidden: Option<Signal<bool>>,
    #[prop(into, optional)] tabindex: Option<Signal<&'static str>>,
    #[prop(into, optional)] data_loading: Option<Signal<bool>>,
    #[prop(optional)] role: Option<&'static str>,
    #[prop(optional)] aria_haspopup: Option<&'static str>,
    #[prop(optional)] aria_controls: Option<&'static str>,
    #[prop(into, optional)] aria_expanded: Option<Signal<bool>>,
    #[prop(optional)] aria_describedby: Option<&'static str>,
    on_click: impl Fn(ev::MouseEvent) + 'static,
    children: ChildrenFn,
) -> impl IntoView {
    let classes = move || {
        let mut classes = variant.class().to_string();
        if variant.composed() {
            classes = format!("{BASE} {classes} {}", size.class());
        }
        let extra = class.get();
        if !extra.is_empty() {
            classes.push(' ');
            classes.push_str(&extra);
        }
        classes
    };
    view! {
        <button
            type=if submit { "submit" } else { "button" }
            class=classes
            disabled=move || disabled.is_some_and(|disabled| disabled.get())
            aria-label=move || aria_label.map(|label| label.get())
            aria-hidden=aria_hidden.map(|hidden| move || hidden.get().to_string())
            aria-disabled=aria_disabled.map(|value| move || value.get().to_string())
            aria-haspopup=aria_haspopup
            aria-controls=aria_controls
            aria-expanded=aria_expanded.map(|expanded| move || expanded.get().to_string())
            aria-describedby=aria_describedby
            tabindex=tabindex.map(|index| move || index.get())
            role=role
            data-testid=testid
            data-loading=data_loading.map(|loading| move || loading.get().then_some("true"))
            on:click=move |event| on_click(event)
        >
            {children()}
        </button>
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum IconButtonVariant {
    /// 枠線の正方形(検索カードの山形など)
    Boxed,
    /// モーダルの閉じる
    Close,
}

impl IconButtonVariant {
    fn class(self) -> &'static str {
        match self {
            Self::Boxed => {
                "flex h-8 w-8 cursor-pointer items-center justify-center rounded-md border border-border-strong bg-surface text-text-soft max-sm:h-11 max-sm:w-11"
            }
            Self::Close => "modal-close-button",
        }
    }
}

/// アイコンのみのボタン。aria-label か aria-hidden のどちらかを持たせる
#[component]
pub fn IconButton(
    variant: IconButtonVariant,
    #[prop(into, optional)] class: Signal<String>,
    #[prop(into, optional)] aria_label: Option<Signal<String>>,
    #[prop(into, optional)] aria_hidden: Option<Signal<bool>>,
    #[prop(into, optional)] tabindex: Option<Signal<&'static str>>,
    #[prop(into, optional)] disabled: Option<Signal<bool>>,
    #[prop(optional)] testid: Option<&'static str>,
    on_click: impl Fn(ev::MouseEvent) + 'static,
    children: ChildrenFn,
) -> impl IntoView {
    let classes = move || {
        let mut classes = variant.class().to_string();
        let extra = class.get();
        if !extra.is_empty() {
            classes.push(' ');
            classes.push_str(&extra);
        }
        classes
    };
    view! {
        <button
            type="button"
            class=classes
            aria-label=move || aria_label.map(|label| label.get())
            aria-hidden=aria_hidden.map(|hidden| move || hidden.get().to_string())
            tabindex=tabindex.map(|index| move || index.get())
            disabled=move || disabled.is_some_and(|disabled| disabled.get())
            data-testid=testid
            on:click=move |event| on_click(event)
        >
            {children()}
        </button>
    }
}
