use crate::session::use_session;
use crate::ui::button::{Button, ButtonSize, ButtonVariant, LinkButton};
use crate::ui::card::Card;
use crate::ui::card::CardVariant;
use crate::ui::page_frame::PageFrame;
use crate::ui::site::get_initials;
use leptos::prelude::*;

#[component]
pub fn SettingsPage() -> impl IntoView {
    let session = use_session();
    let user = move || session.user.get();

    let sections = [
        ("account", "アカウント"),
        ("data", "データ"),
        ("danger", "危険な操作"),
    ];

    view! {
        <PageFrame title="設定">
            <div class="flex flex-col lg:flex-row gap-7">
                <nav class="lg:w-48 shrink-0" aria-label="設定セクション">
                    <ul class="flex flex-col gap-1" role="list">
                        {sections.iter().map(|(id, label)| {
                            let id = *id;
                            let label = *label;
                            let is_active = move || {
                                true
                            };
                            view! {
                                <li>
                                    <a
                                        href={format!("/settings#{}", id)}
                                        class=move || {
                                            let base = "flex min-h-11 items-center gap-3 rounded-lg px-3 py-2 text-sm font-medium transition-colors no-underline";
                                            if is_active() {
                                                format!("{} bg-brand text-text-inverse", base)
                                            } else {
                                                format!("{} text-text-deep hover:bg-surface-raised", base)
                                            }
                                        }
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
                                <SectionData />
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
                    <div>
                        <div class="font-bold text-lg">{display_name}</div>
                        <div class="text-sm text-text-muted">{user.email.clone()} / {login_method_text}</div>
                    </div>
                </div>
            </Card>
        </section>
    }
}

#[component]
fn SectionData() -> impl IntoView {
    view! {
        <section id="data" aria-labelledby="data-heading">
            <Card variant=CardVariant::Panel class="p-5">
                <h2 id="data-heading" class="text-lg font-bold mb-4">"データ"</h2>
                <div class="space-y-3 border-t border-ink/10">
                    <div class="flex items-center justify-between py-3">
                        <div>
                            <div class="font-medium">"CSV管理"</div>
                            <div class="text-xs text-text-muted">"取り込み・保存済みデータの削除"</div>
                        </div>
                        <LinkButton
                            variant=ButtonVariant::Ghost
                            href="/receipts/csv"
                            class="no-print"
                        >
                            "CSV管理へ →"
                        </LinkButton>
                    </div>
                </div>
            </Card>
        </section>
    }
}

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
                            class="no-print text-negative border-negative-border hover:bg-negative/10"
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
                            class="no-print text-negative border-negative-border hover:bg-negative/10"
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
                            class="no-print text-negative border-negative-border hover:bg-negative/10"
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
                            class="no-print text-negative border-negative-border hover:bg-negative/10"
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
                            class="text-negative border-negative-border hover:bg-negative/10"
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
