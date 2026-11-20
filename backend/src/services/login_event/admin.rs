#[cfg(test)]
use crate::db::QueryBuilder;
use crate::db::{self, Bind, Db, DbError};
use crate::errors::ApiError;
use crate::models::common::{PaginatedSearchResponse, SearchQueryParams};
use crate::services::domain::search_filters;
use serde::{Deserialize, Serialize};
use shared::value::UserId;
use time::OffsetDateTime;
use utoipa::ToSchema;

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct AdminLoginEntry {
    #[serde(with = "time::serde::rfc3339")]
    pub occurred_at: OffsetDateTime,
    pub email: String,
    pub result: String,
    pub country: Option<String>,
    pub region: Option<String>,
    pub browser: Option<String>,
    pub os: Option<String>,
}

impl db::FromRow for AdminLoginEntry {
    fn from_row(row: &db::Row) -> Result<Self, DbError> {
        Ok(Self {
            occurred_at: row.try_get("occurred_at")?,
            email: row.try_get("email")?,
            result: row.try_get("result")?,
            country: row.try_get("country")?,
            region: row.try_get("region")?,
            browser: row.try_get("browser")?,
            os: row.try_get("os")?,
        })
    }
}

pub type AdminLoginListResponse = PaginatedSearchResponse<AdminLoginEntry, ()>;

#[derive(Debug, Default, Deserialize)]
pub struct AdminLoginQuery {
    #[serde(flatten)]
    pub search: SearchQueryParams,
    pub email: Option<String>,
    pub place: Option<String>,
    pub sort_by: Option<String>,
    pub sort_order: Option<String>,
}

#[cfg(test)]
const FROM: &str = " FROM login_events le JOIN users u ON u.id = le.user_id";
const PLACE: &str = "CONCAT_WS(' ', le.country, le.region)";

#[cfg(test)]
fn push_filters(qb: &mut QueryBuilder, params: &AdminLoginQuery) {
    qb.push(FROM).push(" WHERE TRUE");
    search_filters::push_token_ilike_filters(
        qb,
        &search_filters::tokens_from_query(params.search.q.as_deref()),
        &["u.email", PLACE],
    );
    search_filters::push_token_ilike_filters(
        qb,
        &search_filters::tokens_from_query(params.email.as_deref()),
        &["u.email"],
    );
    search_filters::push_token_ilike_filters(
        qb,
        &search_filters::tokens_from_query(params.place.as_deref()),
        &[PLACE],
    );
}

#[cfg(test)]
fn filtered_query(select: &'static str, params: &AdminLoginQuery) -> QueryBuilder {
    let mut qb = QueryBuilder::new(select);
    push_filters(&mut qb, params);
    qb
}

/// 許可リスト外の sort_by は SQL インジェクションを防ぐため Validation エラーにする。
fn sort_column(sort_by: Option<&str>) -> Result<&'static str, ApiError> {
    match sort_by.unwrap_or("occurred_at") {
        "occurred_at" => Ok("le.occurred_at"),
        "email" => Ok("u.email"),
        "result" => Ok("le.result"),
        "place" => Ok(PLACE),
        _ => Err(ApiError::Validation("sort_by の値が不正です".into())),
    }
}

fn sort_direction(sort_order: Option<&str>) -> Result<&'static str, ApiError> {
    match sort_order.unwrap_or("desc") {
        "asc" => Ok(" ASC"),
        "desc" => Ok(" DESC"),
        _ => Err(ApiError::Validation("sort_order の値が不正です".into())),
    }
}

fn validate_sort(params: &AdminLoginQuery) -> Result<(), ApiError> {
    sort_column(params.sort_by.as_deref())?;
    sort_direction(params.sort_order.as_deref())?;
    Ok(())
}

pub async fn record_view(pool: &Db, user_id: UserId) -> Result<(), ApiError> {
    let mut tx = pool.begin_with_user(user_id).await?;
    db::query(
        "INSERT INTO admin_login_views (user_id) VALUES ($1)",
        vec![Bind::from(user_id)],
    )
    .execute(&mut tx)
    .await?;
    tx.commit().await?;
    Ok(())
}

