use crate::controllers::error_controller::render_error;
use crate::controllers::session_guard::{redirect, require_admin};
use crate::services;
use crate::views::{
    render, AdminCustomerApplicationsTemplate, AdminDashboardTemplate, AdminPersonalLoansTemplate,
};
use crate::AppState;
use actix_session::Session;
use actix_web::{web, HttpResponse, Result};
use uuid::Uuid;

pub async fn admin_dashboard(data: web::Data<AppState>, session: Session) -> Result<HttpResponse> {
    if let Err(response) = require_admin(&data, &session).await {
        return Ok(response);
    }

    match services::load_admin_dashboard(&data.db).await {
        Ok(summary) => render(AdminDashboardTemplate { summary }),
        Err(error) => render_error("Admin dashboard unavailable", error),
    }
}

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

pub async fn approve_customer_application(
    data: web::Data<AppState>,
    session: Session,
    path: web::Path<String>,
) -> Result<HttpResponse> {
    if let Err(response) = require_admin(&data, &session).await {
        return Ok(response);
    }

    let customer_id = match parse_uuid(path.into_inner()) {
        Ok(value) => value,
        Err(response) => return Ok(response),
    };

    match services::approve_customer_application(&data, customer_id).await {
        Ok(_) => Ok(redirect("/admin/signups")),
        Err(error) => render_error("Customer approval failed", error),
    }
}

pub async fn reject_customer_application(
    data: web::Data<AppState>,
    session: Session,
    path: web::Path<String>,
) -> Result<HttpResponse> {
    if let Err(response) = require_admin(&data, &session).await {
        return Ok(response);
    }

    let customer_id = match parse_uuid(path.into_inner()) {
        Ok(value) => value,
        Err(response) => return Ok(response),
    };

    match services::reject_customer_application(&data.db, customer_id).await {
        Ok(_) => Ok(redirect("/admin/signups")),
        Err(error) => render_error("Customer rejection failed", error),
    }
}

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

fn parse_uuid(value: String) -> Result<Uuid, HttpResponse> {
    Uuid::parse_str(&value).map_err(|_| redirect("/admin/dashboard"))
}
