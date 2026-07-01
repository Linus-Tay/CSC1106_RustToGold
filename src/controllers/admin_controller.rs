// Controller layer: handles HTTP/session flow and delegates business rules to services.

use crate::controllers::error_controller::render_error;
use crate::controllers::session_guard::{redirect, require_admin};
use crate::forms::MonitoringStatusForm;
use crate::services;
use crate::views::{
    render, AdminAuditLogTemplate, AdminCustomerAccountsTemplate,
    AdminCustomerApplicationsTemplate, AdminDashboardTemplate, AdminHighValueMonitoringTemplate,
    AdminPersonalLoansTemplate, AdminStaffTemplate,
};
use crate::AppState;
use actix_session::Session;
use actix_web::{web, HttpResponse, Result};
use serde::Deserialize;
use uuid::Uuid;

#[derive(Debug, Deserialize)]
// Data carrier for the StaffCreateForm workflow.
pub struct StaffCreateForm {
    pub username: String,
    pub full_name: String,
    pub email: String,
    pub phone_number: String,
    pub role: String,
    pub password: String,
}

#[derive(Debug, Deserialize)]
// Data carrier for the StaffUpdateForm workflow.
pub struct StaffUpdateForm {
    pub full_name: String,
    pub email: String,
    pub phone_number: String,
    pub role: String,
    pub status: String,
    pub password: Option<String>,
}

// Handles the admin dashboard request.
pub async fn admin_dashboard(data: web::Data<AppState>, session: Session) -> Result<HttpResponse> {
    if let Err(response) = require_admin(&data, &session).await {
        return Ok(response);
    }

    match services::load_admin_dashboard(&data.db).await {
        Ok(summary) => render(AdminDashboardTemplate { summary }),
        Err(error) => render_error("Admin dashboard unavailable", error),
    }
}

// Renders the admin signups page screen with data prepared by the service layer.
pub async fn admin_signups_page(
    data: web::Data<AppState>,
    session: Session,
) -> Result<HttpResponse> {
    if let Err(response) = require_admin(&data, &session).await {
        return Ok(response);
    }

    match services::list_admin_customer_applications(&data.db).await {
        Ok(applications) => render(AdminCustomerApplicationsTemplate {
            has_applications: !applications.is_empty(),
            applications,
            error: String::new(),
            has_error: false,
        }),
        Err(error) => render(AdminCustomerApplicationsTemplate {
            applications: Vec::new(),
            has_applications: false,
            error,
            has_error: true,
        }),
    }
}

// Handles the approve customer application form action and redirects after the service result.
pub async fn approve_customer_application(
    data: web::Data<AppState>,
    session: Session,
    path: web::Path<String>,
) -> Result<HttpResponse> {
    let staff = match require_admin(&data, &session).await {
        Ok(user) => user,
        Err(response) => return Ok(response),
    };

    let customer_id = match parse_uuid(path.into_inner()) {
        Ok(value) => value,
        Err(response) => return Ok(response),
    };

    match services::approve_customer_application(&data, staff.id, customer_id).await {
        Ok(_) => Ok(redirect("/admin/signups")),
        Err(error) => render_error("Customer approval failed", error),
    }
}

// Handles the reject customer application form action and redirects after the service result.
pub async fn reject_customer_application(
    data: web::Data<AppState>,
    session: Session,
    path: web::Path<String>,
) -> Result<HttpResponse> {
    let staff = match require_admin(&data, &session).await {
        Ok(user) => user,
        Err(response) => return Ok(response),
    };

    let customer_id = match parse_uuid(path.into_inner()) {
        Ok(value) => value,
        Err(response) => return Ok(response),
    };

    match services::reject_customer_application(&data.db, staff.id, customer_id).await {
        Ok(_) => Ok(redirect("/admin/signups")),
        Err(error) => render_error("Customer rejection failed", error),
    }
}

