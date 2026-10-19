//! 端末ごとの設定。ブラウザ(localStorage)に保存するのは設定値だけで、
//! 証券データ・メールアドレスなどの個人情報は保存しない。
//! キーを増やすときは docs/cookie-policy.md の記載も合わせて更新する

#[cfg(test)]
mod tests;

use crate::features::asset_balance::{SortKey, SortOrder};
use crate::features::receipts::ReceiptsTab;
use leptos::prelude::*;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use shared::format::{format_currency, format_number};

const KEY_ASSET_SORT: &str = "setting_asset_sort";
const KEY_RECEIPTS_TAB: &str = "setting_receipts_initial_tab";
const KEY_AMOUNT_UNIT: &str = "setting_amount_unit";
const KEY_IDLE_MINUTES: &str = "setting_auto_logout_minutes";
const KEY_REVIEW_PROMPT: &str = "setting_review_prompt";

// 非 wasm(単体テスト)では window が無いため常に None を返す
fn storage() -> Option<web_sys::Storage> {
    #[cfg(target_arch = "wasm32")]
    {
        crate::session::local_storage()
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        None
    }
}

fn read_setting<T>(key: &str, parse: impl FnOnce(&str) -> Option<T>, default: T) -> T {
    storage()
        .and_then(|storage| storage.get_item(key).ok().flatten())
        .and_then(|value| parse(&value))
        .unwrap_or(default)
}

fn write_setting(key: &str, value: &str) {
    if let Some(storage) = storage() {
        // 容量不足・プライベートモード等で書けなくても、設定自体は動作するので握る
        let _ = storage.set_item(key, value);
    }
}

/// 資産管理を開いたときの初期の並び順
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct AssetSort {
    pub key: SortKey,
    pub order: SortOrder,
}

impl AssetSort {
    pub const DEFAULT: Self = Self {
        key: SortKey::MarketValue,
        order: SortOrder::Desc,
    };

    /// 設定画面の選択肢。並び項目ごとに降順・昇順
    pub(crate) const ALL: [Self; 8] = [
        Self::DEFAULT,
        Self {
            key: SortKey::MarketValue,
            order: SortOrder::Asc,
        },
        Self {
            key: SortKey::ProfitLoss,
            order: SortOrder::Desc,
        },
        Self {
            key: SortKey::ProfitLoss,
            order: SortOrder::Asc,
        },
        Self {
            key: SortKey::Dividend,
            order: SortOrder::Desc,
        },
        Self {
            key: SortKey::Dividend,
            order: SortOrder::Asc,
        },
        Self {
            key: SortKey::Yield,
            order: SortOrder::Desc,
        },
        Self {
            key: SortKey::Yield,
            order: SortOrder::Asc,
        },
    ];

    pub(crate) fn label(self) -> String {
        let order = match self.order {
            SortOrder::Desc => "高い順",
            SortOrder::Asc => "低い順",
        };
        format!("{}({})", self.key.label(), order)
    }

    pub(crate) fn value(self) -> String {
        format!("{}:{}", self.key.value(), self.order.value())
    }

    pub(crate) fn parse(value: &str) -> Option<Self> {
        let (key, order) = value.split_once(':')?;
        Some(Self {
            key: SortKey::from_value(key)?,
            order: SortOrder::from_value(order)?,
        })
    }
}

/// 金額の表示単位。単価(1株配当・取得単価・売却単価・現在値など)は常に円のまま
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum AmountUnit {
    #[default]
    Yen,
    Sen,
    Man,
}

impl AmountUnit {
    pub(crate) fn label(self) -> &'static str {
        match self {
            Self::Yen => "円",
            Self::Sen => "千円",
            Self::Man => "万円",
        }
    }

    pub(crate) fn value(self) -> &'static str {
        match self {
            Self::Yen => "yen",
            Self::Sen => "sen",
            Self::Man => "man",
        }
    }

    pub(crate) fn from_value(value: &str) -> Option<Self> {
        match value {
            "yen" => Some(Self::Yen),
            "sen" => Some(Self::Sen),
            "man" => Some(Self::Man),
            _ => None,
        }
    }

    pub(crate) fn divisor(self) -> Decimal {
        match self {
            Self::Yen => Decimal::ONE,
            Self::Sen => Decimal::from(1_000),
            Self::Man => Decimal::from(10_000),
        }
    }

    pub(crate) fn divisor_f64(self) -> f64 {
        match self {
            Self::Yen => 1.0,
            Self::Sen => 1_000.0,
            Self::Man => 10_000.0,
        }
    }
}

