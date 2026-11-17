use crate::ui::amount::{Amount, AmountKind};
use crate::ui::badge::AccountBadge;
use crate::ui::button::{Button, ButtonSize, ButtonVariant};
use crate::ui::dialog::DialogShell;
use crate::ui::labels;
use crate::ui::security_link::{SecurityCodeLink, SecurityLinkVariant};
use leptos::prelude::*;

#[derive(Clone, Debug, PartialEq)]
pub struct DetailCell {
    pub label: String,
    pub value: String,
    pub sub: Option<String>,
    pub sub_rate: Option<String>,
    pub sub_rate_negative: bool,
    pub sub_rate_positive: bool,
    pub negative: bool,
    pub positive: bool,
    pub bold: bool,
    pub kind: AmountKind,
}

impl DetailCell {
    pub const fn plain(label: String, value: String) -> Self {
        Self {
            label,
            value,
            sub: None,
            sub_rate: None,
            sub_rate_negative: false,
            sub_rate_positive: false,
            negative: false,
            positive: false,
            bold: false,
            kind: AmountKind::Plain,
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
            sub_rate: None,
            sub_rate_negative: false,
            sub_rate_positive: false,
            negative,
            positive,
            bold: false,
            kind: AmountKind::Plain,
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
            sub_rate: None,
            sub_rate_negative: false,
            sub_rate_positive: false,
            negative,
            positive,
            bold: true,
            kind: AmountKind::Plain,
        }
    }

    pub const fn strong(
        label: String,
        value: String,
        negative: bool,
        positive: bool,
        kind: AmountKind,
    ) -> Self {
        Self {
            label,
            value,
            sub: None,
            sub_rate: None,
            sub_rate_negative: false,
            sub_rate_positive: false,
            negative,
            positive,
            bold: true,
            kind,
        }
    }
}

/// 主値の明示の正負。内訳(sub)付きの行の正負は内訳の符号なので主値には渡さない。
/// 主値は表示文字列の符号だけで色を決める(負は赤・正の Plain は無色)
const fn detail_main_flags(cell: &DetailCell) -> (bool, bool) {
    if cell.sub.is_none() {
        (cell.negative, cell.positive)
    } else {
        (false, false)
    }
}

fn detail_value(cell: &DetailCell) -> AnyView {
    let kind = cell.kind;
    let class = if cell.bold { "font-bold" } else { "" };
    let (negative, positive) = detail_main_flags(cell);
    view! {
        <Amount
            kind=kind
            text=cell.value.clone()
            negative=negative
            positive=positive
            class=class.to_string()
        />
    }
    .into_any()
}

fn detail_sub(cell: &DetailCell) -> Option<AnyView> {
    cell.sub.clone().map(|sub| {
        let sub_rate = cell.sub_rate.clone();
        let sub_rate_negative = cell.sub_rate_negative;
        let sub_rate_positive = cell.sub_rate_positive;
        view! {
            <span class="detail-dialog-sub">
                <Amount
                    kind=AmountKind::Delta
                    text=sub
                    negative=cell.negative
                    positive=cell.positive
                />
                {sub_rate.map(|rate| view! {
                    " "
                    <Amount
                        kind=AmountKind::Delta
                        text=rate
                        negative=sub_rate_negative
                        positive=sub_rate_positive
                    />
                })}
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
#[expect(
    clippy::needless_pass_by_value,
    reason = "Leptos component props are owned by the view"
)]
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
                    {labels::DIALOG_CLOSE}
                </Button>
            </div>
        </DialogShell>
    }
}

#[cfg(test)]
mod tests {
    use super::DetailCell;
    use super::detail_main_flags;
    use crate::ui::amount::AmountKind;

    #[test]
    fn plain_cell_has_no_delta_or_bold() {
        let cell = DetailCell::plain("口座".to_string(), "特定".to_string());
        assert_eq!(cell.kind, AmountKind::Plain);
        assert!(!cell.bold);
        assert!(cell.sub.is_none());
    }

    #[test]
    fn sub_flags_do_not_leak_to_main_value() {
        let cell = DetailCell::with_sub(
            "現在値".to_string(),
            "¥1,000".to_string(),
            "-¥10".to_string(),
            true,
            false,
        );
        assert_eq!(detail_main_flags(&cell), (false, false));
    }

    #[test]
    fn plain_cell_without_sub_keeps_its_flags() {
        let cell = DetailCell {
            label: "取得利回り".to_string(),
            value: "5.0%".to_string(),
            sub: None,
            sub_rate: None,
            sub_rate_negative: false,
            sub_rate_positive: false,
            negative: false,
            positive: true,
            bold: false,
            kind: AmountKind::Plain,
        };
        assert_eq!(detail_main_flags(&cell), (false, true));
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
        assert_eq!(cell.kind, AmountKind::Plain);
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
            AmountKind::Plain,
        );
        assert!(cell.bold);
        assert_eq!(cell.kind, AmountKind::Plain);
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
        assert_eq!(cell.kind, AmountKind::Plain);
        assert!(cell.negative);
    }
}