// Renders the admin personal loans page screen with data prepared by the service layer.
pub async fn admin_personal_loans_page(
    data: web::Data<AppState>,
    session: Session,
) -> Result<HttpResponse> {
    if let Err(response) = require_admin(&data, &session).await {
        return Ok(response);
    }

    match services::list_admin_personal_loans(&data.db).await {
        Ok(loans) => render(AdminPersonalLoansTemplate {
            has_loans: !loans.is_empty(),
            loans,
            error: String::new(),
            has_error: false,
        }),
        Err(error) => render(AdminPersonalLoansTemplate {
            loans: Vec::new(),
            has_loans: false,
            error,
            has_error: true,
        }),
    }
}

// Handles the approve personal loan form action and redirects after the service result.
pub async fn approve_personal_loan(
    data: web::Data<AppState>,
    session: Session,
    path: web::Path<String>,
) -> Result<HttpResponse> {
    let staff = match require_admin(&data, &session).await {
        Ok(user) => user,
        Err(response) => return Ok(response),
    };

    let loan_id = match parse_uuid(path.into_inner()) {
        Ok(value) => value,
        Err(response) => return Ok(response),
    };

    match services::approve_personal_loan(&data.db, staff.id, loan_id).await {
        Ok(_) => Ok(redirect("/admin/personal-loans")),
        Err(error) => render_error("Personal loan approval failed", error),
    }
}

// Handles the reject personal loan form action and redirects after the service result.
pub async fn reject_personal_loan(
    data: web::Data<AppState>,
    session: Session,
    path: web::Path<String>,
) -> Result<HttpResponse> {
    let staff = match require_admin(&data, &session).await {
        Ok(user) => user,
        Err(response) => return Ok(response),
    };

    let loan_id = match parse_uuid(path.into_inner()) {
        Ok(value) => value,
        Err(response) => return Ok(response),
    };

    match services::reject_personal_loan(&data.db, staff.id, loan_id).await {
        Ok(_) => Ok(redirect("/admin/personal-loans")),
        Err(error) => render_error("Personal loan rejection failed", error),
    }
}


// Renders the admin high value monitoring page screen with data prepared by the service layer.
pub async fn admin_high_value_monitoring_page(
    data: web::Data<AppState>,
    session: Session,
) -> Result<HttpResponse> {
    if let Err(response) = require_admin(&data, &session).await {
        return Ok(response);
    }

    match services::load_high_value_monitoring_dashboard(&data.db).await {
        Ok(page) => render(AdminHighValueMonitoringTemplate {
            has_alerts: !page.alerts.is_empty(),
            alerts: page.alerts,
            blocked_count: page.blocked_count,
            flagged_count: page.flagged_count,
            cleared_count: page.cleared_count,
            error: String::new(),
            has_error: false,
        }),
        Err(error) => render(AdminHighValueMonitoringTemplate {
            alerts: Vec::new(),
            has_alerts: false,
            blocked_count: 0,
            flagged_count: 0,
            cleared_count: 0,
            error,
            has_error: true,
        }),
    }
}

// Handles the update high value alert status form action and redirects after the service result.
pub async fn update_high_value_alert_status(
    data: web::Data<AppState>,
    session: Session,
    path: web::Path<String>,
    form: web::Form<MonitoringStatusForm>,
) -> Result<HttpResponse> {
    let staff = match require_admin(&data, &session).await {
        Ok(user) => user,
        Err(response) => return Ok(response),
    };

    let alert_id = match parse_uuid(path.into_inner()) {
        Ok(value) => value,
        Err(response) => return Ok(response),
    };

    match services::update_high_value_alert_status(&data.db, staff.id, alert_id, form.into_inner()).await {
        Ok(_) => Ok(redirect("/admin/high-value-monitoring")),
        Err(error) => render_error("Monitoring update failed", error),
    }
}

