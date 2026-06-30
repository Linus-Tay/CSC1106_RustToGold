use crate::models::Transaction;
use sqlx::PgPool;
use uuid::Uuid;

pub async fn find_recent_transactions_by_user_id(
    db: &PgPool,
    user_id: Uuid,
    limit: i64,
) -> Result<Vec<Transaction>, sqlx::Error> {
    sqlx::query_as::<_, Transaction>(
        r#"
        SELECT id, product_id, customer_id, transaction_type, amount_cents,
               balance_after_cents, description, created_at
        FROM transactions
        WHERE user_id = $1
           OR customer_id = (SELECT customer_id FROM users WHERE id = $1)
        ORDER BY created_at DESC, id DESC
        LIMIT $2
        "#,
    )
    .bind(user_id)
    .bind(limit)
    .fetch_all(db)
    .await
}
