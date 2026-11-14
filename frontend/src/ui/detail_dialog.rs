use crate::ui::amount::{Amount, AmountKind};
use crate::ui::badge::AccountBadge;
use crate::ui::button::{Button, ButtonSize, ButtonVariant};
use crate::ui::dialog::DialogShell;
use crate::ui::labels::DIALOG_CLOSE;
use crate::ui::security_link::{SecurityCodeLink, SecurityLinkVariant};
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
    pub const fn plain(label: String, value: String) -> Self {
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

    pub const fn with_sub(
        label: String,
        value: String,
        sub: String,
        negative: bool,
        positive: bool,
    ) -> Self {
        Self {
            label,
            value,
            sub: Some(sub),
            negative,
            positive,
            bold: false,
            delta: false,
        }
    }

    pub const fn strong_sub(
        label: String,
        value: String,
        sub: String,
        negative: bool,
        positive: bool,
    ) -> Self {
        Self {
            label,
            value,
            sub: Some(sub),
            negative,
            positive,
            bold: true,
            delta: false,
        }
    }

    pub const fn strong(
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
    /// 保有銘柄の行だけ銘柄画面へのリンクを出す。投資信託の行は空文字
    #[prop(into, optional)]
    link_code: String,
) -> impl IntoView {
    let panel_testid = testid.unwrap_or("detail-dialog");
    let dialog_title = name.clone();
    let head_name = name.clone();
    let footer_close = on_close.clone();
    let code_text = code.clone();
    let account_text = account.clone();
    let code_view = (!code_text.trim().is_empty()).then(|| code_text.clone());
    let account_view = (!account_text.trim().is_empty()).then_some(account_text.clone());
    let link_view = (!link_code.trim().is_empty()).then(|| link_code.clone());
    view! {
        <DialogShell title=dialog_title on_close=on_close testid=panel_testid>
            <div class="detail-dialog-head">
                <h2 class="dialog-title">{head_name}</h2>
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
                {link_view
                    .map(|code| {
                        view! {
                            <SecurityCodeLink
                                value=code
                                variant=SecurityLinkVariant::Details
                            />
                        }
                    })}
                <Button
                    variant=ButtonVariant::Secondary(ButtonSize::Md)
                    class="detail-dialog-close-button".to_string()
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
    fn sub_cell_keeps_plain_value_and_colored_sub() {
        let cell = DetailCell::with_sub(
            "評価額".to_string(),
            "¥260,000".to_string(),
            "+¥10,000".to_string(),
            false,
            true,
        );
        assert!(!cell.delta);
        assert!(!cell.bold);
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

    #[test]
    fn strong_sub_keeps_bold_value_and_colored_sub() {
        let cell = DetailCell::strong_sub(
            "評価額".to_string(),
            "¥200,000".to_string(),
            "-¥50,000 -20.0%".to_string(),
            true,
            false,
        );
        assert!(cell.bold);
        assert!(!cell.delta);
        assert!(cell.negative);
    }
}
