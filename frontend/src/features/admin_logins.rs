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
    reload: impl Fn() + Copy + 'static + Send + Sync,
) -> impl IntoView {
    view! {
        <SortColumnButton label=label
            active=Signal::derive(move || query.with(|query| query.sort_by == column))
            descending=Signal::derive(move || query.with(|query| query.descending))
            on_click=move || { query.update(|query| query.select_sort(column)); reload(); } />
    }
}

fn sort_label(sort_by: &str) -> &'static str {
    SORT_COLUMNS
        .iter()
        .find(|(column, _)| *column == sort_by)
        .map(|(_, label)| *label)
        .unwrap_or(labels::LOGIN_HISTORY_COL_DATETIME)
}

/// 並べ替え対象ごとの順序の言葉(降順・昇順の順)。日時は新しい順・古い順、
/// 文字の列はあいうえお順・逆順にし、ほかの画面と同じ考え方にそろえる
fn order_labels(sort_by: &str) -> (&'static str, &'static str) {
    if sort_by == "occurred_at" {
        (labels::SORT_NEWEST_FIRST, labels::SORT_OLDEST_FIRST)
    } else {
        (labels::SORT_ALPHA_LAST, labels::SORT_ALPHA_FIRST)
    }
}

fn sort_order_label(sort_by: &str, descending: bool) -> &'static str {
    let (descending_label, ascending_label) = order_labels(sort_by);
    if descending {
        descending_label
    } else {
        ascending_label
    }
}

/// 「もっと見る」を出せるか。初回取得が成功して一覧が出ているときだけ、
/// 読み込み済み件数が総件数より少なければ出す。絞り込み直後の読み込み中・
/// 失敗中は旧結果の件数で出さない(押すと新条件の続きを旧一覧に混ぜるため)
fn can_load_more(ready: bool, loaded: usize, total: Option<i64>) -> bool {
    ready && total.is_some_and(|total| (loaded as i64) < total)
}

/// スマホのダイアログの中身。キーワードの入力と並べ替えの選択を
/// 下書きの上で編集し、検索で一覧へ反映する。メール・場所の絞り込みは
/// キーワード(メール・場所の両方に当たる)1つにまとめ、欄は作らない
#[component]
fn AdminDialogFields(
    draft: RwSignal<ListQuery>,
    on_search: impl Fn() + Copy + 'static + Send + Sync,
) -> impl IntoView {
    let search_callback = Callback::new(move |_| on_search());
    let draft_text = move || draft.with(|query| query.q.clone());
    let set_draft_text = move |value: String| draft.update(|query| query.q = value);
    view! {
        <div class="flex flex-col gap-4">
            <section aria-label="検索">
                <p class="search-dialog-section-title">"検索"</p>
                <div class="search-dialog-field">
                    <SearchBox>
                        <SearchTextInput
                            placeholder=labels::search_placeholder(ADMIN_LOGIN_SEARCH)
                            testid="admin-login-search-input"
                            value=draft_text
                            on_input=set_draft_text
                            on_enter=search_callback
                        />
                    </SearchBox>
                    <p class="search-dialog-hint">{labels::search_placeholder(ADMIN_LOGIN_SEARCH)}</p>
                </div>
            </section>
            <section aria-label=labels::SORT_BY.to_string()>
                <p class="search-dialog-section-title">{labels::SORT_BY}</p>
                <div class="search-dialog-sort-row">
                    <div>
                        <label class="mb-1 block text-sm font-medium text-text-deep">
                            {labels::SORT_TARGET}
                            <Select
                                value=Signal::derive(move || draft.with(|query| query.sort_by.to_string()))
                                aria_label=labels::SORT_TARGET.to_string()
                                testid="admin-login-dialog-sort-target"
                                class="search-dialog-select".to_string()
                                on_change=move |next: String| {
                                    let next: &'static str = SORT_COLUMNS
                                        .iter()
                                        .find(|(column, _)| *column == next)
                                        .map(|(column, _)| *column)
                                        .unwrap_or("occurred_at");
                                    draft.update(|query| query.sort_by = next);
                                }
                            >
                                {SORT_COLUMNS
                                    .iter()
                                    .map(|(column, label)| {
                                        view! { <option value=*column>{*label}</option> }
                                    })
                                    .collect_view()}
                            </Select>
                        </label>
                    </div>
                    <div>
                        <label class="mb-1 block text-sm font-medium text-text-deep">
                            {labels::SORT_ORDER_LABEL}
                            <Select
                                value=Signal::derive(move || if draft.with(|query| query.descending) { "desc".to_string() } else { "asc".to_string() })
                                aria_label=labels::SORT_ORDER_LABEL.to_string()
                                testid="admin-login-dialog-sort-order"
                                class="search-dialog-select".to_string()
                                on_change=move |next: String| {
                                    draft.update(|query| query.descending = next != "asc");
                                }
                            >
                                {move || {
                                    let (descending_label, ascending_label) = order_labels(
                                        draft.with(|query| query.sort_by),
                                    );
                                    view! {
                                        <option value="desc">{descending_label}</option>
                                        <option value="asc">{ascending_label}</option>
                                    }
                                }}
                            </Select>
                        </label>
                    </div>
                </div>
            </section>
        </div>
    }
}

