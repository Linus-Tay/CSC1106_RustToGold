
use crate::controllers::error_controller::render_error;
use crate::controllers::session_guard::{redirect, require_customer};
use crate::forms::{LoanApplicationForm, LoanPaymentForm};
use crate::services;
use crate::views::{render, LoanApplyTemplate, LoanDashboardTemplate};
use crate::AppState;
use actix_session::Session;
use actix_web::{web, HttpResponse, Result};
use uuid::Uuid;

// Render loans page
pub async fn loans_page(data: web::Data<AppState>, session: Session) -> Result<HttpResponse> {
    let user = match require_customer(&data, &session).await {
        Ok(user) => user,
        Err(response) => return Ok(response),
    };
    let customer_id = user.customer_id_or_nil();

    match services::load_loan_dashboard(&data.db, customer_id).await {
        Ok(dashboard) => render(LoanDashboardTemplate {
            account: dashboard.account,
            accounts: dashboard.accounts,
            has_loans: !dashboard.loans.is_empty(),
            loans: dashboard.loans,
            error: String::new(),
            has_error: false,
        }),
        Err(error) => render_error("Loans unavailable", error),
    }
}

// Render loan apply page
pub async fn loan_apply_page(data: web::Data<AppState>, session: Session) -> Result<HttpResponse> {
    let user = match require_customer(&data, &session).await {
        Ok(user) => user,
        Err(response) => return Ok(response),
    };
    let customer_id = user.customer_id_or_nil();

    let accounts = match services::list_active_customer_products(&data.db, customer_id).await {
        Ok(accounts) => accounts,
        Err(error) => return render_error("Loan application unavailable", error),
    };

    render(LoanApplyTemplate {
        has_accounts: !accounts.is_empty(),
        accounts,
        error: String::new(),
        has_error: false,
    })
}

// Handle create personal loan
pub async fn create_personal_loan(
    data: web::Data<AppState>,
    session: Session,
    form: web::Form<LoanApplicationForm>,
) -> Result<HttpResponse> {
    let user = match require_customer(&data, &session).await {
        Ok(user) => user,
        Err(response) => return Ok(response),
    };
    let customer_id = user.customer_id_or_nil();

    match services::apply_personal_loan(&data.db, customer_id, form.into_inner()).await {
        Ok(_) => Ok(redirect("/customer/loans")),
        Err(error) => {
            let accounts = services::list_active_customer_products(&data.db, customer_id)
                .await
                .unwrap_or_default();
            render(LoanApplyTemplate {
                has_accounts: !accounts.is_empty(),
                accounts,
                error,
                has_error: true,
            })
        }
    }
}

// Handle pay loan
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
    let customer_id = user.customer_id_or_nil();

    let loan_id = match Uuid::parse_str(&path.into_inner()) {
        Ok(value) => value,
        Err(_) => {
            return render_error(
                "Invalid loan",
                "The selected loan ID is invalid.".to_string(),
            )
        }
    };

    match services::pay_personal_loan(&data.db, customer_id, loan_id, form.into_inner()).await
    {
        Ok(_) => Ok(redirect("/customer/loans")),
        Err(error) => render_error("Loan payment failed", error),
    }
}
