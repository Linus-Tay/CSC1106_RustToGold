// Repository layer: isolates SQLx queries so services do not depend on raw database code.

use crate::models::{FraudAlert, TransactionControl};
use sqlx::PgPool;
use uuid::Uuid;

const CONTROL_SELECT: &str = r#"
    SELECT customer_id, daily_limit_cents, pending_daily_limit_cents, limit_change_effective_at,
           money_lock_enabled, unlock_requested_at, unlock_effective_at, created_at, updated_at
    FROM transaction_controls
"#;

// Reads get or create controls data from the database.
pub async fn get_or_create_controls(
    db: &PgPool,
    customer_id: Uuid,
) -> Result<TransactionControl, sqlx::Error> {
    sqlx::query(
        r#"
        INSERT INTO transaction_controls (customer_id)
        VALUES ($1)
        ON CONFLICT (customer_id) DO NOTHING
        "#,
    )
    .bind(customer_id)
    .execute(db)
    .await?;

    let query = format!("{} WHERE customer_id = $1", CONTROL_SELECT);
    sqlx::query_as::<_, TransactionControl>(&query)
        .bind(customer_id)
        .fetch_one(db)
        .await
}

// Persists the apply ready cooldowns database change.
pub async fn apply_ready_cooldowns(
    db: &PgPool,
    customer_id: Uuid,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        UPDATE transaction_controls
        SET daily_limit_cents = pending_daily_limit_cents,
            pending_daily_limit_cents = NULL,
            updated_at = NOW()
        WHERE customer_id = $1
          AND pending_daily_limit_cents IS NOT NULL
        "#,
    )
    .bind(customer_id)
    .execute(db)
    .await?;

    sqlx::query(
        r#"
        UPDATE transaction_controls
        SET unlock_requested_at = NULL,
            unlock_effective_at = NULL,
            updated_at = NOW()
        WHERE customer_id = $1
          AND unlock_effective_at IS NOT NULL
        "#,
    )
    .bind(customer_id)
    .execute(db)
    .await?;

    sqlx::query(
        r#"
        UPDATE transaction_controls
        SET limit_change_effective_at = NULL,
            updated_at = NOW()
        WHERE customer_id = $1
          AND pending_daily_limit_cents IS NULL
          AND limit_change_effective_at IS NOT NULL
          AND limit_change_effective_at <= NOW()
        "#,
    )
    .bind(customer_id)
    .execute(db)
    .await?;

    Ok(())
}

// Persists the set daily limit immediate database change.
pub async fn set_daily_limit_immediate(
    db: &PgPool,
    customer_id: Uuid,
    daily_limit_cents: i64,
) -> Result<TransactionControl, sqlx::Error> {
    sqlx::query_as::<_, TransactionControl>(
        r#"
        UPDATE transaction_controls
        SET daily_limit_cents = $2,
            pending_daily_limit_cents = NULL,
            limit_change_effective_at = NOW() + INTERVAL '24 hours',
            updated_at = NOW()
        WHERE customer_id = $1
        RETURNING customer_id, daily_limit_cents, pending_daily_limit_cents, limit_change_effective_at,
                  money_lock_enabled, unlock_requested_at, unlock_effective_at, created_at, updated_at
        "#,
    )
    .bind(customer_id)
    .bind(daily_limit_cents)
    .fetch_one(db)
    .await
}

// Persists the set daily limit pending database change.
pub async fn set_daily_limit_pending(
    db: &PgPool,
    customer_id: Uuid,
    pending_daily_limit_cents: i64,
) -> Result<TransactionControl, sqlx::Error> {
    // Kept for older call sites: limit changes now apply first, then cooldown locks edits.
    set_daily_limit_immediate(db, customer_id, pending_daily_limit_cents).await
}

// Persists the enable money lock database change.
pub async fn enable_money_lock(
    db: &PgPool,
    customer_id: Uuid,
) -> Result<TransactionControl, sqlx::Error> {
    sqlx::query_as::<_, TransactionControl>(
        r#"
        UPDATE transaction_controls
        SET money_lock_enabled = TRUE,
            unlock_requested_at = NULL,
            unlock_effective_at = NULL,
            updated_at = NOW()
        WHERE customer_id = $1
        RETURNING customer_id, daily_limit_cents, pending_daily_limit_cents, limit_change_effective_at,
                  money_lock_enabled, unlock_requested_at, unlock_effective_at, created_at, updated_at
        "#,
    )
    .bind(customer_id)
    .fetch_one(db)
    .await
}

