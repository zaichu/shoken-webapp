use crate::ui::amount::{Amount, AmountKind};
use crate::ui::card::{Card, CardVariant};
use leptos::prelude::*;

#[component]
pub fn DividendStatGrid(children: Children) -> impl IntoView {
    view! { <div class="grid grid-cols-1 gap-4 md:grid-cols-3">{children()}</div> }
}

#[component]
pub fn DividendTotalGrid(children: Children) -> impl IntoView {
    view! {
        <div class="grid grid-cols-1 gap-4 md:grid-cols-3 mt-4 pt-4 border-t border-border-subtle">
            {children()}
        </div>
    }
}

#[component]
pub fn ReceiptStatLabel(children: Children) -> impl IntoView {
    view! { <p class="text-cell font-bold text-text-muted mb-1">{children()}</p> }
}

#[component]
pub fn DividendStatCard(
    label: &'static str,
    children: Children,
    #[prop(optional_no_strip)] hint: Option<Signal<Option<&'static str>>>,
    #[prop(optional)] title_hint: bool,
) -> impl IntoView {
    view! {
        <Card variant=CardVariant::Sunken>
            <ReceiptStatLabel>{label}</ReceiptStatLabel>
            <p
                class="text-summary-secondary font-bold tabular-nums text-text-deep"
                title=move || {
                    title_hint.then(|| hint.and_then(|hint| hint.get())).flatten()
                }
            >
                {children()}
            </p>
            {move || {
                hint.and_then(|hint| hint.get()).map(|text| {
                    view! { <p class="mt-0.5 text-cell text-text-subtle">{text}</p> }
                })
            }}
        </Card>
    }
}

#[component]
pub fn DividendTotalCard(
    label: &'static str,
    #[prop(into)] text: String,
    negative: bool,
    positive: bool,
    rate: Signal<Option<(String, bool, bool)>>,
) -> impl IntoView {
    view! {
        <Card variant=CardVariant::Sunken>
            <ReceiptStatLabel>{label}</ReceiptStatLabel>
            <p class="text-summary-secondary font-bold tabular-nums text-ink">
                <Amount kind=AmountKind::Delta text=text negative=negative positive=positive />
                {move || {
                    rate.get().map(|(rate_text, rate_negative, rate_positive)| view! {
                        " ("
                        <Amount
                            kind=AmountKind::Delta
                            class="text-cell font-normal ml-1"
                            text=rate_text
                            negative=rate_negative
                            positive=rate_positive
                        />
                        ")"
                    })
                }}
            </p>
        </Card>
    }
}

#[component]
pub fn DividendSummaryTitle(children: Children) -> impl IntoView {
    view! { <div class="flex items-center gap-2">{children()}</div> }
}

#[component]
pub fn DividendSummaryBody(expanded: RwSignal<bool>, children: Children) -> impl IntoView {
    view! {
        <div id="receipt-summary-body" hidden=move || !expanded.get() class="pt-3">
            {children()}
        </div>
    }
}

#[component]
pub fn ReceiptDialogLabel(children: Children) -> impl IntoView {
    view! { <label class="mb-1 block text-label font-bold text-text-deep">{children()}</label> }
}

#[component]
pub fn ReceiptDialogStack(children: Children) -> impl IntoView {
    view! { <div class="flex flex-col gap-4">{children()}</div> }
}

#[component]
pub fn ReceiptDialogFooter(children: Children) -> impl IntoView {
    view! { <div class="flex w-full items-center justify-between gap-2">{children()}</div> }
}

#[component]
pub fn PeriodOptionWrap(children: Children) -> impl IntoView {
    view! { <div class="mt-2 flex flex-wrap gap-1.5">{children()}</div> }
}

#[component]
pub fn PeriodFlatOptions(children: Children) -> impl IntoView {
    view! { <div class="flex flex-wrap gap-1.5">{children()}</div> }
}

#[component]
pub fn PeriodYearMonthRow(children: Children) -> impl IntoView {
    view! { <div class="flex items-center gap-1">{children()}</div> }
}

#[component]
pub fn PeriodNativeSelect(
    #[prop(into)] aria_label: String,
    value: Memo<String>,
    on_change: impl Fn(String) + Send + Sync + 'static,
    children: Children,
) -> impl IntoView {
    view! {
        <select
            class="h-9 px-3 border border-border-strong bg-surface rounded-lg text-label max-sm:min-h-11"
            aria-label=aria_label
            prop:value=move || value.get()
            on:change=move |ev| on_change(event_target_value(&ev))
        >
            {children()}
        </select>
    }
}

