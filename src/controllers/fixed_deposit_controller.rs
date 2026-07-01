// Controller layer: handles HTTP/session flow and delegates business rules to services.

use crate::controllers::error_controller::render_error;
use crate::controllers::session_guard::{redirect, require_admin, require_customer};
use crate::forms::{CreateFixedDepositForm, FixedDepositMessageQuery, FixedDepositPlanForm};
use crate::services;
use crate::views::{
    render, AdminFixedDepositPlansTemplate, AdminFixedDepositsTemplate, FixedDepositCreateTemplate,
    FixedDepositDashboardTemplate,
};
use crate::AppState;
use actix_session::Session;
use actix_web::{web, HttpResponse, Result};
use uuid::Uuid;

// Handles the display money without symbol request.
fn display_money_without_symbol(value: String) -> String {
    value.trim_start_matches('$').to_string()
}

// Renders the fixed deposits page screen with data prepared by the service layer.
pub async fn fixed_deposits_page(
    data: web::Data<AppState>,
    session: Session,
    query: web::Query<FixedDepositMessageQuery>,
) -> Result<HttpResponse> {
    let user = match require_customer(&data, &session).await {
        Ok(user) => user,
        Err(response) => return Ok(response),
    };
    let customer_id = user.customer_id_or_nil();

    match services::load_fixed_deposit_dashboard(&data.db, customer_id).await {
        Ok(dashboard) => {
            let accounts = dashboard.accounts.clone();
            let success = if query.created.is_some() {
                "Fixed deposit placed successfully.".to_string()
            } else if query.withdrawn.is_some() {
                "Fixed deposit withdrawn. Interest is paid only when the placement has matured."
                    .to_string()
            } else if query.paid_out.is_some() {
                "Matured fixed deposit paid out successfully.".to_string()
            } else {
                String::new()
            };

            render(FixedDepositDashboardTemplate {
                account_number: dashboard.account.account_number.clone(),
                balance: display_money_without_symbol(dashboard.account.balance_display()),
                accounts,
                summary: dashboard.summary,
                has_fixed_deposits: !dashboard.fixed_deposits.is_empty(),
                fixed_deposits: dashboard.fixed_deposits,
                has_success: !success.is_empty(),
                success,
                error: String::new(),
                has_error: false,
            })
        }
        Err(error) => render_error("Fixed deposits unavailable", error),
    }
}

// Renders the fixed deposit new page screen with data prepared by the service layer.
pub async fn fixed_deposit_new_page(
    data: web::Data<AppState>,
    session: Session,
) -> Result<HttpResponse> {
    let user = match require_customer(&data, &session).await {
        Ok(user) => user,
        Err(response) => return Ok(response),
    };
    let customer_id = user.customer_id_or_nil();

    match services::load_fixed_deposit_create_page(&data.db, customer_id).await {
        Ok((account, accounts, plans)) => render(FixedDepositCreateTemplate {
            account_number: account.account_number.clone(),
            balance: display_money_without_symbol(account.balance_display()),
            accounts,
            has_plans: !plans.is_empty(),
            plans,
            error: String::new(),
            has_error: false,
        }),
        Err(error) => render_error("Fixed deposit unavailable", error),
    }
}

// Handles the create fixed deposit form action and redirects after the service result.
pub async fn create_fixed_deposit(
    data: web::Data<AppState>,
    session: Session,
    form: web::Form<CreateFixedDepositForm>,
) -> Result<HttpResponse> {
    let user = match require_customer(&data, &session).await {
        Ok(user) => user,
        Err(response) => return Ok(response),
    };
    let customer_id = user.customer_id_or_nil();

    match services::create_fixed_deposit(&data.db, customer_id, form.into_inner()).await {
        Ok(_) => Ok(redirect("/customer/fixed-deposits?created=1")),
        Err(error) => {
            match services::load_fixed_deposit_create_page(&data.db, customer_id).await {
                Ok((account, accounts, plans)) => render(FixedDepositCreateTemplate {
                    account_number: account.account_number.clone(),
                    balance: display_money_without_symbol(account.balance_display()),
                    accounts,
                    has_plans: !plans.is_empty(),
                    plans,
                    error,
                    has_error: true,
                }),
                Err(_) => render_error("Fixed deposit failed", error),
            }
        }
    }
}

