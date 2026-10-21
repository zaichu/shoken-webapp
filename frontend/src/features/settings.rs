mod export;

use crate::session::use_session;
use crate::settings::{
    AmountUnit, AssetSort, IDLE_LOGOUT_CHOICES, PromptColumn, Theme, use_settings,
};
use crate::ui::button::{Button, ButtonSize, ButtonVariant, LinkButton};
use crate::ui::card::Card;
use crate::ui::card::CardVariant;
use crate::ui::page_frame::PageFrame;
use crate::ui::segmented::Segmented;
use crate::ui::select::Select;
use crate::ui::site::get_initials;
use export::ExportKind;
use leptos::prelude::*;

#[component]
pub fn SettingsPage() -> impl IntoView {
    let session = use_session();
    let user = move || session.user.get();

    let sections = [
        ("account", "アカウント"),
        ("display", "表示"),
        ("data", "データ"),
        ("security", "セキュリティ"),
        ("danger", "危険な操作"),
    ];

    view! {
        <PageFrame title="設定">
            <div class="flex flex-col lg:flex-row gap-7">
                <nav class="hidden lg:block lg:w-48 shrink-0" aria-label="設定セクション">
                    <ul class="flex flex-col gap-1" role="list">
                        {sections.iter().map(|(id, label)| {
                            let id = *id;
                            let label = *label;
                            view! {
                                <li>
                                    <a
                                        href={format!("/settings#{}", id)}
                                        class="flex min-h-11 items-center gap-3 rounded-lg px-3 py-2 text-sm font-medium text-text-deep transition-colors no-underline hover:bg-surface-raised"
                                    >
                                        {label}
                                    </a>
                                </li>
                            }
                        }).collect_view()}
                    </ul>
                </nav>

                <div class="flex-1 min-w-0 space-y-4">
                    {move || {
                        if let Some(user) = user() {
                            view! {
                                <SectionAccount user=user />
                                <SectionDisplay />
                                <SectionData />
                                <SectionSecurity />
                                <SectionDanger />
                            }.into_any()
                        } else {
                            view! { <div class="text-center py-12">"ログインが必要です"</div> }.into_any()
                        }
                    }}
                </div>
            </div>
        </PageFrame>
    }
}

#[component]
fn SectionAccount(user: crate::api::dto::SessionUser) -> impl IntoView {
    let display_name = user
        .name
        .clone()
        .filter(|name| !name.is_empty())
        .unwrap_or_else(|| user.email.clone());
    let initials = get_initials(user.name.as_deref(), Some(&user.email));
    let login_method_text = "Google でログイン中";

    view! {
        <section id="account" aria-labelledby="account-heading">
            <Card variant=CardVariant::Panel class="p-5">
                <h2 id="account-heading" class="text-lg font-bold mb-4">"アカウント"</h2>
                <div class="flex items-center gap-4">
                    <span class="flex h-14 w-14 shrink-0 items-center justify-center rounded-full bg-gold text-brand font-bold text-2xl">
                        {initials}
                    </span>
                    <div class="min-w-0">
                        <div class="font-bold text-lg break-words">{display_name}</div>
                        <div class="text-sm text-text-muted break-all">{user.email.clone()}</div>
                        <div class="text-sm text-text-muted">{login_method_text}</div>
                    </div>
                </div>
            </Card>
        </section>
    }
}

/// セクション内の1行。右側に操作(子要素)を置く
#[component]
fn SettingRow(
    title: &'static str,
    #[prop(optional)] note: &'static str,
    children: Children,
) -> impl IntoView {
    view! {
        <div class="flex items-center justify-between gap-3 py-3 max-sm:flex-col max-sm:items-start">
            <div class="min-w-0">
                <div class="font-medium">{title}</div>
                {(!note.is_empty()).then(|| {
                    view! { <div class="text-xs text-text-muted">{note}</div> }
                })}
            </div>
            <div class="shrink-0 flex items-center gap-2">{children()}</div>
        </div>
    }
}

