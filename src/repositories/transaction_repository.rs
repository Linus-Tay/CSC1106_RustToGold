use crate::models::Transaction;
use sqlx::PgPool;

pub async fn find_recent_transactions_by_user_id(
    db: &PgPool,
    user_id: i64,
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

pub async fn find_customer_cash_transactions_by_user_id(
    db: &PgPool,
    user_id: i64,
    limit: i64,
) -> Result<Vec<Transaction>, sqlx::Error> {
    sqlx::query_as::<_, Transaction>(
        r#"
        SELECT id, product_id, customer_id, transaction_type, amount_cents,
               balance_after_cents, description, created_at
        FROM transactions
        WHERE (user_id = $1 OR customer_id = (SELECT customer_id FROM users WHERE id = $1))
          AND transaction_type IN ('deposit', 'transfer_in', 'transfer_out')
        ORDER BY created_at DESC, id DESC
        LIMIT $2
        "#,
    )
    .bind(user_id)
    .bind(limit)
    .fetch_all(db)
    .await
}

pub async fn find_customer_loan_transactions_by_user_id(
    db: &PgPool,
    user_id: i64,
    limit: i64,
) -> Result<Vec<Transaction>, sqlx::Error> {
    sqlx::query_as::<_, Transaction>(
        r#"
        SELECT id, product_id, customer_id, transaction_type, amount_cents,
               balance_after_cents, description, created_at
        FROM transactions
        WHERE (user_id = $1 OR customer_id = (SELECT customer_id FROM users WHERE id = $1))
          AND transaction_type IN ('loan_disbursement', 'loan_payment', 'home_loan_payment')
        ORDER BY created_at DESC, id DESC
        LIMIT $2
        "#,
    )
    .bind(user_id)
    .bind(limit)
    .fetch_all(db)
    .await
}

pub async fn find_customer_fixed_deposit_transactions_by_user_id(
    db: &PgPool,
    user_id: i64,
    limit: i64,
) -> Result<Vec<Transaction>, sqlx::Error> {
    sqlx::query_as::<_, Transaction>(
        r#"
        SELECT id, product_id, customer_id, transaction_type, amount_cents,
               balance_after_cents, description, created_at
        FROM transactions
        WHERE (user_id = $1 OR customer_id = (SELECT customer_id FROM users WHERE id = $1))
          AND transaction_type IN ('fixed_deposit_open', 'fixed_deposit_withdrawal', 'fixed_deposit_payout')
        ORDER BY created_at DESC, id DESC
        LIMIT $2
        "#,
    )
    .bind(user_id)
    .bind(limit)
    .fetch_all(db)
    .await
}
