use crate::models::HighValueAlertRecord;
use sqlx::PgPool;
use uuid::Uuid;

const HIGH_VALUE_ALERT_SELECT: &str = r#"
    SELECT
        fa.id,
        fa.customer_id,
        c.full_name AS customer_name,
        c.email AS customer_email,
        fa.product_id,
        cp.account_number,
        cp.product_id AS product_id_code,
        fa.rule_code,
        fa.severity,
        fa.channel,
        fa.amount_cents,
        fa.message,
        fa.status,
        fa.review_notes,
        fa.reviewed_at,
        fa.created_at
    FROM fraud_alerts fa
    JOIN customers c ON c.id = fa.customer_id
    LEFT JOIN customer_products cp ON cp.id = fa.product_id
"#;

pub async fn list_high_value_alerts(
    db: &PgPool,
) -> Result<Vec<HighValueAlertRecord>, sqlx::Error> {
    let query = format!(
        r#"
        {HIGH_VALUE_ALERT_SELECT}
        WHERE fa.rule_code IN (
            'HIGH_VALUE_MONITORING',
            'HIGH_VALUE_REVIEW'
        )
        ORDER BY
            CASE fa.status WHEN 'blocked' THEN 0 WHEN 'flagged' THEN 1 WHEN 'reviewed' THEN 1 ELSE 2 END,
            fa.created_at DESC
        LIMIT 150
        "#
    );

    sqlx::query_as::<_, HighValueAlertRecord>(&query)
        .fetch_all(db)
        .await
}

pub async fn clear_alert(
    db: &PgPool,
    actor_user_id: Uuid,
    alert_id: Uuid,
    review_notes: &str,
) -> Result<Option<HighValueAlertRecord>, sqlx::Error> {
    let query = format!(
        r#"
        WITH updated AS (
            UPDATE fraud_alerts
            SET status = 'cleared',
                review_notes = $3,
                reviewed_by = $2,
                reviewed_at = NOW()
            WHERE id = $1
              AND rule_code IN ('HIGH_VALUE_MONITORING', 'HIGH_VALUE_REVIEW')
            RETURNING id
        )
        {HIGH_VALUE_ALERT_SELECT}
        WHERE fa.id = (SELECT id FROM updated)
        "#
    );

    sqlx::query_as::<_, HighValueAlertRecord>(&query)
        .bind(alert_id)
        .bind(actor_user_id)
        .bind(review_notes)
        .fetch_optional(db)
        .await
}