#[component]
fn SectionDisplay() -> impl IntoView {
    let settings = use_settings();
    let asset_sort = settings.asset_sort;
    let initial_tab = settings.receipts_initial_tab;
    let amount_unit = settings.amount_unit;
    let theme = settings.theme;

    view! {
        <section id="display" aria-labelledby="display-heading">
            <Card variant=CardVariant::Panel class="p-5">
                <h2 id="display-heading" class="text-lg font-bold">"表示"</h2>
                <p class="text-xs text-text-muted mb-2">"この端末のブラウザに保存されます"</p>
                <div class="divide-y divide-ink/10 border-t border-ink/10">
                    <SettingRow title="テーマ" note="端末の設定に合わせることもできます">
                        <Segmented
                            options=&THEME_SEGMENTS
                            value=Signal::derive(move || theme.get().value())
                            aria_label="テーマ"
                            testid="setting-theme"
                            on_change=move |value| {
                                if let Some(parsed) = Theme::from_value(&value) {
                                    theme.set(parsed);
                                }
                            }
                        />
                    </SettingRow>
                    <SettingRow title="資産管理の並び順" note="開いたときの初期の並び">
                        <Select
                            value=Signal::derive(move || asset_sort.get().value())
                            aria_label="資産管理の初期の並び順".to_string()
                            testid="setting-asset-sort"
                            class="min-w-44 rounded-md border border-border-strong bg-surface px-3 py-2 text-sm max-sm:min-h-11".to_string()
                            on_change=move |value| {
                                if let Some(sort) = AssetSort::parse(&value) {
                                    asset_sort.set(sort);
                                }
                            }
                        >
                            {AssetSort::ALL.iter().map(|sort| {
                                view! {
                                    <option value={sort.value()}>{sort.label()}</option>
                                }
                            }).collect_view()}
                        </Select>
                    </SettingRow>
                    <SettingRow title="取引明細の初期タブ">
                        <Select
                            value=Signal::derive(move || initial_tab.get().kind_param().to_string())
                            aria_label="取引明細の初期タブ".to_string()
                            testid="setting-receipts-tab"
                            class="min-w-44 rounded-md border border-border-strong bg-surface px-3 py-2 text-sm max-sm:min-h-11".to_string()
                            on_change=move |value| {
                                if let Some(tab) = crate::features::receipts::ReceiptsTab::ALL
                                    .iter()
                                    .copied()
                                    .find(|tab| tab.kind_param() == value)
                                {
                                    initial_tab.set(tab);
                                }
                            }
                        >
                            {crate::features::receipts::ReceiptsTab::ALL.iter().map(|tab| {
                                view! {
                                    <option value={tab.kind_param()}>{tab.label()}</option>
                                }
                            }).collect_view()}
                        </Select>
                    </SettingRow>
                    <SettingRow title="金額の表示単位" note="合計の欄に使います。明細の行は円のまま表示します">
                        <Segmented
                            options=&[
                                ("yen", "円"),
                                ("sen", "千円"),
                                ("man", "万円"),
                            ]
                            value=Signal::derive(move || amount_unit.get().value())
                            aria_label="金額の表示単位"
                            testid="setting-amount-unit"
                            on_change=move |value| {
                                if let Some(unit) = AmountUnit::from_value(&value) {
                                    amount_unit.set(unit);
                                }
                            }
                        />
                    </SettingRow>
                </div>
            </Card>
        </section>
    }
}

