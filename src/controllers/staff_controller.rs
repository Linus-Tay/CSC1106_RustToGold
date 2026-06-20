use crate::controllers::error_controller::render_error;
use crate::controllers::session_guard::{redirect, require_admin};
use crate::forms::{CreateStaffForm, UpdateStaffForm};
use crate::views::render;
use crate::views::templates::{AdminStaffDashboardTemplate, AdminStaffEditTemplate};
use crate::AppState;
use crate::services;
use actix_session::Session;
use actix_web::{web, HttpResponse, Result};

/// GET /admin/staff
/// Lists all staff users. Shows success/error flash messages via query params.
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
/// Shows the create staff form.
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
/// Handles creation of a new staff user.
pub async fn create_staff(
    data: web::Data<AppState>,
    session: Session,
    form: web::Form<CreateStaffForm>,
) -> Result<HttpResponse> {
    if let Err(response) = require_admin(&data, &session).await {
        return Ok(response);
    }

    match services::create_staff(&data.db, form.into_inner()).await {
        Ok(_) => Ok(redirect("/admin/staff?created=1")),
        Err(error) => render(AdminStaffEditTemplate {
            staff: None,
            error,
            has_error: true,
        }),
    }
}

/// GET /admin/staff/{id}/edit
/// Shows the edit form pre-filled with the staff member's current data.
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
/// Handles update of an existing staff member.
pub async fn update_staff(
    data: web::Data<AppState>,
    session: Session,
    path: web::Path<i64>,
    form: web::Form<UpdateStaffForm>,
) -> Result<HttpResponse> {
    if let Err(response) = require_admin(&data, &session).await {
        return Ok(response);
    }

    let user_id = path.into_inner();

    match services::update_staff(&data.db, user_id, form.into_inner()).await {
        Ok(_) => Ok(redirect("/admin/staff?updated=1")),
        Err(error) => {
            // Re-load the staff member to re-render the edit form with the error
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
/// Deletes a staff member. Uses POST since HTML forms don't support DELETE.
pub async fn delete_staff(
    data: web::Data<AppState>,
    session: Session,
    path: web::Path<i64>,
) -> Result<HttpResponse> {
    if let Err(response) = require_admin(&data, &session).await {
        return Ok(response);
    }

    match services::delete_staff(&data.db, path.into_inner()).await {
        Ok(_) => Ok(redirect("/admin/staff?deleted=1")),
        Err(message) => render_error("Could not delete staff member", message),
    }
}
