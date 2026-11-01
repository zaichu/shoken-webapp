use crate::ui::amount::{Amount, AmountKind};
use crate::ui::bottom_sheet::BottomSheet;
use crate::ui::stat_row::StatRow;
use leptos::prelude::*;

#[derive(Clone)]
pub struct SheetItem {
    pub label: String,
    pub text: String,
    pub negative: bool,
    pub positive: bool,
    pub is_profit: bool,
    pub plain: bool,
}

impl SheetItem {
    pub fn amount(
        label: String,
        text: String,
        negative: bool,
        positive: bool,
        is_profit: bool,
    ) -> Self {
        Self {
            label,
            text,
            negative,
            positive,
            is_profit,
            plain: false,
        }
    }

    pub fn plain(label: String, text: String) -> Self {
        Self {
            label,
            text,
            negative: false,
            positive: false,
            is_profit: false,
            plain: true,
        }
    }
}

#[component]
pub fn RowDetailSheet(
    #[prop(into)] title: String,
    #[prop(into)] subtitle: String,
    #[prop(into)] hero_label: String,
    #[prop(into)] hero_text: String,
    #[prop(into, optional)] hero_negative: Signal<bool>,
    #[prop(into, optional)] hero_positive: Signal<bool>,
    #[prop(optional)] hero_is_profit: bool,
    details: Vec<SheetItem>,
    on_close: impl Fn() + Clone + 'static,
    #[prop(optional)] testid: Option<&'static str>,
) -> impl IntoView {
    let sheet_testid = testid.unwrap_or("row-detail-sheet");
    let hero_kind = if hero_is_profit {
        AmountKind::Delta
    } else {
        AmountKind::Plain
    };
    view! {
        <BottomSheet title=title on_close=on_close testid=sheet_testid>
            <p class="receipt-detail-sub" data-testid="row-detail-sub">{subtitle}</p>
            <div class="receipt-detail-hero">
                <p class="receipt-detail-hero-label">{hero_label}</p>
                <p class="receipt-detail-hero-value">
                    <Amount
                        kind=hero_kind
                        text=hero_text
                        negative=hero_negative
                        positive=hero_positive
                    />
                </p>
            </div>
            <dl class="receipt-detail-list" data-testid="row-detail-list">
                {details
                    .into_iter()
                    .map(|item| {
                        let body: AnyView = if item.plain {
                            view! { <span>{item.text}</span> }.into_any()
                        } else {
                            let kind = if item.is_profit {
                                AmountKind::Delta
                            } else {
                                AmountKind::Plain
                            };
                            view! {
                                <Amount
                                    kind=kind
                                    text=item.text
                                    negative=item.negative
                                    positive=item.positive
                                />
                            }
                                .into_any()
                        };
                        view! {
                            <StatRow label=item.label numeric=true>{body}</StatRow>
                        }
                    })
                    .collect_view()}
            </dl>
        </BottomSheet>
    }
}