// Renders the admin staff page screen with data prepared by the service layer.
pub async fn admin_staff_page(data: web::Data<AppState>, session: Session) -> Result<HttpResponse> {
    let user = match require_admin(&data, &session).await {
        Ok(user) => user,
        Err(response) => return Ok(response),
    };

    match services::list_admin_staff(&data.db).await {
        Ok(staff_users) => render(AdminStaffTemplate {
            staff_users,
            has_error: false,
            error: String::new(),
            has_success: false,
            success: String::new(),
            current_admin_id: user.id,
        }),
        Err(error) => render(AdminStaffTemplate {
            staff_users: Vec::new(),
            has_error: true,
            error,
            has_success: false,
            success: String::new(),
            current_admin_id: user.id,
        }),
    }
}

// Handles the create staff user form action and redirects after the service result.
pub async fn create_staff_user(
    data: web::Data<AppState>,
    session: Session,
    form: web::Form<StaffCreateForm>,
) -> Result<HttpResponse> {
    let user = match require_admin(&data, &session).await {
        Ok(user) => user,
        Err(response) => return Ok(response),
    };
    if !user.is_admin() {
        return Ok(redirect("/403"));
    }

    let form = form.into_inner();
    match services::create_staff_user(
        &data.db,
        user.id,
        form.username,
        form.full_name,
        form.email,
        form.phone_number,
        form.role,
        form.password,
    )
    .await
    {
        Ok(_) => Ok(redirect("/admin/staff")),
        Err(error) => render_staff_with_message(&data, user.id, error, false).await,
    }
}

// Handles the update staff user form action and redirects after the service result.
pub async fn update_staff_user(
    data: web::Data<AppState>,
    session: Session,
    path: web::Path<String>,
    form: web::Form<StaffUpdateForm>,
) -> Result<HttpResponse> {
    let user = match require_admin(&data, &session).await {
        Ok(user) => user,
        Err(response) => return Ok(response),
    };
    if !user.is_admin() {
        return Ok(redirect("/403"));
    }

    let staff_user_id = match parse_uuid(path.into_inner()) {
        Ok(value) => value,
        Err(response) => return Ok(response),
    };
    let form = form.into_inner();
    match services::update_staff_user(
        &data.db,
        user.id,
        staff_user_id,
        form.full_name,
        form.email,
        form.phone_number,
        form.role,
        form.status,
        form.password,
    )
    .await
    {
        Ok(_) => Ok(redirect("/admin/staff")),
        Err(error) => render_staff_with_message(&data, user.id, error, false).await,
    }
}

// Handles the delete staff user form action and redirects after the service result.
pub async fn delete_staff_user(
    data: web::Data<AppState>,
    session: Session,
    path: web::Path<String>,
) -> Result<HttpResponse> {
    let user = match require_admin(&data, &session).await {
        Ok(user) => user,
        Err(response) => return Ok(response),
    };
    if !user.is_admin() {
        return Ok(redirect("/403"));
    }

    let staff_user_id = match parse_uuid(path.into_inner()) {
        Ok(value) => value,
        Err(response) => return Ok(response),
    };

    match services::delete_staff_user(&data.db, user.id, staff_user_id).await {
        Ok(_) => Ok(redirect("/admin/staff")),
        Err(error) => render_staff_with_message(&data, user.id, error, false).await,
    }
}

// Renders the admin customer accounts page screen with data prepared by the service layer.
pub async fn admin_customer_accounts_page(
    data: web::Data<AppState>,
    session: Session,
) -> Result<HttpResponse> {
    if let Err(response) = require_admin(&data, &session).await {
        return Ok(response);
    }

    match services::list_admin_customer_accounts(&data.db).await {
        Ok(accounts) => render(AdminCustomerAccountsTemplate {
            has_accounts: !accounts.is_empty(),
            accounts,
            has_error: false,
            error: String::new(),
        }),
        Err(error) => render(AdminCustomerAccountsTemplate {
            has_accounts: false,
            accounts: Vec::new(),
            has_error: true,
            error,
        }),
    }
}

// Handles the suspend customer user form action and redirects after the service result.
pub async fn suspend_customer_user(
    data: web::Data<AppState>,
    session: Session,
    path: web::Path<String>,
) -> Result<HttpResponse> {
    let target_user_id = match parse_uuid(path.into_inner()) {
        Ok(value) => value,
        Err(response) => return Ok(response),
    };
    set_customer_user_status(data, session, target_user_id, "suspended").await
}

