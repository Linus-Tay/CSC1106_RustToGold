use crate::controllers::error_controller::render_error;
use crate::controllers::session_guard::require_customer;
use crate::forms::account_forms::TransferForm;
use crate::forms::{DepositForm, ProfileForm};
use crate::repositories::{loan_repository, product_repository};
use crate::services::{self, product_service};
use crate::views::{
    render, CustomerActivityLogTemplate, DashboardTemplate, DepositTemplate, ErrorTemplate,
    ProfileTemplate, TransactionsTemplate, TransferTemplate,
};
use crate::AppState;
use actix_session::Session;
use actix_web::{web, HttpResponse, Result};
use uuid::Uuid;

fn display_money_without_symbol(value: String) -> String {
    value.trim_start_matches('$').to_string()
}

pub async fn dashboard(data: web::Data<AppState>, session: Session) -> Result<HttpResponse> {
    let user = match require_customer(&data, &session).await {
        Ok(user) => user,
        Err(response) => return Ok(response),
    };

    match services::load_customer_dashboard(&data.db, user.id).await {
        Ok(account) => render(DashboardTemplate {
            full_name: user.full_name,
            balance: display_money_without_symbol(account.balance_display()),
        }),
        Err(message) => render_error("Dashboard unavailable", message),
    }
}

pub async fn deposit_page(data: web::Data<AppState>, session: Session) -> Result<HttpResponse> {
    let user = match require_customer(&data, &session).await {
        Ok(user) => user,
        Err(response) => return Ok(response),
    };

    let account = match loan_repository::find_primary_active_product(&data.db, user.customer_id).await {
        Ok(account) => account,
        _ => return render_error("Account unavailable", "No active customer product account was found.".to_string()),
    };

    render(DepositTemplate {
        account_number: account.account_number.clone(),
        balance: display_money_without_symbol(account.balance_display()),
        error: String::new(),
        has_error: false,
        success: String::new(),
        has_success: false,
    })
}

pub async fn deposit(
    app_state: web::Data<AppState>,
    session: Session,
    form: web::Form<DepositForm>,
) -> Result<HttpResponse> {
    let user = match require_customer(&app_state, &session).await {
        Ok(user) => user,
        Err(response) => return Ok(response),
    };

    let account_number = form.account_number.clone();

    match services::deposit(&app_state, user.customer_id, form.into_inner()).await {
        Ok(account) => render(DepositTemplate {
            account_number: account.account_number.clone(),
            balance: display_money_without_symbol(account.balance_display()),
            error: String::new(),
            has_error: false,
            success: "Deposit completed successfully.".to_string(),
            has_success: true,
        }),
        Err(error) => {
            let account = match product_repository::get_product_by_account_number(&app_state.db, &account_number).await {
                Ok(Some(account)) => account,
                _ => return render_error("Account unavailable", "No bank account was found.".to_string()),
            };

            render(DepositTemplate {
                account_number: account.account_number.clone(),
                balance: display_money_without_symbol(account.balance_display()),
                error,
                has_error: true,
                success: String::new(),
                has_success: false,
            })
        }
    }
}

pub async fn transfer_page(data: web::Data<AppState>, session: Session) -> Result<HttpResponse> {
    let user = match require_customer(&data, &session).await {
        Ok(user) => user,
        Err(response) => return Ok(response),
    };

    let account = match loan_repository::find_primary_active_product(&data.db, user.customer_id).await {
        Ok(account) => account,
        _ => {
            return render_error(
                "Account unavailable",
                "No active customer product account was found.".to_string(),
            )
        }
    };

    render(TransferTemplate {
        account_number: account.account_number.clone(),
        balance: display_money_without_symbol(account.balance_display()),
        error: String::new(),
        has_error: false,
    })
}

pub async fn transfer(
    app_state: web::Data<AppState>,
    session: Session,
    form: web::Form<TransferForm>,
) -> Result<HttpResponse> {
    let user = match require_customer(&app_state, &session).await {
        Ok(user) => user,
        Err(response) => return Ok(response),
    };

    let form_data = form.into_inner();
    let account_number = form_data.account_number.clone();

    match services::transfer(&app_state, user.customer_id, form_data).await {
        Ok(true) => Ok(crate::controllers::session_guard::redirect("/customer/transactions")),
        Ok(false) => render(ErrorTemplate),
        Err(error) => {
            let account = match product_repository::get_product_by_account_number(&app_state.db, &account_number).await {
                Ok(Some(account)) => account,
                _ => return render_error("Account unavailable", "No bank account was found.".to_string()),
            };

            render(TransferTemplate {
                account_number: account.account_number.clone(),
                balance: display_money_without_symbol(account.balance_display()),
                error,
                has_error: true,
            })
        }
    }
}

