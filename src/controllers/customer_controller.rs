use crate::controllers::error_controller::render_error;
use crate::controllers::session_guard::require_customer;
use crate::forms::{DepositForm, ProfileForm};
use crate::repositories::account_repository;
use crate::services;
use crate::views::{
    render, CustomerPageTemplate, DashboardTemplate, DepositTemplate, ProfileTemplate,
    TransactionsTemplate,
};
use crate::AppState;
use actix_session::Session;
use actix_web::{web, HttpResponse, Result};

fn display_money_without_symbol(value: String) -> String {
    value.trim_start_matches('$').to_string()
}

pub async fn dashboard(data: web::Data<AppState>, session: Session) -> Result<HttpResponse> {
    let user = match require_customer(&data, &session).await {
        Ok(user) => user,
        Err(response) => return Ok(response),
    };

    match services::load_customer_dashboard(&data.db, user.id).await {
        Ok((account, transactions)) => {
            let has_transactions = !transactions.is_empty();
            let balance = display_money_without_symbol(account.balance_display());
            let account_number = account.account_number;

            render(DashboardTemplate {
                full_name: user.full_name,
                account_number,
                balance,
                recent_transactions: transactions,
                has_transactions,
            })
        }
        Err(message) => render_error("Dashboard unavailable", message),
    }
}

pub async fn deposit_page(data: web::Data<AppState>, session: Session) -> Result<HttpResponse> {
    let user = match require_customer(&data, &session).await {
        Ok(user) => user,
        Err(response) => return Ok(response),
    };

    let account = match account_repository::find_primary_account_by_user_id(&data.db, user.id).await {
        Ok(Some(account)) => account,
        _ => return render_error("Account unavailable", "No bank account was found.".to_string()),
    };

    let balance = display_money_without_symbol(account.balance_display());
    let account_number = account.account_number;

    render(DepositTemplate {
        account_number,
        balance,
        error: String::new(),
        has_error: false,
        success: String::new(),
        has_success: false,
    })
}

pub async fn deposit(
    data: web::Data<AppState>,
    session: Session,
    form: web::Form<DepositForm>,
) -> Result<HttpResponse> {
    let user = match require_customer(&data, &session).await {
        Ok(user) => user,
        Err(response) => return Ok(response),
    };

    match services::deposit(&data.db, user.id, form.into_inner()).await {
        Ok(account) => {
            let balance = display_money_without_symbol(account.balance_display());
            let account_number = account.account_number;

            render(DepositTemplate {
                account_number,
                balance,
                error: String::new(),
                has_error: false,
                success: "Deposit completed successfully.".to_string(),
                has_success: true,
            })
        }
        Err(error) => {
            let account = match account_repository::find_primary_account_by_user_id(&data.db, user.id).await {
                Ok(Some(account)) => account,
                _ => return render_error("Account unavailable", "No bank account was found.".to_string()),
            };

            let balance = display_money_without_symbol(account.balance_display());
            let account_number = account.account_number;

            render(DepositTemplate {
                account_number,
                balance,
                error,
                has_error: true,
                success: String::new(),
                has_success: false,
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
        Ok(transactions) => {
            let has_transactions = !transactions.is_empty();
            render(TransactionsTemplate {
                transactions,
                has_transactions,
            })
        }
        Err(message) => render_error("Transactions unavailable", message),
    }
}

pub async fn profile_page(data: web::Data<AppState>, session: Session) -> Result<HttpResponse> {
    let user = match require_customer(&data, &session).await {
        Ok(user) => user,
        Err(response) => return Ok(response),
    };

    let account = match account_repository::find_primary_account_by_user_id(&data.db, user.id).await {
        Ok(Some(account)) => account,
        _ => return render_error("Account unavailable", "No bank account was found.".to_string()),
    };

    let date_of_birth = user.date_of_birth_display();
    let last_login = user.last_login_display();
    let created_at = account.created_at.format("%d %b %Y").to_string();

    render(ProfileTemplate {
        full_name: user.full_name,
        email: user.email,
        phone: user.phone_number,
        date_of_birth,
        account_number: account.account_number,
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

    let account = match account_repository::find_primary_account_by_user_id(&data.db, user.id).await {
        Ok(Some(account)) => account,
        _ => return render_error("Account unavailable", "No bank account was found.".to_string()),
    };

    let updated_user = match services::update_customer_profile(&data.db, user.id, form.into_inner()).await {
        Ok(updated_user) => updated_user,
        Err(_) => user,
    };

    let date_of_birth = updated_user.date_of_birth_display();
    let last_login = updated_user.last_login_display();
    let created_at = account.created_at.format("%d %b %Y").to_string();

    render(ProfileTemplate {
        full_name: updated_user.full_name,
        email: updated_user.email,
        phone: updated_user.phone_number,
        date_of_birth,
        account_number: account.account_number,
        created_at,
        last_login,
    })
}

pub async fn transfer_page(data: web::Data<AppState>, session: Session) -> Result<HttpResponse> {
    if let Err(response) = require_customer(&data, &session).await {
        return Ok(response);
    }

    render(CustomerPageTemplate {
        title: "RustToGold | Transfer Money",
        active_nav: "transfer",
        heading: "Transfer Money",
        description: "Send money through a guided customer banking workflow.",
        message: "Transfer routing is connected for Phase 1. In Phase 2, this will validate recipient accounts, check balance, simulate OTP, and create debit/credit transaction records.",
        primary_label: "View Transactions",
        primary_href: "/customer/transactions",
    })
}

pub async fn loans_page(data: web::Data<AppState>, session: Session) -> Result<HttpResponse> {
    if let Err(response) = require_customer(&data, &session).await {
        return Ok(response);
    }

    render(CustomerPageTemplate {
        title: "RustToGold | Loans",
        active_nav: "loans",
        heading: "Loan Applications",
        description: "Apply for loans and track application status from the customer portal.",
        message: "The loan module route is connected for Phase 1. The next phase can add loan application forms, approval statuses, staff review, and admin reporting.",
        primary_label: "Apply for Loan",
        primary_href: "/customer/loans/apply",
    })
}

pub async fn loan_apply_page(data: web::Data<AppState>, session: Session) -> Result<HttpResponse> {
    if let Err(response) = require_customer(&data, &session).await {
        return Ok(response);
    }

    render(CustomerPageTemplate {
        title: "RustToGold | Apply for Loan",
        active_nav: "loans",
        heading: "Apply for Loan",
        description: "A placeholder for the loan application workflow.",
        message: "This page is intentionally prepared as a Phase 1 placeholder so the dashboard link works. Add the application form and loan repository in the next module.",
        primary_label: "Back to Loans",
        primary_href: "/customer/loans",
    })
}
