use crate::controllers::error_controller::render_error;
use crate::controllers::session_guard::{redirect, require_admin, require_customer};
use crate::forms::{HomeLoanApplicationForm, HomeLoanPaymentForm};
use crate::services;
use crate::views::{render, AdminHomeLoansTemplate, HomeLoanApplyTemplate, HomeLoanDashboardTemplate};
use crate::AppState;
use actix_session::Session;
use actix_web::{web, HttpResponse, Result};

pub async fn home_loans_page(
    data: web::Data<AppState>,
    session: Session,
) -> Result<HttpResponse> {
    let user = match require_customer(&data, &session).await {
        Ok(user) => user,
        Err(response) => return Ok(response),
    };

    match services::load_home_loan_dashboard(&data.db, user.id).await {
        Ok(dashboard) => {
            let has_applications = !dashboard.applications.is_empty();

            render(HomeLoanDashboardTemplate {
                account: dashboard.account,
                summary: dashboard.summary,
                applications: dashboard.applications,
                has_applications,
                error: String::new(),
                has_error: false,
            })
        }
        Err(message) => render_error("Home loans unavailable", message),
    }
}

pub async fn home_loan_apply_page(
    data: web::Data<AppState>,
    session: Session,
) -> Result<HttpResponse> {
    match require_customer(&data, &session).await {
        Ok(_) => render(HomeLoanApplyTemplate {
            error: String::new(),
            has_error: false,
        }),
        Err(response) => Ok(response),
    }
}

pub async fn create_home_loan_application(
    data: web::Data<AppState>,
    session: Session,
    form: web::Form<HomeLoanApplicationForm>,
) -> Result<HttpResponse> {
    let user = match require_customer(&data, &session).await {
        Ok(user) => user,
        Err(response) => return Ok(response),
    };

    match services::apply_home_loan(&data.db, user.id, form.into_inner()).await {
        Ok(_) => Ok(redirect("/customer/home-loans")),
        Err(message) => render(HomeLoanApplyTemplate {
            error: message,
            has_error: true,
        }),
    }
}

pub async fn pay_home_loan(
    data: web::Data<AppState>,
    session: Session,
    path: web::Path<i64>,
    form: web::Form<HomeLoanPaymentForm>,
) -> Result<HttpResponse> {
    let user = match require_customer(&data, &session).await {
        Ok(user) => user,
        Err(response) => return Ok(response),
    };

    match services::pay_home_loan(&data.db, user.id, path.into_inner(), form.into_inner()).await {
        Ok(_) => Ok(redirect("/customer/home-loans")),
        Err(message) => render_error("Home loan payment failed", message),
    }
}

pub async fn admin_home_loans_page(
    data: web::Data<AppState>,
    session: Session,
) -> Result<HttpResponse> {
    if let Err(response) = require_admin(&data, &session).await {
        return Ok(response);
    }

    match services::list_all_home_loans_for_admin(&data.db).await {
        Ok(records) => {
            let has_records = !records.is_empty();

            render(AdminHomeLoansTemplate {
                records,
                has_records,
                error: String::new(),
                has_error: false,
            })
        }
        Err(message) => render_error("Home loan records unavailable", message),
    }
}

pub async fn approve_home_loan(
    data: web::Data<AppState>,
    session: Session,
    path: web::Path<i64>,
) -> Result<HttpResponse> {
    let staff = match require_admin(&data, &session).await {
        Ok(user) => user,
        Err(response) => return Ok(response),
    };

    match services::approve_home_loan(&data.db, staff.id, path.into_inner()).await {
        Ok(_) => Ok(redirect("/admin/home-loans")),
        Err(message) => render_error("Home loan approval failed", message),
    }
}

pub async fn reject_home_loan(
    data: web::Data<AppState>,
    session: Session,
    path: web::Path<i64>,
) -> Result<HttpResponse> {
    if let Err(response) = require_admin(&data, &session).await {
        return Ok(response);
    }

    match services::reject_home_loan(&data.db, path.into_inner()).await {
        Ok(_) => Ok(redirect("/admin/home-loans")),
        Err(message) => render_error("Home loan rejection failed", message),
    }
}