fn page_info(params: &AdminLoginQuery) -> Result<(i64, i64, i64), ApiError> {
    let page = params.search.page();
    let per_page = params.search.per_page();
    let offset = (page - 1)
        .checked_mul(per_page)
        .ok_or_else(|| ApiError::Validation("page の値が大きすぎます".into()))?;
    Ok((page, per_page, offset))
}

async fn fetch_count_and_data(
    pool: &Db,
    params: &AdminLoginQuery,
    per_page: i64,
    offset: i64,
) -> Result<(Vec<AdminLoginEntry>, i64), ApiError> {
    let patterns = |query: Option<&str>| {
        search_filters::tokens_from_query(query)
            .iter()
            .map(|token| search_filters::escape_like_pattern(token))
            .collect::<Vec<_>>()
    };
    let query_patterns = patterns(params.search.q.as_deref());
    let email_patterns = patterns(params.email.as_deref());
    let place_patterns = patterns(params.place.as_deref());
    let mut tx = pool.begin().await?;
    let data = db::query(
        "SELECT * FROM public.admin_login_search_data_v2($1, $2, $3, $4, $5, $6, $7)",
        vec![
            Bind::from(query_patterns.clone()),
            Bind::from(email_patterns.clone()),
            Bind::from(place_patterns.clone()),
            Bind::from(params.sort_by.as_deref().unwrap_or("occurred_at")),
            Bind::from(params.sort_order.as_deref().unwrap_or("desc")),
            Bind::from(per_page),
            Bind::from(offset),
        ],
    )
    .fetch_all::<AdminLoginEntry, _>(&mut tx)
    .await?;
    let total = db::query_scalar::<i64>(
        "SELECT public.admin_login_search_count_v2($1, $2, $3)",
        vec![
            Bind::from(query_patterns),
            Bind::from(email_patterns),
            Bind::from(place_patterns),
        ],
    )
    .fetch_one(&mut tx)
    .await?;
    tx.commit().await?;
    Ok((data, total))
}

