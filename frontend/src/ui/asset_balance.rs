//! 資産管理の見た目。余白は features/ ではなくこの中の部品が持つ。

use crate::ui::amount::{Amount, AmountKind};
use crate::ui::button::{Button, ButtonVariant};
use crate::ui::empty_state::EmptyState;
use crate::ui::labels;
use crate::ui::state::{Alert, AlertVariant};
use leptos::prelude::*;

/// 一覧の上の CSV・構成比への入口。`PageFrame` の links に渡す
#[component]
pub fn AssetPageLinks(children: Children) -> impl IntoView {
    view! { <span class="flex items-center gap-2">{children()}</span> }
}

/// 一覧(全面)の帯の知らせ。全面のため左右に溝を空ける
#[component]
pub fn AssetWorkspaceError(message: String) -> impl IntoView {
    view! {
        <div class="no-print mb-3 px-4 lg:px-0">
            <Alert variant=AlertVariant::Danger>
                <strong>{labels::ERROR_PREFIX}</strong>
                " "
                {message}
            </Alert>
        </div>
    }
}

/// 一覧(全面)の取りこぼしの知らせ。全面のため左右に溝を空ける
#[component]
pub fn AssetWorkspaceWarning(message: String, testid: &'static str) -> impl IntoView {
    view! {
        <div class="no-print mb-3 px-4 lg:px-0">
            <Alert variant=AlertVariant::Warning>
                <strong>{labels::WARNING_PREFIX}</strong>
                " "
                <span data-testid=testid>{message}</span>
            </Alert>
        </div>
    }
}

/// 下層ページの帯の知らせ。本文に溝があるため足さない
#[component]
pub fn AssetPageError(message: String) -> impl IntoView {
    view! {
        <div class="no-print mb-3">
            <Alert variant=AlertVariant::Danger>
                <strong>{labels::ERROR_PREFIX}</strong>
                " "
                {message}
            </Alert>
        </div>
    }
}

/// 一覧が空のとき。CSV 取り込みへの入口を持つ
#[component]
pub fn AssetEmptyPanel() -> impl IntoView {
    view! {
        <div class="px-4 py-4 lg:px-0">
            <EmptyState
                title=labels::ASSET_EMPTY_TITLE
                description=labels::ASSET_EMPTY_DESCRIPTION
            >
                <a
                    href="/csv"
                    class="mt-4 inline-flex items-center justify-center rounded-md border border-border-strong bg-surface px-4 py-2 text-body font-bold text-text no-underline hover:border-border-bold hover:bg-surface-sunken"
                >
                    {labels::CSV_IMPORT}
                </a>
            </EmptyState>
        </div>
    }
}

/// 下層ページが空のとき
#[component]
pub fn AssetPageEmpty(title: &'static str) -> impl IntoView {
    view! {
        <div class="px-4 py-4">
            <EmptyState title=title />
        </div>
    }
}

/// 絞り込みで全行が消えたとき。解除操作は呼び出し側が渡す
#[component]
pub fn AssetFilteredEmpty(
    title: &'static str,
    on_clear: impl Fn() + Copy + Send + Sync + 'static,
) -> impl IntoView {
    view! {
        <div class="asset-panel-padded mx-4 mt-3 lg:mx-0">
            <div>
                <EmptyState title=title />
                <div class="mt-3">
                    <Button variant=ButtonVariant::Ghost on_click=move |_| on_clear()>
                        {labels::CLEAR_SEARCH_CONDITIONS}
                    </Button>
                </div>
            </div>
        </div>
    }
}

/// 損益率の後ろ。損益額と組で使う
#[component]
pub fn AssetRateAmount(text: String, negative: bool, positive: bool) -> impl IntoView {
    view! {
        <Amount
            kind=AmountKind::Delta
            class="ml-1 text-cell"
            text=text
            negative=negative
            positive=positive
        />
    }
}

/// 前日比の日付・更新待ちの注記。前日比の値の後ろに小さく出す
#[component]
pub fn DayChangeNote(text: String) -> impl IntoView {
    view! {
        <span class="ml-1 whitespace-nowrap text-cell text-text-muted tabular-nums">
            {text}
        </span>
    }
}

/// 構成比の枠。PC もスマホも同じ最大幅で真ん中に寄せる
#[component]
pub fn CompositionFrame(children: Children) -> impl IntoView {
    view! {
        <div class="mx-auto w-full max-w-2xl" data-testid="asset-composition">
            {children()}
        </div>
    }
}

/// ファンド明細の面
#[component]
pub fn FundDetailCard(children: Children) -> impl IntoView {
    view! { <div class="asset-panel">{children()}</div> }
}

/// ファンド明細の見出し(銘柄名・口座札)
#[component]
pub fn FundDetailHeader(children: Children) -> impl IntoView {
    view! { <div class="border-b border-border p-4">{children()}</div> }
}

/// ファンド明細の見出しの下(口座札・コース)
#[component]
pub fn FundDetailMeta(children: Children) -> impl IntoView {
    view! {
        <p class="mt-1 flex items-center gap-1 text-label text-text-muted">
            {children()}
        </p>
    }
}

/// ファンド明細の数値の並び
#[component]
pub fn FundDetailBody(children: Children) -> impl IntoView {
    view! { <div class="p-4">{children()}</div> }
}

/// スマホの件数と入口の行
#[component]
pub fn AssetSearchMetaRow(children: Children) -> impl IntoView {
    view! {
        <div class="flex w-full items-center justify-between gap-2">
            {children()}
        </div>
    }
}

/// スマホの件数の横の入口の並び
#[component]
pub fn AssetSearchLinks(children: Children) -> impl IntoView {
    view! { <span class="flex items-center gap-4">{children()}</span> }
}

/// ダイアログの中の段の重ね
#[component]
pub fn AssetDialogStack(children: Children) -> impl IntoView {
    view! { <div class="flex flex-col gap-4">{children()}</div> }
}

/// ダイアログの中の1項目の見出し。操作は children に渡す
#[component]
pub fn AssetDialogField(label: &'static str, children: Children) -> impl IntoView {
    view! {
        <label class="mb-1 block text-label font-bold text-text-deep">
            {label}
            {children()}
        </label>
    }
}
