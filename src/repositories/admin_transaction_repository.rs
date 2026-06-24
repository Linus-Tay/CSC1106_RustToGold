use crate::models::{BankAccount, Transaction};
use sqlx::{PgPool, Postgres, Transaction as DbTransaction};
use uuid::Uuid;

pub async fn admin_list_transactions(
    db: &PgPool,
    transaction_type: Option<&str>,
    user_id: Option<i64>,
    account_id: Option<i64>,
    limit: i64,
    offset: i64,
) -> Result<Vec<Transaction>, sqlx::Error> {
    sqlx::query_as::<_, Transaction>(
        r#"
        SELECT id, account_id, user_id, transaction_type,
               amount_cents, balance_after_cents, description, created_at
        FROM transactions
        WHERE ($1::TEXT IS NULL OR transaction_type = $1::TEXT)
          AND ($2::BIGINT IS NULL OR user_id = $2::BIGINT)
          AND ($3::BIGINT IS NULL OR account_id = $3::BIGINT)
        ORDER BY created_at DESC, id DESC
        LIMIT $4::BIGINT OFFSET $5::BIGINT
        "#,
    )
    .bind(transaction_type)
    .bind(user_id)
    .bind(account_id)
    .bind(limit)
    .bind(offset)
    .fetch_all(db)
    .await
}

pub async fn admin_count_transactions(
    db: &PgPool,
    transaction_type: Option<&str>,
    user_id: Option<i64>,
    account_id: Option<i64>,
) -> Result<i64, sqlx::Error> {
    let row: (i64,) = sqlx::query_as(
        r#"
        SELECT COUNT(*)
        FROM transactions
        WHERE ($1::TEXT IS NULL OR transaction_type = $1::TEXT)
          AND ($2::BIGINT IS NULL OR user_id = $2::BIGINT)
          AND ($3::BIGINT IS NULL OR account_id = $3::BIGINT)
        "#,
    )
    .bind(transaction_type)
    .bind(user_id)
    .bind(account_id)
    .fetch_one(db)
    .await?;

    Ok(row.0)
}