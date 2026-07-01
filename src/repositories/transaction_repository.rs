use crate::models::Transaction;
use sqlx::PgPool;
use uuid::Uuid;

pub async fn find_recent_transactions_by_customer_id(
    db: &PgPool,
    customer_id: Uuid,
    limit: i64,
) -> Result<Vec<Transaction>, sqlx::Error> {
    sqlx::query_as::<_, Transaction>(
        r#"
        SELECT t.id, t.product_id, t.transaction_type, t.amount_cents,
               t.balance_after_cents, t.description, t.created_at
        FROM transactions t
        JOIN customer_products cp ON cp.id = t.product_id
        WHERE cp.customer_id = $1
        ORDER BY t.created_at DESC, t.id DESC
        LIMIT $2
        "#,
    )
    .bind(customer_id)
    .bind(limit)
    .fetch_all(db)
    .await
}

pub async fn find_customer_cash_transactions(
    db: &PgPool,
    customer_id: Uuid,
    limit: i64,
) -> Result<Vec<Transaction>, sqlx::Error> {
    sqlx::query_as::<_, Transaction>(
        r#"
        SELECT t.id, t.product_id, t.transaction_type, t.amount_cents,
               t.balance_after_cents, t.description, t.created_at
        FROM transactions t
        JOIN customer_products cp ON cp.id = t.product_id
        WHERE cp.customer_id = $1
          AND t.transaction_type IN ('deposit', 'transfer_in', 'transfer_out', 'paynow_transfer_in', 'paynow_transfer_out')
        ORDER BY t.created_at DESC, t.id DESC
        LIMIT $2
        "#,
    )
    .bind(customer_id)
    .bind(limit)
    .fetch_all(db)
    .await
}

pub async fn find_customer_loan_transactions(
    db: &PgPool,
    customer_id: Uuid,
    limit: i64,
) -> Result<Vec<Transaction>, sqlx::Error> {
    sqlx::query_as::<_, Transaction>(
        r#"
        SELECT t.id, t.product_id, t.transaction_type, t.amount_cents,
               t.balance_after_cents, t.description, t.created_at
        FROM transactions t
        JOIN customer_products cp ON cp.id = t.product_id
        WHERE cp.customer_id = $1
          AND t.transaction_type IN ('loan_disbursement', 'loan_payment', 'home_loan_payment')
        ORDER BY t.created_at DESC, t.id DESC
        LIMIT $2
        "#,
    )
    .bind(customer_id)
    .bind(limit)
    .fetch_all(db)
    .await
}

pub async fn find_customer_fixed_deposit_transactions(
    db: &PgPool,
    customer_id: Uuid,
    limit: i64,
) -> Result<Vec<Transaction>, sqlx::Error> {
    sqlx::query_as::<_, Transaction>(
        r#"
        SELECT t.id, t.product_id, t.transaction_type, t.amount_cents,
               t.balance_after_cents, t.description, t.created_at
        FROM transactions t
        JOIN customer_products cp ON cp.id = t.product_id
        WHERE cp.customer_id = $1
          AND t.transaction_type IN ('fixed_deposit_open', 'fixed_deposit_withdrawal', 'fixed_deposit_payout')
        ORDER BY t.created_at DESC, t.id DESC
        LIMIT $2
        "#,
    )
    .bind(customer_id)
    .bind(limit)
    .fetch_all(db)
    .await
}
