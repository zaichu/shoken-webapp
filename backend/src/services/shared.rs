//! 互換レイヤー: 責務別に分割された各モジュールへの re-export。
//! 既存の `crate::services::shared::{...}` import path を壊さないために残す。
//! bin crate のため、現時点でこのパス経由の呼び出しがない旧公開APIは
//! unused_imports の対象になるが、互換維持のため意図的に re-export している。
#![allow(unused_imports)]

pub use crate::services::bulk_helpers::{
    delete_all_for_user, user_ids_for_bulk_insert, BulkTimer, DeleteTarget,
};
pub use crate::services::facets::{fetch_group_facets, fetch_security_facets, FacetOrder};
pub use crate::services::search_filters::{
    escape_like_pattern, parse_date_param, parse_year_month_range, push_date_axis_filters,
    push_token_ilike_filters, tokens_from_query, year_to_range, DateAxisFilter,
};
