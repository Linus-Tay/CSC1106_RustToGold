// Service layer: keeps banking validation and workflow rules away from templates and SQL.

use crate::forms::MonitoringStatusForm;
use crate::models::HighValueAlertRecord;
use crate::repositories::{admin_repository, monitoring_repository};
use sqlx::PgPool;
use uuid::Uuid;

// Data carrier for the HighValueMonitoringDashboard workflow.
pub struct HighValueMonitoringDashboard {
    pub alerts: Vec<HighValueAlertRecord>,
    pub blocked_count: i64,
    pub flagged_count: i64,
    pub cleared_count: i64,
}

/// Loads only real high-value records for the admin monitoring queue.
pub async fn load_high_value_monitoring_dashboard(
    db: &PgPool,
) -> Result<HighValueMonitoringDashboard, String> {
    let alerts = monitoring_repository::list_high_value_alerts(db)
        .await
        .map_err(|error| {
            eprintln!("high-value monitoring list failed: {error:?}");
            "Could not load high-value monitoring records.".to_string()
        })?;

    let blocked_count = alerts.iter().filter(|alert| alert.status == "blocked").count() as i64;
    let flagged_count = alerts
        .iter()
        .filter(|alert| alert.status == "flagged" || alert.status == "reviewed")
        .count() as i64;
    let cleared_count = alerts.iter().filter(|alert| alert.status == "cleared").count() as i64;

    Ok(HighValueMonitoringDashboard {
        alerts,
        blocked_count,
        flagged_count,
        cleared_count,
    })
}

/// Clears a monitoring record only after the staff user adds a review note.
pub async fn update_high_value_alert_status(
    db: &PgPool,
    actor_user_id: Uuid,
    alert_id: Uuid,
    form: MonitoringStatusForm,
) -> Result<(), String> {
    if form.status.trim() != "cleared" {
        return Err("Only clearing is supported from this monitoring queue.".to_string());
    }

    let notes = form
        .review_notes
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .ok_or_else(|| "Add a short review note before clearing this record.".to_string())?;

    if notes.len() < 8 {
        return Err("Review note is too short. Add what was checked before clearing.".to_string());
    }

    if notes.len() > 500 {
        return Err("Review note must be 500 characters or fewer.".to_string());
    }

    let Some(alert) = monitoring_repository::clear_alert(db, actor_user_id, alert_id, notes)
        .await
        .map_err(|error| {
            eprintln!("high-value alert clear failed: {error:?}");
            "Could not clear the monitoring record.".to_string()
        })? else {
        return Err("Monitoring record not found.".to_string());
    };

    let _ = admin_repository::record_audit_log(
        db,
        Some(actor_user_id),
        "clear_high_value_monitoring_record",
        "fraud_alert",
        Some(alert.id.to_string()),
        Some(format!(
            "Cleared {} alert for {} with note: {}",
            alert.rule_display(),
            alert.customer_name,
            notes
        )),
    )
    .await;

    Ok(())
}
