// FacetOption・SearchFacets はテスト（cfg(test)）からのみ参照されるため bin クレートでは unused 警告が出る
#[allow(unused_imports)]
pub use shared::common::{FacetOption, SearchFacets};
pub use shared::common::{MessageResponse, PaginatedSearchResponse, SessionUser, Stock};
pub use shared::csv_import::{CsvPreviewResponse, CsvRowError, CsvUploadResponse};
#[allow(unused_imports)]
pub use shared::csv_kind_stats::{CsvKindStat, CsvKindStatsResponse};
pub use shared::domain::{
    AssetBalance, AssetBalanceSummary, Dividend, DividendSummary, DomesticStock,
    DomesticStockSummary, FundBalance, FundBalanceSummary, FundPriceSource, Mutualfund,
};

// wire 形が DomesticStockSummary と同一なので、serde の実装を wasm に増やさないよう使い回す
pub type MutualfundSummary = DomesticStockSummary;

#[cfg(test)]
pub type DomesticStockListResponse = PaginatedSearchResponse<DomesticStock, DomesticStockSummary>;
pub type DividendListResponse = PaginatedSearchResponse<Dividend, DividendSummary>;
#[cfg(test)]
pub type MutualfundListResponse = PaginatedSearchResponse<Mutualfund, MutualfundSummary>;
pub type AssetBalanceListResponse = PaginatedSearchResponse<AssetBalance, AssetBalanceSummary>;

#[cfg(test)]
mod tests;
