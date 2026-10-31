use crate::{
    db::{self, Bind, Db, DbError, QueryBuilder},
    errors::ApiError,
    models::common::{PaginatedSearchResponse, SearchQueryParams},
    services::domain::search_filters::{push_token_ilike_filters, tokens_from_query},
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use shared::value::UserId;
use utoipa::ToSchema;

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct AdminLoginEntry {
    pub occurred_at: DateTime<Utc>,
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

const FROM: &str = " FROM login_events le JOIN users u ON u.id = le.user_id";
const PLACE: &str = "CONCAT_WS(' ', le.country, le.region)";

fn filtered_query(select: &'static str, params: &AdminLoginQuery) -> QueryBuilder {
    let mut qb = QueryBuilder::new(select);
    qb.push(FROM).push(" WHERE TRUE");
    push_token_ilike_filters(
        &mut qb,
        &tokens_from_query(params.search.q.as_deref()),
        &["u.email", PLACE],
    );
    push_token_ilike_filters(
        &mut qb,
        &tokens_from_query(params.email.as_deref()),
        &["u.email"],
    );
    push_token_ilike_filters(
        &mut qb,
        &tokens_from_query(params.place.as_deref()),
        &[PLACE],
    );
    qb
}

fn push_order(qb: &mut QueryBuilder, params: &AdminLoginQuery) -> Result<(), ApiError> {
    let column = match params.sort_by.as_deref().unwrap_or("occurred_at") {
        "occurred_at" => "le.occurred_at",
        "email" => "u.email",
        "result" => "le.result",
        "place" => PLACE,
        _ => return Err(ApiError::Validation("sort_by の値が不正です".into())),
    };
    let direction = match params.sort_order.as_deref().unwrap_or("desc") {
        "asc" => " ASC",
        "desc" => " DESC",
        _ => return Err(ApiError::Validation("sort_order の値が不正です".into())),
    };
    qb.push(" ORDER BY ")
        .push(column)
        .push(direction)
        .push(", le.occurred_at DESC, le.id DESC");
    Ok(())
}

pub async fn record_view(pool: &Db, user_id: UserId) -> Result<(), ApiError> {
    db::query(
        "INSERT INTO admin_login_views (user_id) VALUES ($1)",
        vec![Bind::from(user_id)],
    )
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn search(
    pool: &Db,
    params: AdminLoginQuery,
) -> Result<AdminLoginListResponse, ApiError> {
    let page = params.search.page();
    let per_page = params.search.per_page();
    let offset = (page - 1)
        .checked_mul(per_page)
        .ok_or_else(|| ApiError::Validation("page の値が大きすぎます".into()))?;
    let mut data_qb = filtered_query(
        "SELECT le.occurred_at, u.email::text AS email, le.result, le.country, le.region, le.browser, le.os",
        &params,
    );
    push_order(&mut data_qb, &params)?;
    data_qb
        .push(" LIMIT ")
        .push_bind(per_page)
        .push(" OFFSET ")
        .push_bind(offset);
    let mut count_qb = filtered_query("SELECT COUNT(*)", &params);
    let (data, total) = futures_util::future::try_join(
        data_qb.build_query_as::<AdminLoginEntry>().fetch_all(pool),
        count_qb.build_query_scalar::<i64>().fetch_one(pool),
    )
    .await?;
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
            occurred_at: Utc::now(),
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