#[component]
fn PeriodDateInput(
    #[prop(into)] aria_label: String,
    value: Memo<String>,
    on_input: impl Fn(String) + Send + Sync + 'static,
) -> impl IntoView {
    view! {
        <input
            type="date"
            class="h-10 px-3 border border-border-strong bg-surface rounded-lg text-label w-full max-sm:min-h-11"
            aria-label=aria_label
            data-empty=move || value.get().is_empty().then_some("true")
            prop:value=move || value.get()
            on:input=move |ev| on_input(event_target_value(&ev))
        />
    }
}

#[component]
fn PeriodFieldLabel(children: Children) -> impl IntoView {
    view! { <label class="block text-cell text-text-muted mb-1">{children()}</label> }
}

#[component]
pub fn PeriodDateField(
    label: &'static str,
    value: Memo<String>,
    on_input: impl Fn(String) + Send + Sync + 'static,
) -> impl IntoView {
    view! {
        <div class="mt-2 max-w-xs">
            <PeriodFieldLabel>{label}</PeriodFieldLabel>
            <PeriodDateInput aria_label=label.to_string() value=value on_input=on_input />
        </div>
    }
}

#[component]
pub fn PeriodRangeWrap(children: Children) -> impl IntoView {
    view! { <div class="mt-2 flex flex-wrap gap-2 items-end">{children()}</div> }
}

#[component]
pub fn PeriodRangeField(
    label: &'static str,
    value: Memo<String>,
    on_input: impl Fn(String) + Send + Sync + 'static,
) -> impl IntoView {
    view! {
        <div class="flex-1 min-w-40">
            <PeriodFieldLabel>{label}</PeriodFieldLabel>
            <PeriodDateInput aria_label=label.to_string() value=value on_input=on_input />
        </div>
    }
}

#[component]
pub fn ReceiptItemHead(children: Children) -> impl IntoView {
    view! { <div class="flex min-w-0 flex-col gap-1.5 px-3 py-3">{children()}</div> }
}

#[component]
pub fn ReceiptItemTitleRow(children: Children) -> impl IntoView {
    view! { <div class="flex min-w-0 items-center gap-2">{children()}</div> }
}

#[component]
pub fn ReceiptItemMeta(children: Children) -> impl IntoView {
    view! { <div class="flex items-center gap-2 text-cell text-text-muted">{children()}</div> }
}

#[component]
pub fn ReceiptItemDetails(children: Children) -> impl IntoView {
    view! { <dl class="space-y-1.5 border-t border-ink/10 px-3 py-2.5">{children()}</dl> }
}

#[component]
pub fn ReceiptMobileGroupList(children: Children) -> impl IntoView {
    view! { <div class="mt-2 space-y-2">{children()}</div> }
}

#[component]
pub fn ReceiptGroupCount(#[prop(into)] text: String) -> impl IntoView {
    view! { <span class="ml-2 text-cell font-normal text-text-muted">{text}</span> }
}

#[component]
pub fn ReceiptGroupSummaryMeta(children: Children) -> impl IntoView {
    view! {
        <span class="flex shrink-0 items-baseline gap-1 whitespace-nowrap" aria-hidden="true">
            {children()}
        </span>
    }
}

#[component]
pub fn ReceiptGroupDetails(
    #[prop(into)] id: String,
    #[prop(into)] labelledby: String,
    expanded: Memo<bool>,
    children: Children,
) -> impl IntoView {
    view! {
        <div
            id=id
            role="region"
            aria-labelledby=labelledby
            hidden=move || !expanded.get()
            class="border-t border-border-subtle bg-surface px-3 py-1"
        >
            {children()}
        </div>
    }
}

#[component]
pub fn ReceiptGroupBreakdown(children: Children) -> impl IntoView {
    view! { <dl class="space-y-1.5">{children()}</dl> }
}

#[component]
pub fn ReceiptTableBody(children: Children) -> impl IntoView {
    view! { <div class="p-0" data-testid="receipt-table-body">{children()}</div> }
}

#[component]
pub fn ReceiptTableGroupCount(#[prop(into)] text: String) -> impl IntoView {
    view! { <span class="ml-2 font-normal">{text}</span> }
}

#[component]
pub fn ReceiptMobileCardList(children: Children) -> impl IntoView {
    view! { <div class="space-y-3 p-2">{children()}</div> }
}
