use crate::controllers::error_controller::render_error;
use crate::controllers::session_guard::{redirect, require_admin, require_customer};
use crate::forms::{HomeLoanApplicationForm, HomeLoanPaymentForm};
use crate::services;
use crate::views::{
    render, AdminHomeLoansTemplate, HomeLoanApplyTemplate, HomeLoanDashboardTemplate,
};
use crate::AppState;
use actix_session::Session;
use actix_web::{web, HttpResponse, Result};
use uuid::Uuid;

pub async fn home_loans_page(data: web::Data<AppState>, session: Session) -> Result<HttpResponse> {
    let user = match require_customer(&data, &session).await {
        Ok(user) => user,
        Err(response) => return Ok(response),
    };

    match services::load_home_loan_dashboard(&data.db, user.customer_id).await {
        Ok(dashboard) => render(HomeLoanDashboardTemplate {
            account: dashboard.account,
            summary: dashboard.summary,
            has_applications: !dashboard.applications.is_empty(),
            applications: dashboard.applications,
            error: String::new(),
            has_error: false,
        }),
        Err(error) => render_error("Home loans unavailable", error),
    }
}

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

    match services::submit_home_loan_application(&data.db, user.customer_id, form.into_inner())
        .await
    {
        Ok(_) => Ok(redirect("/customer/home-loans")),
        Err(error) => render(HomeLoanApplyTemplate {
            error,
            has_error: true,
        }),
    }
}

pub async fn pay_home_loan(
    data: web::Data<AppState>,
    session: Session,
    path: web::Path<String>,
    form: web::Form<HomeLoanPaymentForm>,
) -> Result<HttpResponse> {
    let user = match require_customer(&data, &session).await {
        Ok(user) => user,
        Err(response) => return Ok(response),
    };

    let application_id = match Uuid::parse_str(&path.into_inner()) {
        Ok(value) => value,
        Err(_) => {
            return render_error(
                "Invalid home loan",
                "The selected application ID is invalid.".to_string(),
            )
        }
    };

    match services::pay_home_loan(
        &data.db,
        user.customer_id,
        application_id,
        form.into_inner(),
    )
    .await
    {
        Ok(_) => Ok(redirect("/customer/home-loans")),
        Err(error) => render_error("Home loan payment failed", error),
    }
}

pub async fn admin_home_loans_page(
    data: web::Data<AppState>,
    session: Session,
) -> Result<HttpResponse> {
    if let Err(response) = require_admin(&data, &session).await {
        return Ok(response);
    }

    match services::list_admin_home_loans(&data.db).await {
        Ok(records) => render(AdminHomeLoansTemplate {
            has_records: !records.is_empty(),
            records,
            error: String::new(),
            has_error: false,
        }),
        Err(error) => render(AdminHomeLoansTemplate {
            records: Vec::new(),
            has_records: false,
            error,
            has_error: true,
        }),
    }
}

pub async fn approve_home_loan(
    data: web::Data<AppState>,
    session: Session,
    path: web::Path<String>,
) -> Result<HttpResponse> {
    let staff = match require_admin(&data, &session).await {
        Ok(user) => user,
        Err(response) => return Ok(response),
    };

    let application_id = match Uuid::parse_str(&path.into_inner()) {
        Ok(value) => value,
        Err(_) => {
            return render_error(
                "Invalid home loan",
                "The selected application ID is invalid.".to_string(),
            )
        }
    };

    match services::approve_home_loan(&data.db, staff.id, application_id).await {
        Ok(_) => Ok(redirect("/admin/home-loans")),
        Err(error) => render_error("Home loan approval failed", error),
    }
}

pub async fn reject_home_loan(
    data: web::Data<AppState>,
    session: Session,
    path: web::Path<String>,
) -> Result<HttpResponse> {
    let staff = match require_admin(&data, &session).await {
        Ok(user) => user,
        Err(response) => return Ok(response),
    };

    let application_id = match Uuid::parse_str(&path.into_inner()) {
        Ok(value) => value,
        Err(_) => {
            return render_error(
                "Invalid home loan",
                "The selected application ID is invalid.".to_string(),
            )
        }
    };

    match services::reject_home_loan(&data.db, staff.id, application_id).await {
        Ok(_) => Ok(redirect("/admin/home-loans")),
        Err(error) => render_error("Home loan rejection failed", error),
    }
}
