use crate::models::Transaction;
use sqlx::PgPool;

pub async fn find_recent_transactions_by_user_id(
    db: &PgPool,
    user_id: i64,
    limit: i64,
) -> Result<Vec<Transaction>, sqlx::Error> {
    sqlx::query_as::<_, Transaction>(
        r#"
        SELECT id, account_id, user_id, transaction_type, amount_cents, balance_after_cents, description, created_at
        FROM transactions
        WHERE user_id = $1
        ORDER BY created_at DESC, id DESC
        LIMIT $2
        "#,
    )
    .bind(user_id)
    .bind(limit)
    .fetch_all(db)
    .await
}