thread_local! {
    // 金額書式はビューの深い場所(表のセル整形など)からも読む必要があるため、
    // コンポーネントツリーのコンテキストを経由せずアプリ内で共有する
    static AMOUNT_UNIT: RwSignal<AmountUnit> = RwSignal::new(AmountUnit::Yen);
}

pub(crate) fn amount_unit_signal() -> RwSignal<AmountUnit> {
    AMOUNT_UNIT.with(|unit| *unit)
}

/// 書式化のみの参照(追跡しない)。ビューでは format_amount 経由で購読される
pub(crate) fn current_amount_unit() -> AmountUnit {
    amount_unit_signal().get_untracked()
}

/// 表示単位設定を反映した金額書式。円以外は単位で割って1桁丸め、単位のラベルを付ける
pub fn format_amount(value: Decimal) -> String {
    match amount_unit_signal().get() {
        AmountUnit::Yen => format_currency(value),
        unit => format!(
            "{}{}",
            format_number(value / unit.divisor(), 1),
            unit.label()
        ),
    }
}

/// 自動ログアウトまでの無操作時間(分)の選択肢
pub(crate) const IDLE_LOGOUT_CHOICES: [u32; 3] = [15, 30, 60];
pub(crate) const DEFAULT_IDLE_LOGOUT_MINUTES: u32 = 30;

fn parse_idle_minutes(value: &str) -> Option<u32> {
    let minutes = value.parse::<u32>().ok()?;
    IDLE_LOGOUT_CHOICES.contains(&minutes).then_some(minutes)
}

/// AI 総評プロンプトの表に含める任意列(銘柄コード・銘柄名は常に含む)
#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PromptColumn {
    Shares,
    AveragePrice,
    TotalPurchase,
    CurrentPrice,
    // 保存値が将来のバージョンの列名を含んでいても読み落とすだけにする
    #[serde(other)]
    Unknown,
}

impl PromptColumn {
    pub(crate) const ALL: [Self; 4] = [
        Self::Shares,
        Self::AveragePrice,
        Self::TotalPurchase,
        Self::CurrentPrice,
    ];

    pub(crate) fn label(self) -> &'static str {
        match self {
            Self::Shares => "保有株数",
            Self::AveragePrice => "平均取得単価",
            Self::TotalPurchase => "取得額合計",
            Self::CurrentPrice => "現在値",
            Self::Unknown => "",
        }
    }
}

/// AI 総評プロンプトの書式。前置き文と、コード・名称のほかに含める列
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct ReviewPromptPref {
    #[serde(default)]
    pub preamble: String,
    #[serde(default = "default_prompt_columns")]
    pub columns: Vec<PromptColumn>,
}

fn default_prompt_columns() -> Vec<PromptColumn> {
    vec![PromptColumn::Shares, PromptColumn::AveragePrice]
}

impl Default for ReviewPromptPref {
    fn default() -> Self {
        Self {
            preamble: String::new(),
            columns: default_prompt_columns(),
        }
    }
}

impl ReviewPromptPref {
    /// ヘッダー順を常に決めるため、選択済みでも未定義列を除き ALL の順に揃える
    fn normalized_columns(columns: &[PromptColumn]) -> Vec<PromptColumn> {
        PromptColumn::ALL
            .iter()
            .copied()
            .filter(|column| columns.contains(column))
            .collect()
    }

    fn parse_json(value: &str) -> Option<Self> {
        // serde は struct を配列からも復元するため、オブジェクト形状だけを受理する
        let parsed = serde_json::from_str::<serde_json::Value>(value).ok()?;
        if !parsed.is_object() {
            return None;
        }
        serde_json::from_value::<Self>(parsed)
            .ok()
            .map(|pref| Self {
                columns: Self::normalized_columns(&pref.columns),
                ..pref
            })
    }

    fn to_json(&self) -> String {
        serde_json::to_string(self).unwrap_or_default()
    }