// Persists the request money unlock database change.
pub async fn request_money_unlock(
    db: &PgPool,
    customer_id: Uuid,
) -> Result<TransactionControl, sqlx::Error> {
    sqlx::query_as::<_, TransactionControl>(
        r#"
        UPDATE transaction_controls
        SET money_lock_enabled = FALSE,
            unlock_requested_at = NULL,
            unlock_effective_at = NULL,
            updated_at = NOW()
        WHERE customer_id = $1
        RETURNING customer_id, daily_limit_cents, pending_daily_limit_cents, limit_change_effective_at,
                  money_lock_enabled, unlock_requested_at, unlock_effective_at, created_at, updated_at
        "#,
    )
    .bind(customer_id)
    .fetch_one(db)
    .await
}

// Reads sum outgoing today data from the database.
pub async fn sum_outgoing_today(
    db: &PgPool,
    customer_id: Uuid,
) -> Result<i64, sqlx::Error> {
    sqlx::query_scalar::<_, i64>(
        r#"
        SELECT COALESCE(SUM(t.amount_cents), 0)::BIGINT
        FROM transactions t
        JOIN customer_products cp ON cp.id = t.product_id
        WHERE cp.customer_id = $1
          AND t.transaction_type IN ('transfer_out', 'paynow_transfer_out', 'giro_payment_out')
          AND t.created_at >= date_trunc('day', NOW())
        "#,
    )
    .bind(customer_id)
    .fetch_one(db)
    .await
}

// Reads count outgoing since minutes data from the database.
pub async fn count_outgoing_since_minutes(
    db: &PgPool,
    customer_id: Uuid,
    minutes: i64,
) -> Result<i64, sqlx::Error> {
    sqlx::query_scalar::<_, i64>(
        r#"
        SELECT COUNT(*)::BIGINT
        FROM transactions t
        JOIN customer_products cp ON cp.id = t.product_id
        WHERE cp.customer_id = $1
          AND t.transaction_type IN ('transfer_out', 'paynow_transfer_out', 'giro_payment_out')
          AND t.created_at >= NOW() - ($2::TEXT || ' minutes')::INTERVAL
        "#,
    )
    .bind(customer_id)
    .bind(minutes)
    .fetch_one(db)
    .await
}

// Reads sum outgoing since minutes data from the database.
pub async fn sum_outgoing_since_minutes(
    db: &PgPool,
    customer_id: Uuid,
    minutes: i64,
) -> Result<i64, sqlx::Error> {
    sqlx::query_scalar::<_, i64>(
        r#"
        SELECT COALESCE(SUM(t.amount_cents), 0)::BIGINT
        FROM transactions t
        JOIN customer_products cp ON cp.id = t.product_id
        WHERE cp.customer_id = $1
          AND t.transaction_type IN ('transfer_out', 'paynow_transfer_out', 'giro_payment_out')
          AND t.created_at >= NOW() - ($2::TEXT || ' minutes')::INTERVAL
        "#,
    )
    .bind(customer_id)
    .bind(minutes)
    .fetch_one(db)
    .await
}

// Persists the insert fraud alert database change.
pub async fn insert_fraud_alert(
    db: &PgPool,
    customer_id: Uuid,
    product_id: Option<Uuid>,
    rule_code: &str,
    severity: &str,
    channel: &str,
    amount_cents: i64,
    message: &str,
) -> Result<(), sqlx::Error> {
    insert_fraud_alert_with_status(
        db,
        customer_id,
        product_id,
        rule_code,
        severity,
        channel,
        amount_cents,
        message,
        "blocked",
    )
    .await
}

// Persists the insert fraud alert with status database change.
pub async fn insert_fraud_alert_with_status(
    db: &PgPool,
    customer_id: Uuid,
    product_id: Option<Uuid>,
    rule_code: &str,
    severity: &str,
    channel: &str,
    amount_cents: i64,
    message: &str,
    status: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        INSERT INTO fraud_alerts (customer_id, product_id, rule_code, severity, channel, amount_cents, message, status)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
        "#,
    )
    .bind(customer_id)
    .bind(product_id)
    .bind(rule_code)
    .bind(severity)
    .bind(channel)
    .bind(amount_cents)
    .bind(message)
    .bind(status)
    .execute(db)
    .await?;

    Ok(())
}

// Reads list recent alerts data from the database.
pub async fn list_recent_alerts(
    db: &PgPool,
    customer_id: Uuid,
    limit: i64,
) -> Result<Vec<FraudAlert>, sqlx::Error> {
    sqlx::query_as::<_, FraudAlert>(
        r#"
        SELECT id, customer_id, product_id, rule_code, severity, channel, amount_cents, message, status, created_at
        FROM fraud_alerts
        WHERE customer_id = $1
          AND rule_code NOT IN ('DAILY_LIMIT', 'MISSING_REFERENCE', 'MONEY_LOCK')
        ORDER BY created_at DESC
        LIMIT $2
        "#,
    )
    .bind(customer_id)
    .bind(limit)
    .fetch_all(db)
    .await
}