// Handles the activate customer user form action and redirects after the service result.
pub async fn activate_customer_user(
    data: web::Data<AppState>,
    session: Session,
    path: web::Path<String>,
) -> Result<HttpResponse> {
    let target_user_id = match parse_uuid(path.into_inner()) {
        Ok(value) => value,
        Err(response) => return Ok(response),
    };
    set_customer_user_status(data, session, target_user_id, "active").await
}

// Handles the set customer user status request.
async fn set_customer_user_status(
    data: web::Data<AppState>,
    session: Session,
    target_user_id: Uuid,
    status: &str,
) -> Result<HttpResponse> {
    let staff = match require_admin(&data, &session).await {
        Ok(user) => user,
        Err(response) => return Ok(response),
    };

    match services::set_customer_user_status(&data.db, staff.id, target_user_id, status).await {
        Ok(_) => Ok(redirect("/admin/accounts")),
        Err(error) => render_error("Customer user update failed", error),
    }
}

// Handles the activate customer product form action and redirects after the service result.
pub async fn activate_customer_product(
    data: web::Data<AppState>,
    session: Session,
    path: web::Path<String>,
) -> Result<HttpResponse> {
    set_customer_product_status(data, session, path.into_inner(), "active").await
}

// Handles the freeze customer product form action and redirects after the service result.
pub async fn freeze_customer_product(
    data: web::Data<AppState>,
    session: Session,
    path: web::Path<String>,
) -> Result<HttpResponse> {
    set_customer_product_status(data, session, path.into_inner(), "frozen").await
}

// Handles the close customer product form action and redirects after the service result.
pub async fn close_customer_product(
    data: web::Data<AppState>,
    session: Session,
    path: web::Path<String>,
) -> Result<HttpResponse> {
    set_customer_product_status(data, session, path.into_inner(), "closed").await
}

// Handles the set customer product status request.
async fn set_customer_product_status(
    data: web::Data<AppState>,
    session: Session,
    product_id: String,
    status: &str,
) -> Result<HttpResponse> {
    let staff = match require_admin(&data, &session).await {
        Ok(user) => user,
        Err(response) => return Ok(response),
    };

    let product_id = match parse_uuid(product_id) {
        Ok(value) => value,
        Err(response) => return Ok(response),
    };

    match services::set_customer_product_status(&data.db, staff.id, product_id, status).await {
        Ok(_) => Ok(redirect("/admin/accounts")),
        Err(error) => render_error("Customer product update failed", error),
    }
}

// Renders the admin audit log page screen with data prepared by the service layer.
pub async fn admin_audit_log_page(
    data: web::Data<AppState>,
    session: Session,
) -> Result<HttpResponse> {
    if let Err(response) = require_admin(&data, &session).await {
        return Ok(response);
    }

    match services::list_audit_logs(&data.db).await {
        Ok(logs) => render(AdminAuditLogTemplate {
            has_logs: !logs.is_empty(),
            logs,
            has_error: false,
            error: String::new(),
        }),
        Err(error) => render(AdminAuditLogTemplate {
            has_logs: false,
            logs: Vec::new(),
            has_error: true,
            error,
        }),
    }
}

// Handles the render staff with message request.
async fn render_staff_with_message(
    data: &web::Data<AppState>,
    current_admin_id: Uuid,
    message: String,
    success: bool,
) -> Result<HttpResponse> {
    let staff_users = services::list_admin_staff(&data.db).await.unwrap_or_default();
    render(AdminStaffTemplate {
        staff_users,
        has_error: !success,
        error: if success { String::new() } else { message.clone() },
        has_success: success,
        success: if success { message } else { String::new() },
        current_admin_id,
    })
}

// Parses uuid from form input into a safer internal value.
fn parse_uuid(value: String) -> Result<Uuid, HttpResponse> {
    Uuid::parse_str(&value).map_err(|_| redirect("/admin/dashboard"))
}
