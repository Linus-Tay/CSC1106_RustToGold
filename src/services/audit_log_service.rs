use crate::forms::AuditLogFilterForm;
use crate::models::AuditLogEntry;
use crate::repositories::audit_log_repository;
use serde::Serialize;
use serde_json::Value as JsonValue;
use sqlx::PgPool;

const PAGE_SIZE: i64 = 25;

#[derive(Debug, Clone)]
pub struct AuditLogPage {
    pub entries: Vec<AuditLogEntry>,
    pub total_count: i64,
    pub page: i64,
    pub total_pages: i64,
    pub has_entries: bool,
}

pub async fn load_audit_log_page(
    db: &PgPool,
    filter: AuditLogFilterForm,
    page: i64,
) -> Result<AuditLogPage, String> {
    let page = page.max(1);
    let offset = (page - 1) * PAGE_SIZE;

    let action = none_if_blank(&filter.action);
    let status = none_if_blank(&filter.status);
    let entity_type = none_if_blank(&filter.entity_type);
    let user_id = filter.user_id.trim().parse::<i64>().ok();

    let entries = audit_log_repository::list_filtered(
        db,
        action.as_deref(),
        status.as_deref(),
        entity_type.as_deref(),
        user_id,
        PAGE_SIZE,
        offset,
    )
    .await
    .map_err(|err| {
        eprintln!("[audit_log_service] list_filtered failed: {err:?}");
        "Could not load audit log entries.".to_string()
    })?;

    let total_count = audit_log_repository::count_filtered(
        db,
        action.as_deref(),
        status.as_deref(),
        entity_type.as_deref(),
        user_id,
    )
    .await
    .map_err(|err| {
        eprintln!("[audit_log_service] count_filtered failed: {err:?}");
        "Could not count audit log entries.".to_string()
    })?;

    let total_pages = ((total_count as f64) / (PAGE_SIZE as f64)).ceil().max(1.0) as i64;
    let has_entries = !entries.is_empty();

    Ok(AuditLogPage {
        entries,
        total_count,
        page,
        total_pages,
        has_entries,
    })

    
}

pub async fn list_for_entity(
    db: &PgPool,
    entity_type: &str,
    entity_id: i64,
) -> Result<Vec<AuditLogEntry>, String> {
    audit_log_repository::list_for_entity(db, entity_type, entity_id)
        .await
        .map_err(|_| "Could not load history for this record.".to_string())
}

fn none_if_blank(value: &str) -> Option<String> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.to_string())
    }
}

// ---------------------------------------------------------------------
// Logging helper — call this from your OTHER services (staff, fixed
// deposit, accounts, etc.) after an action completes.
// ---------------------------------------------------------------------

/// Generic request-context bundle, since IP/user agent come from the HTTP
/// layer, not from the service layer itself. Build one of these in the
/// controller and pass it down, or pass `None` if you don't have it handy.
#[derive(Debug, Clone, Default)]
pub struct AuditContext {
    pub actor_user_id: Option<i64>,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
}

/// Records an audit log entry. This is "fire and forget" by design —
/// logging failures are swallowed (just eprintln'd) so that a logging
/// hiccup never causes a real business operation to fail or roll back.
/// If you need guaranteed logging (e.g. compliance-critical), call
/// audit_log_repository::write_log directly inside the same DB
/// transaction as the action instead.
pub async fn record<T: Serialize>(
    db: &PgPool,
    ctx: &AuditContext,
    action: &str,
    entity_type: &str,
    entity_id: Option<i64>,
    old_value: Option<&T>,
    new_value: Option<&T>,
    status: &str,
) {
    let old_json = old_value.and_then(|v| serde_json::to_value(v).ok());
    let new_json = new_value.and_then(|v| serde_json::to_value(v).ok());

    let result = audit_log_repository::write_log(
        db,
        ctx.actor_user_id,
        action,
        entity_type,
        entity_id,
        old_json,
        new_json,
        ctx.ip_address.clone(),
        ctx.user_agent.clone(),
        status,
    )
    .await;

    if let Err(err) = result {
        // Never let audit logging break the real operation.
        eprintln!("AUDIT LOG WRITE FAILED action={action} entity={entity_type} err={err:?}");
    }
}

/// Convenience wrapper when there's no before/after JSON to attach
/// (e.g. a simple action like delete, or freeze_account).
pub async fn record_simple(
    db: &PgPool,
    ctx: &AuditContext,
    action: &str,
    entity_type: &str,
    entity_id: Option<i64>,
    status: &str,
) {
    record::<JsonValue>(db, ctx, action, entity_type, entity_id, None, None, status).await;
}
