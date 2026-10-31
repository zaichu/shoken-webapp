#[derive(Clone, Debug, PartialEq)]
struct ListQuery {
    page: i64,
    q: String,
    email: String,
    place: String,
    sort_by: &'static str,
    descending: bool,
}

impl Default for ListQuery {
    fn default() -> Self {
        Self {
            page: 1,
            q: String::new(),
            email: String::new(),
            place: String::new(),
            sort_by: "occurred_at",
            descending: true,
        }
    }
}

impl ListQuery {
    fn select_sort(&mut self, column: &'static str) {
        self.descending = if self.sort_by == column {
            !self.descending
        } else {
            true
        };
        self.sort_by = column;
        self.page = 1;
    }

    fn set_filter(&mut self, field: &'static str, value: String) {
        match field {
            "email" => self.email = value,
            "place" => self.place = value,
            _ => self.q = value,
        }
        self.page = 1;
    }

    fn pairs(&self) -> Vec<(&'static str, String)> {
        vec![
            ("page", self.page.to_string()),
            ("per_page", "50".into()),
            ("q", self.q.clone()),
            ("email", self.email.clone()),
            ("place", self.place.clone()),
            ("sort_by", self.sort_by.into()),
            (
                "sort_order",
                if self.descending { "desc" } else { "asc" }.into(),
            ),
        ]
    }
}

const SORT_COLUMNS: [(&str, &str); 4] = [
    ("occurred_at", labels::LOGIN_HISTORY_COL_DATETIME),
    ("email", ADMIN_LOGIN_EMAIL),
    ("result", labels::LOGIN_HISTORY_COL_RESULT),
    ("place", labels::LOGIN_HISTORY_COL_PLACE),
];

#[component]
fn LoginSortButton(
    column: &'static str,
    label: &'static str,
    query: RwSignal<ListQuery>,
) -> impl IntoView {
    view! {
        <SortColumnButton label=label
            active=Signal::derive(move || query.with(|query| query.sort_by == column))
            descending=Signal::derive(move || query.with(|query| query.descending))
            on_click=move || query.update(|query| query.select_sort(column)) />
    }
}