// Handles the withdraw fixed deposit form action and redirects after the service result.
pub async fn withdraw_fixed_deposit(
    data: web::Data<AppState>,
    session: Session,
    path: web::Path<String>,
) -> Result<HttpResponse> {
    let user = match require_customer(&data, &session).await {
        Ok(user) => user,
        Err(response) => return Ok(response),
    };
    let customer_id = user.customer_id_or_nil();

    let fixed_deposit_id = match Uuid::parse_str(&path.into_inner()) {
        Ok(value) => value,
        Err(_) => {
            return render_error(
                "Invalid fixed deposit",
                "The selected fixed deposit ID is invalid.".to_string(),
            )
        }
    };

    match services::withdraw_fixed_deposit(&data.db, customer_id, fixed_deposit_id).await {
        Ok(status) if status == "paid_out" => Ok(redirect("/customer/fixed-deposits?paid_out=1")),
        Ok(_) => Ok(redirect("/customer/fixed-deposits?withdrawn=1")),
        Err(error) => render_error("Fixed deposit withdrawal failed", error),
    }
}

// Renders the admin fixed deposits page screen with data prepared by the service layer.
pub async fn admin_fixed_deposits_page(
    data: web::Data<AppState>,
    session: Session,
) -> Result<HttpResponse> {
    if let Err(response) = require_admin(&data, &session).await {
        return Ok(response);
    }

    match services::list_admin_fixed_deposits(&data.db).await {
        Ok(records) => render(AdminFixedDepositsTemplate {
            has_records: !records.is_empty(),
            records,
        }),
        Err(error) => render_error("Fixed deposit admin unavailable", error),
    }
}

// Renders the admin fixed deposit plans page screen with data prepared by the service layer.
pub async fn admin_fixed_deposit_plans_page(
    data: web::Data<AppState>,
    session: Session,
    query: web::Query<FixedDepositMessageQuery>,
) -> Result<HttpResponse> {
    if let Err(response) = require_admin(&data, &session).await {
        return Ok(response);
    }

    let success = if query.created.is_some() {
        "Fixed deposit plan created.".to_string()
    } else if query.updated.is_some() {
        "Fixed deposit plan updated.".to_string()
    } else {
        String::new()
    };

    match services::list_admin_plans(&data.db).await {
        Ok(plans) => render(AdminFixedDepositPlansTemplate {
            plans,
            error: String::new(),
            has_error: false,
            has_success: !success.is_empty(),
            success,
        }),
        Err(error) => render(AdminFixedDepositPlansTemplate {
            plans: Vec::new(),
            error,
            has_error: true,
            success: String::new(),
            has_success: false,
        }),
    }
}

// Handles the create fixed deposit plan form action and redirects after the service result.
pub async fn create_fixed_deposit_plan(
    data: web::Data<AppState>,
    session: Session,
    form: web::Form<FixedDepositPlanForm>,
) -> Result<HttpResponse> {
    if let Err(response) = require_admin(&data, &session).await {
        return Ok(response);
    }

    match services::create_plan(&data.db, form.into_inner()).await {
        Ok(_) => Ok(redirect("/admin/fixed-deposit-plans?created=1")),
        Err(error) => match services::list_admin_plans(&data.db).await {
            Ok(plans) => render(AdminFixedDepositPlansTemplate {
                plans,
                error,
                has_error: true,
                success: String::new(),
                has_success: false,
            }),
            Err(_) => render_error("Plan creation failed", error),
        },
    }
}

// Handles the update fixed deposit plan form action and redirects after the service result.
pub async fn update_fixed_deposit_plan(
    data: web::Data<AppState>,
    session: Session,
    path: web::Path<i64>,
    form: web::Form<FixedDepositPlanForm>,
) -> Result<HttpResponse> {
    if let Err(response) = require_admin(&data, &session).await {
        return Ok(response);
    }

    match services::update_plan(&data.db, path.into_inner(), form.into_inner()).await {
        Ok(_) => Ok(redirect("/admin/fixed-deposit-plans?updated=1")),
        Err(error) => render_error("Plan update failed", error),
    }
}
