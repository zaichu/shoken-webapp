use serde::{Deserialize, Serialize};

/// CSV種類ごとの統計情報
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
pub struct CsvKindStat {
    /// 種類識別子 (assetbalance, fundbalance, dividend, domesticstock, mutualfund)
    pub kind: String,
    /// 表示名
    pub label: String,
    /// 保存済み件数
    pub count: i64,
    /// 最終取込日時 (ISO 8601, UTC)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub latest_imported_at: Option<String>,
}

/// CSV種類統計のレスポンス
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
pub struct CsvKindStatsResponse {
    pub kinds: Vec<CsvKindStat>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn csv_kind_stat_roundtrips() {
        let stat = CsvKindStat {
            kind: "assetbalance".to_string(),
            label: "資産管理(国内株式)".to_string(),
            count: 100,
            latest_imported_at: Some("2026-10-25T12:39:00Z".to_string()),
        };
        let json = serde_json::to_value(&stat).expect("serialize");
        assert_eq!(
            json,
            serde_json::json!({
                "kind": "assetbalance",
                "label": "資産管理(国内株式)",
                "count": 100,
                "latest_imported_at": "2026-10-25T12:39:00Z"
            })
        );
        let parsed: CsvKindStat = serde_json::from_value(json).expect("deserialize");
        assert_eq!(parsed, stat);
    }

    #[test]
    fn csv_kind_stat_roundtrips_without_latest() {
        let stat = CsvKindStat {
            kind: "dividend".to_string(),
            label: "取引明細(配当金)".to_string(),
            count: 0,
            latest_imported_at: None,
        };
        let json = serde_json::to_value(&stat).expect("serialize");
        assert_eq!(
            json,
            serde_json::json!({
                "kind": "dividend",
                "label": "取引明細(配当金)",
                "count": 0
            })
        );
        let parsed: CsvKindStat = serde_json::from_value(json).expect("deserialize");
        assert_eq!(parsed, stat);
    }

    #[test]
    fn csv_kind_stats_response_roundtrips() {
        let response = CsvKindStatsResponse {
            kinds: vec![
                CsvKindStat {
                    kind: "assetbalance".to_string(),
                    label: "資産管理(国内株式)".to_string(),
                    count: 50,
                    latest_imported_at: Some("2026-10-25T12:39:00Z".to_string()),
                },
                CsvKindStat {
                    kind: "dividend".to_string(),
                    label: "取引明細(配当金)".to_string(),
                    count: 0,
                    latest_imported_at: None,
                },
            ],
        };
        let json = serde_json::to_value(&response).expect("serialize");
        let parsed: CsvKindStatsResponse = serde_json::from_value(json).expect("deserialize");
        assert_eq!(parsed, response);
    }
}