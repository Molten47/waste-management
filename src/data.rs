use sqlx::PgPool;

use crate::models::RouteInfo;

pub async fn find_route(
    pool: &PgPool,
    street: &str,
    house: i32,
) -> Result<Option<RouteInfo>, sqlx::Error> {
    sqlx::query_as::<_, RouteInfo>(
        r#"
        SELECT t.fleet_code,
               d.full_name AS driver,
               s.full_name AS supervisor,
               di.name AS district,
               di.collection_day,
               r.street, r.lane, r.first_house, r.last_house,
               t.shift_start, t.shift_end
        FROM routes r
        JOIN trucks t     ON t.id = r.truck_id
        JOIN districts di ON di.id = r.district_id
        LEFT JOIN users d ON d.id = t.driver_id
        LEFT JOIN users s ON s.id = t.supervisor_id
        WHERE lower(r.street) = lower($1)
          AND $2 BETWEEN r.first_house AND r.last_house
        LIMIT 1
        "#,
    )
    .bind(street)
    .bind(house)
    .fetch_optional(pool)
    .await
}
