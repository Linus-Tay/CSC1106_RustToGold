use crate::controllers::error_controller::render_error;
use crate::controllers::session_guard::{redirect, require_admin};
use crate::forms::{CreateStaffForm, UpdateStaffForm};
use crate::views::render;
use crate::views::templates::{AdminStaffDashboardTemplate, AdminStaffEditTemplate};
use crate::AppState;
use crate::services;
use crate::services::AuditContext;
use actix_session::Session;
use actix_web::{web, HttpRequest, HttpResponse, Result};

/// GET /admin/staff
pub async fn admin_staff_page(
    data: web::Data<AppState>,
    session: Session,
    query: web::Query<std::collections::HashMap<String, String>>,
) -> Result<HttpResponse> {
    if let Err(response) = require_admin(&data, &session).await {
        return Ok(response);
    }

    match services::list_all_staff(&data.db).await {
        Ok(staff_users) => {
            let has_staff = !staff_users.is_empty();

            let success = if query.get("created").is_some() {
                "Staff member created successfully.".to_string()
            } else if query.get("updated").is_some() {
                "Staff member updated successfully.".to_string()
            } else if query.get("deleted").is_some() {
                "Staff member deleted successfully.".to_string()
            } else {
                String::new()
            };

            let has_success = !success.is_empty();

            render(AdminStaffDashboardTemplate {
                staff_users,
                has_staff,
                success,
                has_success,
                error: String::new(),
                has_error: false,
            })
        }
        Err(message) => render_error("Staff dashboard unavailable", message),
    }
}

/// GET /admin/staff/new
pub async fn admin_staff_new_page(
    data: web::Data<AppState>,
    session: Session,
) -> Result<HttpResponse> {
    if let Err(response) = require_admin(&data, &session).await {
        return Ok(response);
    }

    render(AdminStaffEditTemplate {
        staff: None,
        error: String::new(),
        has_error: false,
    })
}

/// POST /admin/staff
pub async fn create_staff(
    data: web::Data<AppState>,
    session: Session,
    req: HttpRequest,
    form: web::Form<CreateStaffForm>,
) -> Result<HttpResponse> {
    let admin = match require_admin(&data, &session).await {
        Ok(user) => user,
        Err(response) => return Ok(response),
    };

    let ctx = build_ctx(&admin.id, &req);

    match services::create_staff(&data.db, &ctx, form.into_inner()).await {
        Ok(_) => Ok(redirect("/admin/staff?created=1")),
        Err(error) => render(AdminStaffEditTemplate {
            staff: None,
            error,
            has_error: true,
        }),
    }
}

/// GET /admin/staff/{id}/edit
pub async fn admin_staff_edit_page(
    data: web::Data<AppState>,
    session: Session,
    path: web::Path<i64>,
) -> Result<HttpResponse> {
    if let Err(response) = require_admin(&data, &session).await {
        return Ok(response);
    }

    match services::find_staff_by_id(&data.db, path.into_inner()).await {
        Ok(staff) => render(AdminStaffEditTemplate {
            staff: Some(staff),
            error: String::new(),
            has_error: false,
        }),
        Err(message) => render_error("Staff member not found", message),
    }
}

/// POST /admin/staff/{id}
pub async fn update_staff(
    data: web::Data<AppState>,
    session: Session,
    req: HttpRequest,
    path: web::Path<i64>,
    form: web::Form<UpdateStaffForm>,
) -> Result<HttpResponse> {
    let admin = match require_admin(&data, &session).await {
        Ok(user) => user,
        Err(response) => return Ok(response),
    };

    let ctx = build_ctx(&admin.id, &req);
    let user_id = path.into_inner();

    match services::update_staff(&data.db, &ctx, user_id, form.into_inner()).await {
        Ok(_) => Ok(redirect("/admin/staff?updated=1")),
        Err(error) => {
            match services::find_staff_by_id(&data.db, user_id).await {
                Ok(staff) => render(AdminStaffEditTemplate {
                    staff: Some(staff),
                    error,
                    has_error: true,
                }),
                Err(message) => render_error("Staff member not found", message),
            }
        }
    }
}

/// POST /admin/staff/{id}/delete
pub async fn delete_staff(
    data: web::Data<AppState>,
    session: Session,
    req: HttpRequest,
    path: web::Path<i64>,
) -> Result<HttpResponse> {
    let admin = match require_admin(&data, &session).await {
        Ok(user) => user,
        Err(response) => return Ok(response),
    };

    let ctx = build_ctx(&admin.id, &req);

    match services::delete_staff(&data.db, &ctx, path.into_inner()).await {
        Ok(_) => Ok(redirect("/admin/staff?deleted=1")),
        Err(message) => render_error("Could not delete staff member", message),
    }
}

// --- Helpers ---

fn build_ctx(user_id: &i64, req: &HttpRequest) -> AuditContext {
    AuditContext {
        actor_user_id: Some(*user_id),
        ip_address: req.peer_addr().map(|a| a.ip().to_string()),
        user_agent: req
            .headers()
            .get("User-Agent")
            .and_then(|v| v.to_str().ok())
            .map(String::from),
    }
}
