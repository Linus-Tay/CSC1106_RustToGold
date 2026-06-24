use crate::models::AuditLogEntry;
use serde_json::Value as JsonValue;
use sqlx::PgPool;

/// Core logging function. Call this from any service after an action completes
/// (success or failure). Never let a logging failure break the calling operation —
/// callers should treat this as best-effort (see the `record_audit_log` helper
/// pattern shown in the usage notes below).
#[allow(clippy::too_many_arguments)]
pub async fn write_log(
    db: &PgPool,
    user_id: Option<i64>,
    action: &str,
    entity_type: &str,
    entity_id: Option<i64>,
    old_value: Option<JsonValue>,
    new_value: Option<JsonValue>,
    ip_address: Option<String>,
    user_agent: Option<String>,
    status: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        INSERT INTO audit_log (
            user_id, action, entity_type, entity_id,
            old_value, new_value, ip_address, user_agent, status
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
        "#,
    )
    .bind(user_id)
    .bind(action)
    .bind(entity_type)
    .bind(entity_id)
    .bind(old_value)
    .bind(new_value)
    .bind(ip_address)
    .bind(user_agent)
    .bind(status)
    .execute(db)
    .await?;

    Ok(())
}

/// List recent audit log entries with actor name/email joined in, newest first.
/// Supports optional filters; pass empty string / None to skip a filter.
pub async fn list_filtered(
    db: &PgPool,
    action: Option<&str>,
    status: Option<&str>,
    entity_type: Option<&str>,
    user_id: Option<i64>,
    limit: i64,
    offset: i64,
) -> Result<Vec<AuditLogEntry>, sqlx::Error> {
    sqlx::query_as::<_, AuditLogEntry>(
        r#"
        SELECT
            al.id, al.user_id, al.action, al.entity_type, al.entity_id,
            al.old_value, al.new_value, al.ip_address, al.user_agent,
            al.status, al.created_at,
            u.full_name AS actor_name, u.email AS actor_email
        FROM audit_log al
        LEFT JOIN users u ON u.id = al.user_id
        WHERE ($1::VARCHAR IS NULL OR al.action = $1)
          AND ($2::VARCHAR IS NULL OR al.status = $2)
          AND ($3::VARCHAR IS NULL OR al.entity_type = $3)
          AND ($4::BIGINT IS NULL OR al.user_id = $4)
        ORDER BY al.created_at DESC, al.id DESC
        LIMIT $5 OFFSET $6
        "#,
    )
    .bind(action)
    .bind(status)
    .bind(entity_type)
    .bind(user_id)
    .bind(limit)
    .bind(offset)
    .fetch_all(db)
    .await
}

pub async fn count_filtered(
    db: &PgPool,
    action: Option<&str>,
    status: Option<&str>,
    entity_type: Option<&str>,
    user_id: Option<i64>,
) -> Result<i64, sqlx::Error> {
    let row: (i64,) = sqlx::query_as(
        r#"
        SELECT COUNT(*)
        FROM audit_log al
        WHERE ($1::VARCHAR IS NULL OR al.action = $1)
          AND ($2::VARCHAR IS NULL OR al.status = $2)
          AND ($3::VARCHAR IS NULL OR al.entity_type = $3)
          AND ($4::BIGINT IS NULL OR al.user_id = $4)
        "#,
    )
    .bind(action)
    .bind(status)
    .bind(entity_type)
    .bind(user_id)
    .fetch_one(db)
    .await?;

    Ok(row.0)
}

pub async fn find_by_id(db: &PgPool, log_id: i64) -> Result<Option<AuditLogEntry>, sqlx::Error> {
    sqlx::query_as::<_, AuditLogEntry>(
        r#"
        SELECT
            al.id, al.user_id, al.action, al.entity_type, al.entity_id,
            al.old_value, al.new_value, al.ip_address, al.user_agent,
            al.status, al.created_at,
            u.full_name AS actor_name, u.email AS actor_email
        FROM audit_log al
        LEFT JOIN users u ON u.id = al.user_id
        WHERE al.id = $1
        "#,
    )
    .bind(log_id)
    .fetch_optional(db)
    .await
}

pub async fn list_for_entity(
    db: &PgPool,
    entity_type: &str,
    entity_id: i64,
) -> Result<Vec<AuditLogEntry>, sqlx::Error> {
    sqlx::query_as::<_, AuditLogEntry>(
        r#"
        SELECT
            al.id, al.user_id, al.action, al.entity_type, al.entity_id,
            al.old_value, al.new_value, al.ip_address, al.user_agent,
            al.status, al.created_at,
            u.full_name AS actor_name, u.email AS actor_email
        FROM audit_log al
        LEFT JOIN users u ON u.id = al.user_id
        WHERE al.entity_type = $1 AND al.entity_id = $2
        ORDER BY al.created_at DESC, al.id DESC
        "#,
    )
    .bind(entity_type)
    .bind(entity_id)
    .fetch_all(db)
    .await
}
