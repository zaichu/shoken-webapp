// FacetOption はテスト（cfg(test)）からのみ参照されるため bin クレートでは unused 警告が出る
#[allow(unused_imports)]
pub use shared::common::FacetOption;
pub use shared::common::{
    MessageResponse, PaginatedSearchResponse, SearchFacets, SessionUser, Stock,
};
pub use shared::csv_import::{CsvPreviewResponse, CsvRowError, CsvUploadResponse};
pub use shared::domain::{
    AssetBalance, AssetBalanceSummary, Dividend, DividendSummary, DomesticStock,
    DomesticStockSummary, Mutualfund,
};

// wire 形が DomesticStockSummary と同一なので、serde の実装を wasm に増やさないよう使い回す
pub type MutualfundSummary = DomesticStockSummary;

#[cfg(test)]
pub type DomesticStockListResponse =
    PaginatedSearchResponse<DomesticStock, DomesticStockSummary, SearchFacets>;
pub type DividendListResponse = PaginatedSearchResponse<Dividend, DividendSummary, SearchFacets>;
#[cfg(test)]
pub type MutualfundListResponse =
    PaginatedSearchResponse<Mutualfund, MutualfundSummary, SearchFacets>;
pub type AssetBalanceListResponse =
    PaginatedSearchResponse<AssetBalance, AssetBalanceSummary, SearchFacets>;

#[cfg(test)]
mod tests;
