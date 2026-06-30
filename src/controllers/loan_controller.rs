use crate::controllers::error_controller::render_error;
use crate::controllers::session_guard::{redirect, require_customer};
use crate::forms::{LoanApplicationForm, LoanPaymentForm};
use crate::services;
use crate::views::{render, LoanApplyTemplate, LoanDashboardTemplate};
use crate::AppState;
use actix_session::Session;
use actix_web::{web, HttpResponse, Result};
use uuid::Uuid;

pub async fn loans_page(data: web::Data<AppState>, session: Session) -> Result<HttpResponse> {
    let user = match require_customer(&data, &session).await {
        Ok(user) => user,
        Err(response) => return Ok(response),
    };

    match services::load_loan_dashboard(&data.db, user.customer_id).await {
        Ok(dashboard) => render(LoanDashboardTemplate {
            account: dashboard.account,
            has_loans: !dashboard.loans.is_empty(),
            loans: dashboard.loans,
            error: String::new(),
            has_error: false,
        }),
        Err(error) => render_error("Loans unavailable", error),
    }
}

pub async fn loan_apply_page(data: web::Data<AppState>, session: Session) -> Result<HttpResponse> {
    if let Err(response) = require_customer(&data, &session).await {
        return Ok(response);
    }

    render(LoanApplyTemplate {
        error: String::new(),
        has_error: false,
    })
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

    match services::apply_personal_loan(&data.db, user.customer_id, form.into_inner()).await {
        Ok(_) => Ok(redirect("/customer/loans")),
        Err(error) => render(LoanApplyTemplate {
            error,
            has_error: true,
        }),
    }
}

pub async fn pay_loan(
    data: web::Data<AppState>,
    session: Session,
    path: web::Path<String>,
    form: web::Form<LoanPaymentForm>,
) -> Result<HttpResponse> {
    let user = match require_customer(&data, &session).await {
        Ok(user) => user,
        Err(response) => return Ok(response),
    };

    let loan_id = match Uuid::parse_str(&path.into_inner()) {
        Ok(value) => value,
        Err(_) => {
            return render_error(
                "Invalid loan",
                "The selected loan ID is invalid.".to_string(),
            )
        }
    };

    match services::pay_personal_loan(&data.db, user.customer_id, loan_id, form.into_inner()).await
    {
        Ok(_) => Ok(redirect("/customer/loans")),
        Err(error) => render_error("Loan payment failed", error),
    }
}