#[component]
fn SectionData() -> impl IntoView {
    let settings = use_settings();
    let review_prompt = settings.review_prompt;
    let exporting = RwSignal::new(Option::<ExportKind>::None);
    let export_message = RwSignal::new(Option::<String>::None);
    let export_failed = RwSignal::new(false);
    let editor_open = RwSignal::new(false);

    let on_export = move |kind: ExportKind| {
        exporting.set(Some(kind));
        export_message.set(None);
        export_failed.set(false);
        leptos::task::spawn_local(async move {
            match export::fetch_csv(kind).await {
                Ok((filename, content)) => {
                    export::download_csv(&filename, &content);
                    export_message.set(Some(format!("{filename} を保存しました")));
                }
                Err(error) => {
                    export_failed.set(true);
                    export_message.set(Some(format!(
                        "{}のエクスポートに失敗しました: {}",
                        kind.label(),
                        error.user_message()
                    )));
                }
            }
            exporting.set(None);
        });
    };

    view! {
        <section id="data" aria-labelledby="data-heading">
            <Card variant=CardVariant::Panel class="p-5">
                <h2 id="data-heading" class="text-lg font-bold mb-4">"データ"</h2>
                <div class="divide-y divide-ink/10 border-t border-ink/10">
                    <SettingRow title="CSVでエクスポート" note="保存済みのデータをCSVでダウンロード">
                        {ExportKind::ALL.iter().map(|kind| {
                            let kind = *kind;
                            view! {
                                <Button
                                    variant=ButtonVariant::Secondary(ButtonSize::Sm)
                                    class="whitespace-nowrap"
                                    disabled=Signal::derive(move || exporting.get().is_some())
                                    testid=match kind {
                                        ExportKind::Dividend => "export-dividends",
                                        ExportKind::AssetBalance => "export-asset-balances",
                                        ExportKind::Transaction => "export-transactions",
                                    }
                                    on_click=move |_| on_export(kind)
                                >
                                    {move || {
                                        if exporting.get() == Some(kind) {
                                            "取得中…".to_string()
                                        } else {
                                            kind.label().to_string()
                                        }
                                    }}
                                </Button>
                            }
                        }).collect_view()}
                    </SettingRow>
                    {move || {
                        export_message.get().map(|message| {
                            let tone = if export_failed.get() {
                                "text-negative"
                            } else {
                                "text-text-muted"
                            };
                            view! { <p class=format!("pb-2 text-xs {tone}")>{message}</p> }
                        })
                    }}
                    <SettingRow title="CSVの取り込み・保存済みデータの削除">
                        <LinkButton
                            variant=ButtonVariant::Ghost
                            href="/receipts/csv"
                            class="no-print shrink-0 whitespace-nowrap"
                        >
                            "CSV管理へ →"
                        </LinkButton>
                    </SettingRow>
                    <SettingRow title="AI総評プロンプトの書式" note="前置き文と、含める列を選ぶ">
                        <Button
                            variant=ButtonVariant::Secondary(ButtonSize::Sm)
                            class="whitespace-nowrap"
                            aria_expanded=Signal::derive(move || editor_open.get())
                            aria_controls="review-prompt-editor"
                            testid="review-prompt-edit"
                            on_click=move |_| editor_open.update(|open| *open = !*open)
                        >
                            {move || if editor_open.get() { "閉じる" } else { "編集" }}
                        </Button>
                    </SettingRow>
                    {move || {
                        editor_open.get().then(|| {
                            view! {
                                <div id="review-prompt-editor" class="pb-3" data-testid="review-prompt-editor">
                                    <label class="block text-xs font-medium text-text-muted mb-1" for="review-prompt-preamble">
                                        "前置き文"
                                    </label>
                                    <textarea
                                        id="review-prompt-preamble"
                                        class="w-full rounded-md border border-border-strong bg-surface px-3 py-2 text-sm text-text"
                                        rows="3"
                                        prop:value=move || review_prompt.get().preamble
                                        on:input=move |ev| {
                                            let value = event_target_value(&ev);
                                            review_prompt.update(|pref| pref.preamble = value);
                                        }
                                    />
                                    <fieldset class="mt-3">
                                        <legend class="text-xs font-medium text-text-muted mb-1">
                                            "プロンプトに含める列(銘柄コード・銘柄名は常に含む)"
                                        </legend>
                                        <div class="flex flex-wrap gap-x-4 gap-y-1">
                                            {PromptColumn::ALL.iter().map(|column| {
                                                let column = *column;
                                                view! {
                                                    <label class="flex items-center gap-1.5 text-sm">
                                                        <input
                                                            type="checkbox"
                                                            class="h-4 w-4 accent-brand-text"
                                                            prop:checked=move || review_prompt.get().has_column(column)
                                                            on:change=move |ev| {
                                                                let checked = event_target_checked(&ev);
                                                                review_prompt.update(|pref| {
                                                                    pref.set_column(column, checked);
                                                                });
                                                            }
                                                        />
                                                        {column.label()}
                                                    </label>
                                                }
                                            }).collect_view()}
                                        </div>
                                    </fieldset>
                                </div>
                            }
                        })
                    }}
                </div>
            </Card>
        </section>
    }
}

#[component]
fn SectionSecurity() -> impl IntoView {
    let settings = use_settings();
    let idle_minutes = settings.idle_logout_minutes;

    view! {
        <section id="security" aria-labelledby="security-heading">
            <Card variant=CardVariant::Panel class="p-5">
                <h2 id="security-heading" class="text-lg font-bold mb-4">"セキュリティ"</h2>
                <div class="divide-y divide-ink/10 border-t border-ink/10">
                    <SettingRow title="操作がないときの自動ログアウト" note="共有のパソコンでは短くしておくと安心です">
                        <Segmented
                            options=&IDLE_SEGMENTS
                            value=Signal::derive(move || idle_minutes.get().to_string())
                            aria_label="自動ログアウトまでの時間"
                            testid="setting-idle-minutes"
                            on_change=move |value| {
                                if let Ok(minutes) = value.parse::<u32>()
                                    && IDLE_LOGOUT_CHOICES.contains(&minutes)
                                {
                                    idle_minutes.set(minutes);
                                }
                            }
                        />
                    </SettingRow>
                </div>
            </Card>
        </section>
    }
}

