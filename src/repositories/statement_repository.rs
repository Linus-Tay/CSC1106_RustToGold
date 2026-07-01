use crate::models::StatementTransaction;
use chrono::NaiveDateTime;
use sqlx::PgPool;
use uuid::Uuid;

pub async fn find_transactions_for_product_in_range(
    db: &PgPool,
    product_id: Uuid,
    start_at: NaiveDateTime,
    end_at: NaiveDateTime,
) -> Result<Vec<StatementTransaction>, sqlx::Error> {
    sqlx::query_as::<_, StatementTransaction>(
        r#"
        SELECT id, transaction_type, amount_cents, balance_after_cents, description, created_at
        FROM transactions
        WHERE product_id = $1
          AND created_at >= $2
          AND created_at < $3
        ORDER BY created_at ASC, id ASC
        "#,
    )
    .bind(product_id)
    .bind(start_at)
    .bind(end_at)
    .fetch_all(db)
    .await
}

pub async fn find_latest_balance_before(
    db: &PgPool,
    product_id: Uuid,
    before_at: NaiveDateTime,
) -> Result<Option<i64>, sqlx::Error> {
    sqlx::query_scalar::<_, i64>(
        r#"
        SELECT balance_after_cents
        FROM transactions
        WHERE product_id = $1
          AND created_at < $2
        ORDER BY created_at DESC, id DESC
        LIMIT 1
        "#,
    )
    .bind(product_id)
    .bind(before_at)
    .fetch_optional(db)
    .await
}
