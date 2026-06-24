use crate::controllers::error_controller::render_error;
use crate::controllers::session_guard::require_admin;
use crate::forms::AuditLogFilterForm;
use crate::services;
use crate::views::render;
use crate::views::templates::AdminAuditLogTemplate;
use crate::AppState;
use actix_session::Session;
use actix_web::{web, HttpResponse, Result};

/// GET /admin/audit-log?action=&status=&entity_type=&user_id=&page=
pub async fn admin_audit_log_page(
    data: web::Data<AppState>,
    session: Session,
    query: web::Query<AuditLogQueryParams>,
) -> Result<HttpResponse> {
    if let Err(response) = require_admin(&data, &session).await {
        return Ok(response);
    }

    let filter = AuditLogFilterForm {
        action: query.action.clone().unwrap_or_default(),
        status: query.status.clone().unwrap_or_default(),
        entity_type: query.entity_type.clone().unwrap_or_default(),
        user_id: query.user_id.clone().unwrap_or_default(),
    };

    let page = query.page.unwrap_or(1);

    match services::load_audit_log_page(&data.db, filter.clone(), page).await {
        Ok(result) => render(AdminAuditLogTemplate {
            entries: result.entries,
            has_entries: result.has_entries,
            total_count: result.total_count,
            page: result.page,
            total_pages: result.total_pages,
            filter_action: filter.action,
            filter_status: filter.status,
            filter_entity_type: filter.entity_type,
            filter_user_id: filter.user_id,
        }),
        Err(message) => render_error("Audit log unavailable", message),
    }
}

#[derive(Debug, serde::Deserialize)]
pub struct AuditLogQueryParams {
    pub action: Option<String>,
    pub status: Option<String>,
    pub entity_type: Option<String>,
    pub user_id: Option<String>,
    pub page: Option<i64>,
}