pub async fn search(
    pool: &Db,
    params: AdminLoginQuery,
) -> Result<AdminLoginListResponse, ApiError> {
    let (page, per_page, offset) = page_info(&params)?;
    validate_sort(&params)?;

    let (data, total) = fetch_count_and_data(pool, &params, per_page, offset).await?;

    Ok(PaginatedSearchResponse {
        data,
        total,
        page,
        per_page,
        summary: None,
        facets: None,
    })
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;

    #[tokio::test]
    #[ignore = "requires Docker to run Postgres container"]
    async fn lists_all_users_filters_sorts_paginates_and_records_viewer() {
        let (pool, _node) = crate::test_db::start_test_pool().await;
        let mut users = Vec::new();
        for (google_id, email, place, result) in [
            ("admin-a", "alpha@example.com", "Tokyo", "success"),
            ("admin-b", "beta@example.com", "Osaka", "failure"),
        ] {
            let user_id: UserId = db::query_scalar(
                "INSERT INTO users (google_id, email) VALUES ($1, $2) RETURNING id",
                vec![google_id.into(), email.into()],
            )
            .fetch_one(&pool)
            .await
            .unwrap();
            db::query(
                "INSERT INTO login_events (user_id, result, device_hash, country, region, browser, os)
                 VALUES ($1, $2, $3, 'JP', $4, 'chrome', 'linux')",
                vec![user_id.into(), result.into(), Bind::Bytes(vec![7; 32]), place.into()],
            ).execute(&pool).await.unwrap();
            users.push(user_id);
        }
        record_view(&pool, users[0]).await.unwrap();
        let audit: UserId = db::query_scalar(
            "SELECT user_id FROM admin_login_views WHERE occurred_at <= NOW()",
            vec![],
        )
        .fetch_one(&pool)
        .await
        .unwrap();
        assert_eq!(audit, users[0]);
        let columns: Vec<String> = db::query_scalar("SELECT ARRAY_AGG(column_name::text ORDER BY column_name) FROM information_schema.columns WHERE table_name = 'admin_login_views'", vec![])
            .fetch_one(&pool).await.unwrap();
        assert_eq!(columns, ["id", "occurred_at", "user_id"]);

        for (key, expected) in [
            ("email", "alpha@example.com"),
            ("result", "beta@example.com"),
            ("place", "beta@example.com"),
        ] {
            for direction in ["asc", "desc"] {
                let response = search(
                    &pool,
                    AdminLoginQuery {
                        sort_by: Some(key.into()),
                        sort_order: Some(direction.into()),
                        ..Default::default()
                    },
                )
                .await
                .unwrap();
                assert_eq!(response.total, 2);
                assert_eq!(response.data.len(), 2);
                assert_eq!(
                    response.data[if direction == "asc" { 0 } else { 1 }].email,
                    expected
                );
            }
        }
        for (q, email, place, expected) in [
            (Some("EXAMPLE tokyo"), None, None, Some("alpha@example.com")),
            (None, Some("BETA"), Some("osaka"), Some("beta@example.com")),
            (None, Some("beta"), Some("tokyo"), None),
            (Some("%"), None, None, None),
        ] {
            let response = search(
                &pool,
                AdminLoginQuery {
                    search: SearchQueryParams {
                        q: q.map(str::to_owned),
                        ..Default::default()
                    },
                    email: email.map(str::to_owned),
                    place: place.map(str::to_owned),
                    ..Default::default()
                },
            )
            .await
            .unwrap();
            assert_eq!(
                response.data.first().map(|row| row.email.as_str()),
                expected
            );
            assert_eq!(response.total, i64::from(expected.is_some()));
        }
        for (page, count) in [(1, 1), (2, 1), (3, 0)] {
            let response = search(
                &pool,
                AdminLoginQuery {
                    search: SearchQueryParams {
                        pagination: crate::models::common::PaginationParams {
                            page: Some(page),
                            per_page: Some(1),
                        },
                        ..Default::default()
                    },
                    sort_by: Some("email".into()),
                    sort_order: Some("asc".into()),
                    ..Default::default()
                },
            )
            .await
            .unwrap();
            assert_eq!(response.data.len(), count);
            assert_eq!(
                (response.total, response.page, response.per_page),
                (2, page, 1)
            );
            if page <= 2 {
                assert_eq!(
                    response.data[0].email,
                    if page == 1 {
                        "alpha@example.com"
                    } else {
                        "beta@example.com"
                    }
                );
            }
        }
    }

    #[test]
    fn response_exposes_only_login_display_fields() {
        let entry = AdminLoginEntry {
            occurred_at: OffsetDateTime::now_utc(),
            email: "login@example.com".into(),
            result: "success".into(),
            country: None,
            region: None,
            browser: None,
            os: None,
        };
        let json = serde_json::to_value(entry).unwrap();
        let mut fields: Vec<_> = json
            .as_object()
            .unwrap()
            .keys()
            .map(String::as_str)
            .collect();
        fields.sort();
        assert_eq!(
            fields,
            [
                "browser",
                "country",
                "email",
                "occurred_at",
                "os",
                "region",
                "result"
            ]
        );
    }

    #[test]
    fn sort_column_accepts_allowlist_and_rejects_others() {
        for key in ["occurred_at", "email", "result", "place"] {
            assert!(sort_column(Some(key)).is_ok(), "key={key}");
        }
        assert!(sort_column(None).is_ok());
        for invalid in ["id", "email; DROP TABLE users", "", "occurred_at DESC"] {
            assert!(
                matches!(sort_column(Some(invalid)), Err(ApiError::Validation(_))),
                "invalid={invalid}"
            );
        }
    }

    #[test]
    fn sort_direction_accepts_asc_desc_and_rejects_others() {
        assert_eq!(sort_direction(Some("asc")).unwrap(), " ASC");
        assert_eq!(sort_direction(Some("desc")).unwrap(), " DESC");
        assert_eq!(sort_direction(None).unwrap(), " DESC");
        for invalid in ["sideways", "ASC", "up", ""] {
            assert!(
                matches!(sort_direction(Some(invalid)), Err(ApiError::Validation(_))),
                "invalid={invalid}"
            );
        }
    }

    #[test]
    fn validate_sort_checks_both_fields() {
        let ok = AdminLoginQuery {
            sort_by: Some("email".into()),
            sort_order: Some("asc".into()),
            ..Default::default()
        };
        assert!(validate_sort(&ok).is_ok());

        let bad_col = AdminLoginQuery {
            sort_by: Some("unknown".into()),
            ..Default::default()
        };
        assert!(matches!(
            validate_sort(&bad_col),
            Err(ApiError::Validation(_))
        ));

        let bad_dir = AdminLoginQuery {
            sort_order: Some("sideways".into()),
            ..Default::default()
        };
        assert!(matches!(
            validate_sort(&bad_dir),
            Err(ApiError::Validation(_))
        ));
    }

    #[test]
    fn filtered_query_builds_filter_clauses_per_input() {
        let params = AdminLoginQuery {
            search: SearchQueryParams {
                q: Some("tokyo".into()),
                ..Default::default()
            },
            email: Some("beta".into()),
            place: Some("osaka".into()),
            ..Default::default()
        };
        let qb = filtered_query("SELECT 1", &params);
        let sql = qb.sql();
        // 3 つのフィルター（検索語・メール・場所）がそれぞれ AND (...) として積まれる
        assert_eq!(sql.matches(" AND (").count(), 3, "sql={sql}");
        assert!(sql.contains("u.email ILIKE"), "sql={sql}");
        assert!(
            sql.contains("CONCAT_WS(' ', le.country, le.region) ILIKE"),
            "sql={sql}"
        );
    }

    #[test]
    fn filtered_query_builds_no_filter_clauses_when_empty() {
        let params = AdminLoginQuery::default();
        let qb = filtered_query("SELECT 1", &params);
        assert_eq!(
            qb.sql(),
            "SELECT 1 FROM login_events le JOIN users u ON u.id = le.user_id WHERE TRUE"
        );
    }

    #[tokio::test]
    #[ignore = "requires Docker to run Postgres container"]
    async fn search_data_limit_is_clamped_to_200_in_database() {
        use crate::models::common::PaginationParams;
        let (pool, _node) = crate::test_db::start_test_pool().await;
        let user_id: UserId = db::query_scalar(
            "INSERT INTO users (google_id, email) VALUES ('admin-limit', 'admin-limit@example.com') RETURNING id",
            vec![],
        )
        .fetch_one(&pool)
        .await
        .unwrap();
        db::query(
            "INSERT INTO login_events (user_id, result, device_hash)
             SELECT $1, 'success', $2 FROM generate_series(1, 250)",
            vec![Bind::from(user_id), Bind::Bytes(vec![7; 32])],
        )
        .execute(&pool)
        .await
        .unwrap();
        let response = search(
            &pool,
            AdminLoginQuery {
                search: SearchQueryParams {
                    pagination: PaginationParams {
                        page: Some(1),
                        per_page: Some(1000),
                    },
                    ..Default::default()
                },
                ..Default::default()
            },
        )
        .await
        .unwrap();
        assert_eq!(response.total, 250);
        assert_eq!(response.data.len(), 200);
        assert_eq!((response.page, response.per_page), (1, 1000));
    }

    #[tokio::test]
    async fn search_rejects_invalid_sort_and_overflow_before_database() {
        let pool = db::connect_pool_lazy("postgres://user:password@localhost/test_db", 1).unwrap();
        pool.close().await;
        for (key, order) in [("email; DROP TABLE users", "desc"), ("email", "sideways")] {
            assert!(matches!(
                search(
                    &pool,
                    AdminLoginQuery {
                        sort_by: Some(key.into()),
                        sort_order: Some(order.into()),
                        ..Default::default()
                    }
                )
                .await,
                Err(ApiError::Validation(_))
            ));
        }
        let params = AdminLoginQuery {
            search: SearchQueryParams {
                pagination: crate::models::common::PaginationParams {
                    page: Some(i64::MAX),
                    per_page: Some(1000),
                },
                ..Default::default()
            },
            ..Default::default()
        };
        assert!(matches!(
            search(&pool, params).await,
            Err(ApiError::Validation(_))
        ));
    }
}
