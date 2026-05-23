use crate::controllers::error_controller::render_error;
use crate::controllers::session_guard::{redirect, require_admin, require_customer};
use crate::forms::{CreateFixedDepositForm, FixedDepositPlanForm};
use crate::services;
use crate::views::render;
use crate::views::templates::{
    AdminFixedDepositPlansTemplate, AdminFixedDepositsTemplate, FixedDepositCreateTemplate,
    FixedDepositDashboardTemplate,
};
use crate::AppState;
use actix_session::Session;
use actix_web::{web, HttpResponse, Result};

fn display_money_without_symbol(value: String) -> String {
    value.trim_start_matches('$').to_string()
}

pub async fn fixed_deposits_page(
    data: web::Data<AppState>,
    session: Session,
) -> Result<HttpResponse> {
    let user = match require_customer(&data, &session).await {
        Ok(user) => user,
        Err(response) => return Ok(response),
    };

    match services::load_fixed_deposit_dashboard(&data.db, user.id).await {
        Ok(view_data) => {
            let has_fixed_deposits = !view_data.fixed_deposits.is_empty();

            let account_number = view_data.account.account_number.clone();
            let balance = display_money_without_symbol(view_data.account.balance_display());

            render(FixedDepositDashboardTemplate {
                account_number,
                balance,
                summary: view_data.summary,
                fixed_deposits: view_data.fixed_deposits,
                has_fixed_deposits,
                success: String::new(),
                has_success: false,
                error: String::new(),
                has_error: false,
            })
        }
        Err(message) => render_error("Fixed deposits unavailable", message),
    }
}

pub async fn fixed_deposit_new_page(
    data: web::Data<AppState>,
    session: Session,
) -> Result<HttpResponse> {
    let user = match require_customer(&data, &session).await {
        Ok(user) => user,
        Err(response) => return Ok(response),
    };

    match services::load_create_fixed_deposit_page(&data.db, user.id).await {
        Ok((account, plans)) => {
            let account_number = account.account_number.clone();
            let balance = display_money_without_symbol(account.balance_display());

            render(FixedDepositCreateTemplate {
                account_number,
                balance,
                plans,
                error: String::new(),
                has_error: false,
            })
        }
        Err(message) => render_error("Fixed deposit form unavailable", message),
    }
}

pub async fn create_fixed_deposit(
    data: web::Data<AppState>,
    session: Session,
    form: web::Form<CreateFixedDepositForm>,
) -> Result<HttpResponse> {
    let user = match require_customer(&data, &session).await {
        Ok(user) => user,
        Err(response) => return Ok(response),
    };

    match services::create_fixed_deposit(&data.db, user.id, form.into_inner()).await {
        Ok(_) => Ok(redirect("/customer/fixed-deposits?created=1")),
        Err(error) => match services::load_create_fixed_deposit_page(&data.db, user.id).await {
            Ok((account, plans)) => {
                let account_number = account.account_number.clone();
                let balance = display_money_without_symbol(account.balance_display());

                render(FixedDepositCreateTemplate {
                    account_number,
                    balance,
                    plans,
                    error,
                    has_error: true,
                })
            }
            Err(message) => render_error("Fixed deposit form unavailable", message),
        },
    }
}

pub async fn withdraw_fixed_deposit(
    data: web::Data<AppState>,
    session: Session,
    path: web::Path<i64>,
) -> Result<HttpResponse> {
    let user = match require_customer(&data, &session).await {
        Ok(user) => user,
        Err(response) => return Ok(response),
    };

    match services::withdraw_fixed_deposit(&data.db, user.id, path.into_inner()).await {
        Ok(_) => Ok(redirect("/customer/fixed-deposits?withdrawn=1")),
        Err(message) => render_error("Fixed deposit withdrawal failed", message),
    }
}

pub async fn admin_fixed_deposits_page(
    data: web::Data<AppState>,
    session: Session,
) -> Result<HttpResponse> {
    if let Err(response) = require_admin(&data, &session).await {
        return Ok(response);
    }

    match services::list_all_fixed_deposits(&data.db).await {
        Ok(fixed_deposits) => {
            let has_fixed_deposits = !fixed_deposits.is_empty();

            render(AdminFixedDepositsTemplate {
                fixed_deposits,
                has_fixed_deposits,
            })
        }
        Err(message) => render_error("Admin fixed deposits unavailable", message),
    }
}

pub async fn admin_fixed_deposit_plans_page(
    data: web::Data<AppState>,
    session: Session,
) -> Result<HttpResponse> {
    if let Err(response) = require_admin(&data, &session).await {
        return Ok(response);
    }

    match services::list_all_fixed_deposit_plans(&data.db).await {
        Ok(plans) => render(AdminFixedDepositPlansTemplate {
            plans,
            error: String::new(),
            has_error: false,
            success: String::new(),
            has_success: false,
        }),
        Err(message) => render_error("Admin fixed deposit plans unavailable", message),
    }
}

pub async fn create_fixed_deposit_plan(
    data: web::Data<AppState>,
    session: Session,
    form: web::Form<FixedDepositPlanForm>,
) -> Result<HttpResponse> {
    if let Err(response) = require_admin(&data, &session).await {
        return Ok(response);
    }

    match services::create_fixed_deposit_plan(&data.db, form.into_inner()).await {
        Ok(_) => Ok(redirect("/admin/fixed-deposit-plans?created=1")),
        Err(error) => render_plan_error(&data, error).await,
    }
}

pub async fn update_fixed_deposit_plan(
    data: web::Data<AppState>,
    session: Session,
    path: web::Path<i64>,
    form: web::Form<FixedDepositPlanForm>,
) -> Result<HttpResponse> {
    if let Err(response) = require_admin(&data, &session).await {
        return Ok(response);
    }

    match services::update_fixed_deposit_plan(&data.db, path.into_inner(), form.into_inner()).await {
        Ok(_) => Ok(redirect("/admin/fixed-deposit-plans?updated=1")),
        Err(error) => render_plan_error(&data, error).await,
    }
}

async fn render_plan_error(data: &web::Data<AppState>, error: String) -> Result<HttpResponse> {
    match services::list_all_fixed_deposit_plans(&data.db).await {
        Ok(plans) => render(AdminFixedDepositPlansTemplate {
            plans,
            error,
            has_error: true,
            success: String::new(),
            has_success: false,
        }),
        Err(message) => render_error("Admin fixed deposit plans unavailable", message),
    }
}