#[component]
pub fn AdminLoginsPage() -> impl IntoView {
    let session = use_session();
    let query = RwSignal::new(ListQuery::default());
    let entries = RwSignal::new(Vec::<AdminLoginEntry>::new());
    let total = RwSignal::new(Option::<i64>::None);
    let initial = RwSignal::new(Option::<Result<(), String>>::None);
    let loading_more = RwSignal::new(false);
    let more_error = RwSignal::new(Option::<String>::None);
    let request_id = RwSignal::new(0u64);
    let panel_open = RwSignal::new(Option::<&'static str>::None);
    let dialog_open = RwSignal::new(false);
    let email_ref = NodeRef::<leptos::html::Button>::new();
    let place_ref = NodeRef::<leptos::html::Button>::new();
    let close_panel = move || {
        let field = panel_open.get_untracked();
        panel_open.set(None);
        let reference = if field == Some("email") {
            email_ref
        } else {
            place_ref
        };
        if let Some(button) = reference.get() {
            let _ = button.focus();
        }
    };

    let fetch_page = move |params: ListQuery, is_first: bool| {
        let generation = session.generation.get_untracked();
        if !session.is_admin.get_untracked() || session.user.get_untracked().is_none() {
            return;
        }
        let id = request_id.get_untracked() + 1;
        request_id.set(id);
        if is_first {
            initial.set(None);
            loading_more.set(false);
        } else {
            loading_more.set(true);
        }
        more_error.set(None);
        let loaded_page = params.page;
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
            match response {
                Ok(response) => {
                    total.set(Some(response.total));
                    if is_first {
                        entries.set(response.data);
                        initial.set(Some(Ok(())));
                    } else {
                        query.update(|query| query.page = loaded_page);
                        entries.update(|all| all.extend(response.data));
                    }
                }
                Err(error) => {
                    if is_first {
                        initial.set(Some(Err(error.user_message())));
                    } else {
                        more_error.set(Some(error.user_message()));
                    }
                }
            }
            if !is_first {
                loading_more.set(false);
            }
        });
    };
    let load_first = move || {
        query.update(|query| query.page = 1);
        fetch_page(query.get_untracked(), true);
    };
    let load_more = move || {
        if loading_more.get_untracked() {
            return;
        }
        let mut params = query.get_untracked();
        params.page += 1;
        fetch_page(params, false);
    };
    // 管理者と利用者が確定したら初回を取得する。以後の絞り込み・並べ替えは
    // 各操作から load_first を呼ぶ(Effect で query を追わない)
    Effect::new(move |_| {
        if !session.is_admin.get() || session.user.get().is_none() {
            return;
        }
        let _ = session.generation.get();
        load_first();
    });
    let has_more = move || {
        let ready = initial.with(|state| matches!(state, Some(Ok(()))));
        can_load_more(ready, entries.with(Vec::len), total.get_untracked())
    };
    let email_tag = Memo::new(move |_| query.with(|query| query.email.clone()));
    let place_tag = Memo::new(move |_| query.with(|query| query.place.clone()));
    let search_count_total = move || total.get().unwrap_or(0).max(0) as usize;
    let sort_summary = move || {
        format!(
            "{} {}",
            sort_label(query.with(|query| query.sort_by)),
            sort_order_label(
                query.with(|query| query.sort_by),
                query.with(|query| query.descending)
            ),
        )
    };
    let sort_summary = Signal::derive(sort_summary);

    view! {
        <PageFrame title=ADMIN_LOGINS>
            <div class="hidden lg:block">
                <div class="pc-search-area mb-4" role="search" aria-label=ADMIN_LOGIN_SEARCH>
                    <SearchBox>
                        <SearchTag label=ADMIN_LOGIN_EMAIL value=Memo::new(move |_| query.with(|query| query.email.clone()))
                            on_clear=move || { query.update(|query| query.set_filter("email", String::new())); load_first(); } />
                        <SearchTag label=labels::LOGIN_HISTORY_COL_PLACE value=Memo::new(move |_| query.with(|query| query.place.clone()))
                            on_clear=move || { query.update(|query| query.set_filter("place", String::new())); load_first(); } />
                        <SearchTextInput placeholder=labels::search_placeholder(ADMIN_LOGIN_SEARCH)
                            value=move || query.with(|query| query.q.clone())
                            on_input=move |value| { query.update(|query| query.set_filter("q", value)); load_first(); } />
                    </SearchBox>
                    <AddConditionRow show_label=true>
                        <AddConditionButton label=ADMIN_LOGIN_EMAIL expanded=move || panel_open.get() == Some("email") node_ref=email_ref
                            on_click=move || panel_open.set(Some("email")) />
                        <AddConditionButton label=labels::LOGIN_HISTORY_COL_PLACE expanded=move || panel_open.get() == Some("place") node_ref=place_ref
                            on_click=move || panel_open.set(Some("place")) />
                        {move || panel_open.get().map(|field| {
                            let draft = RwSignal::new(query.with(|query| if field == "email" { query.email.clone() } else { query.place.clone() }));
                            let label = if field == "email" { ADMIN_LOGIN_EMAIL } else { labels::LOGIN_HISTORY_COL_PLACE };
                            view! {
                                <ConditionPanel title=label on_close=close_panel>
                                    <form on:submit=move |event| {
                                        event.prevent_default();
                                        query.update(|query| query.set_filter(field, draft.get_untracked()));
                                        close_panel();
                                        load_first();
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
                            <span class="receipts-search-count" role="status" aria-live="polite">{move || total.get().map(|total| labels::count(total.max(0) as usize))}</span>
                            <SearchClearButton disabled=move || query.with(|query| query.q.is_empty() && query.email.is_empty() && query.place.is_empty())
                                on_clear=move || { query.update(|query| { query.q.clear(); query.email.clear(); query.place.clear(); query.page = 1; }); load_first(); } />
                        </ConditionRowEnd>
                    </AddConditionRow>
                </div>
            </div>
            <MobileSearchEntryShell
                entry_testid="admin-login-search-entry"
                search_label=ADMIN_LOGIN_SEARCH.to_string()
                sort_summary=sort_summary
                open=dialog_open
            >
                <SearchTag label=ADMIN_LOGIN_EMAIL value=email_tag
                    on_clear=move || { query.update(|query| query.set_filter("email", String::new())); load_first(); } />
                <SearchTag label=labels::LOGIN_HISTORY_COL_PLACE value=place_tag
                    on_clear=move || { query.update(|query| query.set_filter("place", String::new())); load_first(); } />
                <SearchCount
                    filtered=move || entries.with(Vec::len)
                    total=search_count_total
                    testid="admin-login-search-count"
                />
            </MobileSearchEntryShell>
            {move || {
                dialog_open.get().then(|| {
                    let draft = RwSignal::new(query.get_untracked());
                    let do_search = move || {
                        query.set(draft.get_untracked());
                        dialog_open.set(false);
                        load_first();
                    };
                    let do_clear = move || {
                        draft.update(|query| query.q.clear());
                    };
                    view! {
                        <SearchDialog
                            title=labels::SEARCH_AND_SORT.to_string()
                            on_close=move || dialog_open.set(false)
                            on_search=do_search
                            on_clear=do_clear
                            testid="admin-login-search-sheet"
                        >
                            <AdminDialogFields draft=draft on_search=do_search />
                        </SearchDialog>
                    }
                })
            }}
            <div aria-busy=move || initial.with(|state| state.is_none()).to_string()>
                {move || match initial.get() {
                    None => view! { <Loading /> }.into_any(),
                    Some(Err(message)) => view! { <ListLoadError message=message on_retry=load_first /> }.into_any(),
                    Some(Ok(())) if entries.with(Vec::is_empty) => view! { <EmptyState title=ADMIN_LOGIN_EMPTY.to_string() /> }.into_any(),
                    Some(Ok(())) => view! { <LoginEntries entries=entries.get() query=query reload=load_first /> }.into_any(),
                }}
            </div>
            {move || {
                more_error
                    .get()
                    .map(|message| {
                        view! {
                            <div class="mt-4">
                                <ListLoadError
                                    message=message
                                    on_retry=load_more
                                />
                            </div>
                        }
                    })
            }}
            {move || {
                has_more()
                    .then(|| {
                        view! {
                            <ShowMoreButton
                                testid="admin-login-more"
                                disabled=Signal::derive(move || loading_more.get())
                                on_click=load_more
                            />
                        }
                    })
            }}
        </PageFrame>
    }
}

#[component]
fn LoginEntries(
    entries: Vec<AdminLoginEntry>,
    query: RwSignal<ListQuery>,
    reload: impl Fn() + Copy + 'static + Send + Sync,
) -> impl IntoView {
    let mobile = entries.clone();
    view! {
        <div class="table-frame hidden lg:block">
            <table class="asset-table login-history-table table-fixed" data-testid="admin-login-table">
                <thead><tr>
                    {SORT_COLUMNS.map(|(column, label)| view! {
                        <th scope="col" aria-sort=move || query.with(|query| (query.sort_by == column).then_some(if query.descending { "descending" } else { "ascending" }))>
                            <LoginSortButton column=column label=label query=query reload=reload />
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
    ADMIN_LOGINS,
};
use crate::ui::page_frame::PageFrame;
use crate::ui::search_controls::{
    AddConditionButton, AddConditionRow, ConditionPanel, ConditionRowEnd, SearchBox,
    SearchClearButton, SearchCount, SearchTag, SearchTextInput, SortColumnButton,
};
use crate::ui::search_dialog::SearchDialog;
use crate::ui::search_entry::{MobileSearchEntryShell, ShowMoreButton};
use crate::ui::select::Select;
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