#[component]
pub fn AdminLoginsPage() -> impl IntoView {
    let session = use_session();
    let query = RwSignal::new(ListQuery::default());
    let list = RwSignal::new(Option::<Result<LoginList, String>>::None);
    let request_id = RwSignal::new(0u64);
    let retry = RwSignal::new(0u64);
    let open = RwSignal::new(Option::<&'static str>::None);
    let email_ref = NodeRef::<leptos::html::Button>::new();
    let place_ref = NodeRef::<leptos::html::Button>::new();
    let close = move || {
        let field = open.get_untracked();
        open.set(None);
        let reference = if field == Some("email") {
            email_ref
        } else {
            place_ref
        };
        if let Some(button) = reference.get() {
            let _ = button.focus();
        }
    };

    Effect::new(move |_| {
        let params = query.get();
        retry.get();
        let generation = session.generation.get();
        if !session.is_admin.get() || session.user.get().is_none() {
            return;
        }
        let id = request_id.get_untracked() + 1;
        request_id.set(id);
        list.set(None);
        leptos::task::spawn_local(async move {
            let pairs = params.pairs();
            let refs: Vec<_> = pairs
                .iter()
                .map(|(key, value)| (*key, value.as_str()))
                .collect();
            let response = ApiClient::read_client()
                .get_json::<LoginList>("/api/v1/admin/logins", &refs)
                .await;
            if !session.is_current(generation)
                || !session.is_admin.get_untracked()
                || request_id.try_get_untracked() != Some(id)
            {
                return;
            }
            if matches!(response, Err(ApiError::Http { status: 403, .. })) {
                session.is_admin.set(false);
                return;
            }
            let _ = list.try_set(Some(response.map_err(|error| error.user_message())));
        });
    });
    let total = move || {
        list.with(|state| {
            state
                .as_ref()
                .and_then(|result| result.as_ref().ok())
                .map(|response| response.total)
        })
    };
    let loading = move || list.with(Option::is_none);
    let last_page =
        move || total().is_none_or(|total| query.with(|query| query.page * 50 >= total));

    view! {
        <PageFrame title=ADMIN_LOGINS>
            <div class="pc-search-area mb-4" role="search" aria-label=ADMIN_LOGIN_SEARCH>
                <SearchBox>
                    <SearchTag label=ADMIN_LOGIN_EMAIL value=Memo::new(move |_| query.with(|query| query.email.clone()))
                        on_clear=move || query.update(|query| query.set_filter("email", String::new())) />
                    <SearchTag label=labels::LOGIN_HISTORY_COL_PLACE value=Memo::new(move |_| query.with(|query| query.place.clone()))
                        on_clear=move || query.update(|query| query.set_filter("place", String::new())) />
                    <SearchTextInput placeholder=labels::search_placeholder(ADMIN_LOGIN_SEARCH)
                        value=move || query.with(|query| query.q.clone())
                        on_input=move |value| query.update(|query| query.set_filter("q", value)) />
                </SearchBox>
                <AddConditionRow show_label=true>
                    <AddConditionButton label=ADMIN_LOGIN_EMAIL expanded=move || open.get() == Some("email") node_ref=email_ref
                        on_click=move || open.set(Some("email")) />
                    <AddConditionButton label=labels::LOGIN_HISTORY_COL_PLACE expanded=move || open.get() == Some("place") node_ref=place_ref
                        on_click=move || open.set(Some("place")) />
                    {move || open.get().map(|field| {
                        let draft = RwSignal::new(query.with(|query| if field == "email" { query.email.clone() } else { query.place.clone() }));
                        let label = if field == "email" { ADMIN_LOGIN_EMAIL } else { labels::LOGIN_HISTORY_COL_PLACE };
                        view! {
                            <ConditionPanel title=label on_close=close>
                                <form on:submit=move |event| {
                                    event.prevent_default();
                                    query.update(|query| query.set_filter(field, draft.get_untracked()));
                                    close();
                                }>
                                    <SearchTextInput placeholder=labels::search_placeholder(label) value=move || draft.get() on_input=move |value| draft.set(value) />
                                    <div class="condition-panel-actions">
                                        <Button variant=ButtonVariant::Primary(ButtonSize::Md) submit=true on_click=move |_| {}>{ADMIN_FILTER_APPLY}</Button>
                                    </div>
                                </form>
                            </ConditionPanel>
                        }
                    })}
                    <ConditionRowEnd>
                        <span class="receipts-search-count" role="status" aria-live="polite">{move || total().map(|total| labels::count(total as usize))}</span>
                        <SearchClearButton disabled=move || query.with(|query| query.q.is_empty() && query.email.is_empty() && query.place.is_empty())
                            on_clear=move || query.update(|query| { query.q.clear(); query.email.clear(); query.place.clear(); query.page = 1; }) />
                    </ConditionRowEnd>
                </AddConditionRow>
            </div>
            <div class="mb-3 flex flex-wrap items-center gap-2 lg:hidden" aria-label=labels::SORT_BY>
                {SORT_COLUMNS.map(|(column, label)| view! { <LoginSortButton column=column label=label query=query /> })}
            </div>
            <div aria-busy=move || loading().to_string()>
                {move || match list.get() {
                    None => view! { <Loading /> }.into_any(),
                    Some(Err(message)) => view! { <ListLoadError message=message on_retry=move || retry.update(|value| *value += 1) /> }.into_any(),
                    Some(Ok(response)) if response.data.is_empty() => view! { <EmptyState title=ADMIN_LOGIN_EMPTY.to_string() /> }.into_any(),
                    Some(Ok(response)) => view! { <LoginEntries entries=response.data query=query /> }.into_any(),
                }}
            </div>
            <div class="mt-4 flex flex-wrap items-center justify-end gap-3" aria-label=labels::PAGE_NAVIGATION>
                <Button variant=ButtonVariant::Secondary(ButtonSize::Sm) disabled=move || loading() || query.with(|query| query.page <= 1)
                    on_click=move |_| query.update(|query| query.page -= 1)>{PAGE_PREVIOUS}</Button>
                <span class="tabular-nums text-sm" aria-live="polite">{move || labels::page_number(query.with(|query| query.page))}</span>
                <Button variant=ButtonVariant::Secondary(ButtonSize::Sm) disabled=move || loading() || last_page()
                    on_click=move |_| query.update(|query| query.page += 1)>{PAGE_NEXT}</Button>
            </div>
        </PageFrame>
    }
}

#[component]
fn LoginEntries(entries: Vec<AdminLoginEntry>, query: RwSignal<ListQuery>) -> impl IntoView {
    let mobile = entries.clone();
    view! {
        <div class="table-frame hidden lg:block">
            <table class="asset-table login-history-table table-fixed" data-testid="admin-login-table">
                <thead><tr>
                    {SORT_COLUMNS.map(|(column, label)| view! {
                        <th scope="col" aria-sort=move || query.with(|query| (query.sort_by == column).then_some(if query.descending { "descending" } else { "ascending" }))>
                            <LoginSortButton column=column label=label query=query />
                        </th>
                    })}
                    <th scope="col">{labels::LOGIN_HISTORY_COL_AGENT}</th>
                </tr></thead>
                <tbody>{entries.into_iter().map(|entry| view! {
                    <tr>
                        <td class="tabular-nums">{format_datetime(&entry.occurred_at)}</td>
                        <td class="break-all">{entry.email}</td>
                        <td><ResultBadge result=entry.result /></td>
                        <td>{format_place(&entry.country, &entry.region)}</td>
                        <td>{format_agent(&entry.browser, &entry.os)}</td>
                    </tr>
                }).collect_view()}</tbody>
            </table>
        </div>
        <ul class="space-y-2 lg:hidden" data-testid="admin-login-list">
            {mobile.into_iter().map(|entry| view! {
                <li><Card variant=CardVariant::Item class="p-3">
                    <div class="flex flex-wrap items-center justify-between gap-2">
                        <span class="tabular-nums text-sm font-medium">{format_datetime(&entry.occurred_at)}</span>
                        <ResultBadge result=entry.result />
                    </div>
                    <div class="mt-2 break-all text-sm font-medium">{entry.email}</div>
                    <div class="mt-1 break-words text-sm text-text-muted">{format_place(&entry.country, &entry.region)}</div>
                    <div class="mt-1 break-words text-xs text-text-muted">{format_agent(&entry.browser, &entry.os)}</div>
                </Card></li>
            }).collect_view()}
        </ul>
    }
}

#[cfg(test)]
mod tests;
use crate::api::dto::PaginatedSearchResponse;
use crate::api::{ApiClient, ApiError};
use crate::features::settings::{ResultBadge, format_agent, format_datetime, format_place};
use crate::session::use_session;
use crate::ui::button::{Button, ButtonSize, ButtonVariant};
use crate::ui::card::{Card, CardVariant};
use crate::ui::empty_state::EmptyState;
use crate::ui::labels::{
    self, ADMIN_FILTER_APPLY, ADMIN_LOGIN_EMAIL, ADMIN_LOGIN_EMPTY, ADMIN_LOGIN_SEARCH,
    ADMIN_LOGINS, PAGE_NEXT, PAGE_PREVIOUS,
};
use crate::ui::page_frame::PageFrame;
use crate::ui::search_controls::{
    AddConditionButton, AddConditionRow, ConditionPanel, ConditionRowEnd, SearchBox,
    SearchClearButton, SearchTag, SearchTextInput, SortColumnButton,
};
use crate::ui::state::{ListLoadError, Loading};
use leptos::prelude::*;
use serde::Deserialize;

#[derive(Clone, Debug, Deserialize)]
struct AdminLoginEntry {
    occurred_at: String,
    email: String,
    result: String,
    country: Option<String>,
    region: Option<String>,
    browser: Option<String>,
    os: Option<String>,
}

type LoginList = PaginatedSearchResponse<AdminLoginEntry, ()>;
