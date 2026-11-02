use crate::ui::button::{Button, ButtonSize, ButtonVariant};
use crate::ui::dialog::DialogShell;
use crate::ui::labels::{DIALOG_CLOSE, SEARCH, SHEET_CLEAR};
use leptos::prelude::*;

#[component]
pub fn SearchDialog(
    #[prop(into)] title: String,
    on_close: impl Fn() + Clone + Send + 'static,
    on_search: impl Fn() + Clone + Send + 'static,
    on_clear: impl Fn() + Clone + Send + 'static,
    #[prop(optional)] testid: Option<&'static str>,
    children: Children,
) -> impl IntoView {
    let footer_close = on_close.clone();
    let head_title = title.clone();
    let panel_testid = testid.unwrap_or("search-dialog");
    view! {
        <DialogShell title=title on_close=on_close testid=panel_testid>
            <div class="search-dialog-head">
                <h2 class="search-dialog-title">{head_title}</h2>
            </div>
            <div class="search-dialog-body">{children()}</div>
            <div class="search-dialog-footer">
                <Button
                    variant=ButtonVariant::Ghost
                    testid="search-dialog-clear"
                    on_click=move |_| on_clear()
                >
                    {SHEET_CLEAR}
                </Button>
                <span class="search-dialog-footer-actions">
                    <Button
                        variant=ButtonVariant::Ghost
                        testid="search-dialog-close"
                        on_click=move |_| footer_close()
                    >
                        {DIALOG_CLOSE}
                    </Button>
                    <Button
                        variant=ButtonVariant::Primary(ButtonSize::Md)
                        testid="search-dialog-search"
                        on_click=move |_| on_search()
                    >
                        {SEARCH}
                    </Button>
                </span>
            </div>
        </DialogShell>
    }
}