    pub(crate) fn has_column(&self, column: PromptColumn) -> bool {
        self.columns.contains(&column)
    }

    pub(crate) fn set_column(&mut self, column: PromptColumn, enabled: bool) {
        if enabled {
            if !self.columns.contains(&column) {
                self.columns.push(column);
            }
        } else {
            self.columns.retain(|existing| *existing != column);
        }
        self.columns = Self::normalized_columns(&self.columns);
    }
}

#[derive(Clone, Copy)]
pub struct SettingsStore {
    /// 資産管理を開いたときの初期の並び順
    pub asset_sort: RwSignal<AssetSort>,
    /// 取引明細を開いたときの初期タブ
    pub receipts_initial_tab: RwSignal<ReceiptsTab>,
    /// 金額の表示単位
    pub amount_unit: RwSignal<AmountUnit>,
    /// 無操作の自動ログアウトまでの分数
    pub idle_logout_minutes: RwSignal<u32>,
    /// AI 総評プロンプトの書式
    pub review_prompt: RwSignal<ReviewPromptPref>,
}

impl SettingsStore {
    /// 既定値だけを持つストア。コンテキスト未提供のビュー単体テスト向け
    pub(crate) fn new() -> Self {
        Self {
            asset_sort: RwSignal::new(AssetSort::DEFAULT),
            receipts_initial_tab: RwSignal::new(ReceiptsTab::Dividend),
            amount_unit: amount_unit_signal(),
            idle_logout_minutes: RwSignal::new(DEFAULT_IDLE_LOGOUT_MINUTES),
            review_prompt: RwSignal::new(ReviewPromptPref::default()),
        }
    }
}

fn persist_on_change<T: Clone + Send + Sync + 'static>(
    signal: RwSignal<T>,
    key: &'static str,
    render: impl Fn(&T) -> String + 'static,
) {
    Effect::new(move |_| {
        let value = render(&signal.get());
        write_setting(key, &value);
    });
}

fn parse_receipts_tab(value: &str) -> Option<ReceiptsTab> {
    ReceiptsTab::ALL
        .iter()
        .copied()
        .find(|tab| tab.kind_param() == value)
}

/// localStorage から読み、コンテキストに載せる。変更は即座に書き戻す
pub fn provide_settings() -> SettingsStore {
    let store = SettingsStore::new();
    store.asset_sort.set(read_setting(
        KEY_ASSET_SORT,
        AssetSort::parse,
        AssetSort::DEFAULT,
    ));
    store.receipts_initial_tab.set(read_setting(
        KEY_RECEIPTS_TAB,
        parse_receipts_tab,
        ReceiptsTab::Dividend,
    ));
    store.amount_unit.set(read_setting(
        KEY_AMOUNT_UNIT,
        AmountUnit::from_value,
        AmountUnit::Yen,
    ));
    store.idle_logout_minutes.set(read_setting(
        KEY_IDLE_MINUTES,
        parse_idle_minutes,
        DEFAULT_IDLE_LOGOUT_MINUTES,
    ));
    store.review_prompt.set(read_setting(
        KEY_REVIEW_PROMPT,
        ReviewPromptPref::parse_json,
        ReviewPromptPref::default(),
    ));

    persist_on_change(store.asset_sort, KEY_ASSET_SORT, |sort| sort.value());
    persist_on_change(store.receipts_initial_tab, KEY_RECEIPTS_TAB, |tab| {
        tab.kind_param().to_string()
    });
    persist_on_change(store.amount_unit, KEY_AMOUNT_UNIT, |unit| {
        unit.value().to_string()
    });
    persist_on_change(store.idle_logout_minutes, KEY_IDLE_MINUTES, |minutes| {
        minutes.to_string()
    });
    persist_on_change(store.review_prompt, KEY_REVIEW_PROMPT, |pref| {
        pref.to_json()
    });

    provide_context(store);
    store
}

pub fn use_settings() -> SettingsStore {
    expect_context::<SettingsStore>()
}

/// コンテキスト未提供(ビューの単体テスト)では既定値だけのストアを返す
pub(crate) fn try_use_settings() -> SettingsStore {
    use_context::<SettingsStore>().unwrap_or_else(SettingsStore::new)
}
