use crate::controllers::session_guard::{redirect, require_customer};
use crate::forms::HomeLoanApplicationForm;
use crate::services;
use crate::views::{render, HomeLoanApplyTemplate};
use crate::AppState;
use crate::controllers::error_controller::render_error;
use actix_session::Session;
use actix_web::{web, HttpResponse, Result};

pub async fn home_loan_apply_page(
    data: web::Data<AppState>,
    session: Session,
) -> Result<HttpResponse> {
    if let Err(response) = require_customer(&data, &session).await {
        return Ok(response);
    }

    render(HomeLoanApplyTemplate {
        error: String::new(),
        has_error: false,
        success: String::new(),
        has_success: false,
    })
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
        Ok(_) => Ok(redirect("/customer/loans?home_applied=1")),
        Err(message) => render(HomeLoanApplyTemplate {
            error: message,
            has_error: true,
            success: String::new(),
            has_success: false,
        }),
    }
}

pub async fn pay_home_loan(
    data: web::Data<AppState>,
    session: Session,
    path: web::Path<i64>,
) -> Result<HttpResponse> {
    let user = match require_customer(&data, &session).await {
        Ok(user) => user,
        Err(response) => return Ok(response),
    };

    match services::pay_home_loan(&data.db, user.id, path.into_inner()).await {
        Ok(_) => Ok(redirect("/customer/loans?home_paid=1")),
        Err(message) => render_error("Home loan repayment failed", message),
    }
}