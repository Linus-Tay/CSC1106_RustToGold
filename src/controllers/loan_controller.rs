use crate::controllers::error_controller::render_error;
use crate::controllers::session_guard::{redirect, require_customer};
use crate::forms::{LoanApplicationForm, LoanPaymentForm};
use crate::services;
use crate::views::{render, LoanApplyTemplate, LoanDashboardTemplate};
use crate::AppState;
use actix_session::Session;
use actix_web::{web, HttpResponse, Result};

pub async fn loans_page(
    data: web::Data<AppState>,
    session: Session,
) -> Result<HttpResponse> {
    let user = match require_customer(&data, &session).await {
        Ok(user) => user,
        Err(response) => return Ok(response),
    };

    match services::load_loan_dashboard(&data.db, user.id).await {
        Ok(dashboard) => {
            let has_loans = !dashboard.loans.is_empty();

            render(LoanDashboardTemplate {
                account: dashboard.account,
                loans: dashboard.loans,
                has_loans,
                error: String::new(),
                has_error: false,
            })
        }
        Err(message) => render_error("Loans unavailable", message),
    }
}

pub async fn loan_apply_page(
    session: Session,
    data: web::Data<AppState>,
) -> Result<HttpResponse> {
    match require_customer(&data, &session).await {
        Ok(_) => render(LoanApplyTemplate {
            error: String::new(),
            has_error: false,
        }),
        Err(response) => Ok(response),
    }
}

pub async fn create_personal_loan(
    data: web::Data<AppState>,
    session: Session,
    form: web::Form<LoanApplicationForm>,
) -> Result<HttpResponse> {
    let user = match require_customer(&data, &session).await {
        Ok(user) => user,
        Err(response) => return Ok(response),
    };

    match services::apply_personal_loan(&data.db, user.id, form.into_inner()).await {
        Ok(_) => Ok(redirect("/customer/loans?created=1")),
        Err(message) => render(LoanApplyTemplate {
            error: message,
            has_error: true,
        }),
    }
}

pub async fn pay_loan(
    data: web::Data<AppState>,
    session: Session,
    path: web::Path<i64>,
    form: web::Form<LoanPaymentForm>,
) -> Result<HttpResponse> {
    let user = match require_customer(&data, &session).await {
        Ok(user) => user,
        Err(response) => return Ok(response),
    };

    match services::pay_loan(&data.db, user.id, path.into_inner(), form.into_inner()).await {
        Ok(_) => Ok(redirect("/customer/loans?paid=1")),
        Err(message) => render_error("Loan payment failed", message),
    }
}