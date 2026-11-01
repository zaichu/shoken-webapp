use leptos::prelude::*;

#[component]
pub fn LedgerRow(
    #[prop(into)] date: String,
    name: impl IntoView + 'static,
    code: impl IntoView + 'static,
    amount: impl IntoView + 'static,
) -> impl IntoView {
    view! {
        <div class="ledger-row">
            <span class="ledger-date">{date}</span>
            <span class="ledger-body">
                <span class="ledger-name">{name}</span>
                <span class="ledger-code">{code}</span>
            </span>
            <span class="ledger-amount">{amount}</span>
        </div>
    }
}