pub async fn transactions(data: web::Data<AppState>, session: Session) -> Result<HttpResponse> {
    let user = match require_customer(&data, &session).await {
        Ok(user) => user,
        Err(response) => return Ok(response),
    };

    match services::list_transactions(&data.db, user.id).await {
        Ok(transactions) => render(TransactionsTemplate {
            has_transactions: !transactions.is_empty(),
            transactions,
        }),
        Err(message) => render_error("Transactions unavailable", message),
    }
}

pub async fn loan_activity(data: web::Data<AppState>, session: Session) -> Result<HttpResponse> {
    let user = match require_customer(&data, &session).await {
        Ok(user) => user,
        Err(response) => return Ok(response),
    };

    match services::list_loan_activity(&data.db, user.id).await {
        Ok(transactions) => {
            let has_transactions = !transactions.is_empty();
            render(CustomerActivityLogTemplate {
                eyebrow: "Loan Records",
                title: "Loan Activity Log",
                description: "Shows loan disbursements and repayments only. Everyday deposits and transfers stay under Transactions.",
                icon: "file-signature",
                empty_title: "No loan activity yet",
                empty_message: "Apply for a loan or make a repayment after approval to generate loan records.",
                transactions,
                has_transactions,
            })
        }
        Err(message) => render_error("Loan activity unavailable", message),
    }
}

pub async fn fixed_deposit_activity(
    data: web::Data<AppState>,
    session: Session,
) -> Result<HttpResponse> {
    let user = match require_customer(&data, &session).await {
        Ok(user) => user,
        Err(response) => return Ok(response),
    };

    match services::list_fixed_deposit_activity(&data.db, user.id).await {
        Ok(transactions) => {
            let has_transactions = !transactions.is_empty();
            render(CustomerActivityLogTemplate {
                eyebrow: "Fixed Deposit Records",
                title: "Fixed Deposit Log",
                description: "Shows fixed deposit openings, withdrawals and payouts only. Normal deposits and transfers stay under Transactions.",
                icon: "piggy-bank",
                empty_title: "No fixed deposit activity yet",
                empty_message: "Create or withdraw a fixed deposit placement to generate fixed deposit records.",
                transactions,
                has_transactions,
            })
        }
        Err(message) => render_error("Fixed deposit activity unavailable", message),
    }
}

pub async fn profile_page(data: web::Data<AppState>, session: Session) -> Result<HttpResponse> {
    let user = match require_customer(&data, &session).await {
        Ok(user) => user,
        Err(response) => return Ok(response),
    };

    let account = match loan_repository::find_primary_active_product(&data.db, user.customer_id).await {
        Ok(account) => account,
        _ => return render_error("Account unavailable", "No active customer product account was found.".to_string()),
    };

    let date_of_birth = user.date_of_birth_display();
    let last_login = user.last_login_display();
    let balance = display_money_without_symbol(account.balance_display());
    let account_type = account.product_id_display();
    let status = account.status_display();
    let created_at = account.created_at.format("%d %b %Y").to_string();

    render(ProfileTemplate {
        full_name: user.full_name,
        email: user.email,
        phone: user.phone_number,
        date_of_birth,
        account_number: account.account_number,
        balance,
        account_type,
        status,
        created_at,
        last_login,
    })
}

pub async fn update_profile(
    data: web::Data<AppState>,
    session: Session,
    form: web::Form<ProfileForm>,
) -> Result<HttpResponse> {
    let user = match require_customer(&data, &session).await {
        Ok(user) => user,
        Err(response) => return Ok(response),
    };

    let account = match loan_repository::find_primary_active_product(&data.db, user.customer_id).await {
        Ok(account) => account,
        _ => return render_error("Account unavailable", "No active customer product account was found.".to_string()),
    };

    let updated_user = match services::update_customer_profile(&data.db, user.id, form.into_inner()).await {
        Ok(updated_user) => updated_user,
        Err(_) => user,
    };

    let date_of_birth = updated_user.date_of_birth_display();
    let last_login = updated_user.last_login_display();
    let balance = display_money_without_symbol(account.balance_display());
    let account_type = account.product_id_display();
    let status = account.status_display();
    let created_at = account.created_at.format("%d %b %Y").to_string();

    render(ProfileTemplate {
        full_name: updated_user.full_name,
        email: updated_user.email,
        phone: updated_user.phone_number,
        date_of_birth,
        account_number: account.account_number,
        balance,
        account_type,
        status,
        created_at,
        last_login,
    })
}

pub async fn approve_product(
    data: web::Data<AppState>,
    path: web::Path<String>,
    session: Session,
) -> Result<HttpResponse> {
    if let Err(response) = require_customer(&data, &session).await {
        return Ok(response);
    }

    let account_id = match Uuid::parse_str(&path.into_inner()) {
        Ok(account_id) => account_id,
        Err(_) => return render(ErrorTemplate),
    };

    match product_service::approve_product(&data, account_id).await {
        Ok(_) => Ok(crate::controllers::session_guard::redirect("/customer/dashboard")),
        Err(error) => render_error("Product approval failed", error),
    }
}
