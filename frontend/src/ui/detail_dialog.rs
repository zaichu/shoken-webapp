use crate::ui::amount::{Amount, AmountKind};
use crate::ui::badge::AccountBadge;
use crate::ui::button::{Button, ButtonSize, ButtonVariant};
use crate::ui::dialog::DialogShell;
use crate::ui::labels::DIALOG_CLOSE;
use leptos::prelude::*;

#[derive(Clone, Debug, PartialEq)]
pub struct DetailCell {
    pub label: String,
    pub value: String,
    pub sub: Option<String>,
    pub negative: bool,
    pub positive: bool,
    pub bold: bool,
    pub delta: bool,
}

impl DetailCell {
    pub fn plain(label: String, value: String) -> Self {
        Self {
            label,
            value,
            sub: None,
            negative: false,
            positive: false,
            bold: false,
            delta: false,
        }
    }

    pub fn delta(
        label: String,
        value: String,
        sub: Option<String>,
        negative: bool,
        positive: bool,
    ) -> Self {
        Self {
            label,
            value,
            sub,
            negative,
            positive,
            bold: false,
            delta: true,
        }
    }

    pub fn strong(
        label: String,
        value: String,
        negative: bool,
        positive: bool,
        delta: bool,
    ) -> Self {
        Self {
            label,
            value,
            sub: None,
            negative,
            positive,
            bold: true,
            delta,
        }
    }
}

fn detail_value(cell: &DetailCell) -> AnyView {
    let kind = if cell.delta {
        AmountKind::Delta
    } else {
        AmountKind::Plain
    };
    let class = if cell.bold { "font-bold" } else { "" };
    view! {
        <Amount
            kind=kind
            text=cell.value.clone()
            negative=cell.negative
            positive=cell.positive
            class=class.to_string()
        />
    }
    .into_any()
}

fn detail_sub(cell: &DetailCell) -> Option<AnyView> {
    cell.sub.clone().map(|sub| {
        view! {
            <span class="detail-dialog-sub">
                <Amount
                    kind=AmountKind::Delta
                    text=sub
                    negative=cell.negative
                    positive=cell.positive
                />
            </span>
        }
        .into_any()
    })
}

fn detail_column(cells: Vec<DetailCell>) -> AnyView {
    view! {
        <dl class="detail-dialog-col">
            {cells
                .into_iter()
                .map(|cell| {
                    let label = cell.label.clone();
                    let value = detail_value(&cell);
                    let sub = detail_sub(&cell);
                    view! {
                        <div class="detail-dialog-row">
                            <dt class="detail-dialog-label">{label}</dt>
                            <dd class="detail-dialog-value">
                                {value}
                                {sub}
                            </dd>
                        </div>
                    }
                })
                .collect_view()}
        </dl>
    }
    .into_any()
}

#[component]
pub fn DetailDialog(
    #[prop(into)] name: String,
    #[prop(into, optional)] code: String,
    #[prop(into, optional)] account: String,
    left: Vec<DetailCell>,
    right: Vec<DetailCell>,
    on_close: impl Fn() + Clone + Send + 'static,
    #[prop(optional)] testid: Option<&'static str>,
) -> impl IntoView {
    let panel_testid = testid.unwrap_or("detail-dialog");
    let dialog_title = name.clone();
    let head_name = name.clone();
    let footer_close = on_close.clone();
    let code_text = code.clone();
    let account_text = account.clone();
    let code_view = (!code_text.trim().is_empty()).then(|| code_text.clone());
    let account_view = (!account_text.trim().is_empty()).then_some(account_text.clone());
    view! {
        <DialogShell title=dialog_title on_close=on_close testid=panel_testid>
            <div class="detail-dialog-head">
                <h2 class="detail-dialog-title">{head_name}</h2>
                <p class="detail-dialog-badges">
                    {code_view
                        .map(|code| {
                            view! { <span class="detail-dialog-code">{code}</span> }
                        })}
                    {account_view
                        .map(|account| {
                            view! { <AccountBadge label=account /> }
                        })}
                </p>
            </div>
            <div class="detail-dialog-body">
                <div class="detail-dialog-grid">
                    {detail_column(left)}
                    {detail_column(right)}
                </div>
            </div>
            <div class="detail-dialog-footer">
                <Button
                    variant=ButtonVariant::Secondary(ButtonSize::Md)
                    testid="detail-dialog-close"
                    on_click=move |_| footer_close()
                >
                    {DIALOG_CLOSE}
                </Button>
            </div>
        </DialogShell>
    }
}

#[cfg(test)]
mod tests {
    use super::DetailCell;

    #[test]
    fn plain_cell_has_no_delta_or_bold() {
        let cell = DetailCell::plain("口座".to_string(), "特定".to_string());
        assert!(!cell.delta);
        assert!(!cell.bold);
        assert!(cell.sub.is_none());
    }

    #[test]
    fn delta_cell_keeps_sub_text() {
        let cell = DetailCell::delta(
            "評価額".to_string(),
            "¥260,000".to_string(),
            Some("+¥10,000".to_string()),
            false,
            true,
        );
        assert!(cell.delta);
        assert_eq!(cell.sub.as_deref(), Some("+¥10,000"));
        assert!(cell.positive);
    }

    #[test]
    fn strong_cell_keeps_bold() {
        let cell = DetailCell::strong(
            "税引後".to_string(),
            "¥1,594".to_string(),
            false,
            false,
            false,
        );
        assert!(cell.bold);
        assert!(!cell.delta);
    }
}