const IDLE_SEGMENTS: [(&str, &str); 3] = [("15", "15分"), ("30", "30分"), ("60", "60分")];

const THEME_SEGMENTS: [(&str, &str); 3] = [
    ("system", "端末に合わせる"),
    ("light", "ライト"),
    ("dark", "ダーク"),
];

#[component]
fn SectionDanger() -> impl IntoView {
    let delete_confirm_open = use_context::<crate::ui::site::AccountDeleteConfirm>()
        .map(|confirm| confirm.0)
        .unwrap_or_else(|| RwSignal::new(false));

    view! {
        <section id="danger" aria-labelledby="danger-heading">
            <Card variant=CardVariant::Panel class="p-5 border-negative-border bg-negative/5">
                <h2 id="danger-heading" class="text-lg font-bold text-negative mb-4">"危険な操作"</h2>
                <div class="space-y-3 border-t border-negative-border/50">
                    <div class="flex items-center justify-between py-3">
                        <div>
                            <div class="font-medium text-negative">"取引明細：配当金を削除"</div>
                            <div class="text-xs text-text-muted">"取り込んだ配当金データ。アカウントは残ります。取り消せません"</div>
                        </div>
                        <LinkButton
                            variant=ButtonVariant::Secondary(ButtonSize::Sm)
                            href="/receipts/csv/delete?kind=dividend"
                            class="no-print shrink-0 whitespace-nowrap text-negative border-negative-border hover:bg-negative/10"
                            aria_label="配当金データを削除"
                        >
                            "削除…"
                        </LinkButton>
                    </div>
                    <div class="flex items-center justify-between py-3">
                        <div>
                            <div class="font-medium text-negative">"取引明細：国内株式を削除"</div>
                            <div class="text-xs text-text-muted">"取り込んだ国内株式売買データ。アカウントは残ります。取り消せません"</div>
                        </div>
                        <LinkButton
                            variant=ButtonVariant::Secondary(ButtonSize::Sm)
                            href="/receipts/csv/delete?kind=stock"
                            class="no-print shrink-0 whitespace-nowrap text-negative border-negative-border hover:bg-negative/10"
                            aria_label="国内株式データを削除"
                        >
                            "削除…"
                        </LinkButton>
                    </div>
                    <div class="flex items-center justify-between py-3">
                        <div>
                            <div class="font-medium text-negative">"取引明細：投資信託を削除"</div>
                            <div class="text-xs text-text-muted">"取り込んだ投資信託売買データ。アカウントは残ります。取り消せません"</div>
                        </div>
                        <LinkButton
                            variant=ButtonVariant::Secondary(ButtonSize::Sm)
                            href="/receipts/csv/delete?kind=fund"
                            class="no-print shrink-0 whitespace-nowrap text-negative border-negative-border hover:bg-negative/10"
                            aria_label="投資信託データを削除"
                        >
                            "削除…"
                        </LinkButton>
                    </div>
                    <div class="flex items-center justify-between py-3">
                        <div>
                            <div class="font-medium text-negative">"資産管理データを削除"</div>
                            <div class="text-xs text-text-muted">"保存済みの保有銘柄データ。アカウントは残ります。取り消せません"</div>
                        </div>
                        <LinkButton
                            variant=ButtonVariant::Secondary(ButtonSize::Sm)
                            href="/assetbalance/csv/delete"
                            class="no-print shrink-0 whitespace-nowrap text-negative border-negative-border hover:bg-negative/10"
                            aria_label="資産管理データを削除"
                        >
                            "削除…"
                        </LinkButton>
                    </div>
                    <div class="flex items-center justify-between py-3">
                        <div>
                            <div class="font-medium text-negative">"アカウントを削除"</div>
                            <div class="text-xs text-text-muted">"データもすべて消えます。取り消せません"</div>
                        </div>
                        <Button
                            variant=ButtonVariant::Secondary(ButtonSize::Sm)
                            class="shrink-0 whitespace-nowrap text-negative border-negative-border hover:bg-negative/10"
                            on_click=move |_| delete_confirm_open.set(true)
                            aria_label="アカウントを削除"
                        >
                            "削除…"
                        </Button>
                    </div>
                </div>
            </Card>
        </section>
    }
}
