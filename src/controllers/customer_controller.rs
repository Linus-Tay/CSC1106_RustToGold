// Controller layer: handles HTTP/session flow and delegates business rules to services.

use crate::controllers::error_controller::render_error;
use crate::controllers::session_guard::{redirect, require_customer};
use crate::forms::account_forms::{CreateBankAccountForm, TransferForm};
use crate::forms::{CardApplicationForm, DepositForm, GiroArrangementForm, MoneyLockForm, PayNowRegisterForm, PayNowTransferForm, ProfileForm, StatementRequest, TransactionLimitForm};
use crate::repositories::customer_repository;
use crate::services;
use crate::views::{
    render, CardDashboardTemplate, CustomerActivityLogTemplate, DashboardTemplate, DepositTemplate,
    GiroTemplate, PayNowTemplate, ProfileTemplate, StatementTemplate, TransactionControlsTemplate,
    TransactionsTemplate, TransferTemplate,
};
use crate::AppState;
use crate::models::Product;
use actix_session::Session;
use actix_web::{web, HttpResponse, Result};

// Handles the display money without symbol request.
fn display_money_without_symbol(value: String) -> String {
    value.trim_start_matches('$').to_string()
}

struct DashboardLimitSummary {
    daily_limit_display: String,
    outgoing_today_display: String,
    remaining_today_display: String,
}

// Handles the load dashboard limit summary request.
async fn load_dashboard_limit_summary(
    data: &web::Data<AppState>,
    customer_id: uuid::Uuid,
) -> DashboardLimitSummary {
    match services::load_transaction_controls_page(&data.db, customer_id).await {
        Ok(page) => DashboardLimitSummary {
            daily_limit_display: page.controls.daily_limit_display(),
            outgoing_today_display: page.outgoing_today_display,
            remaining_today_display: page.remaining_today_display,
        },
        Err(_) => DashboardLimitSummary {
            daily_limit_display: "Unavailable".to_string(),
            outgoing_today_display: "Unavailable".to_string(),
            remaining_today_display: "Unavailable".to_string(),
        },
    }
}

struct SavingsApplicationState {
    can_apply_everyday_savings: bool,
    can_apply_high_yield_savings: bool,
    notice: String,
    has_notice: bool,
}

// Handles the savings application state request.
fn savings_application_state(accounts: &[Product]) -> SavingsApplicationState {
    let mut has_everyday_savings = false;
    let mut has_high_yield_savings = false;
    let mut pending_labels: Vec<&str> = Vec::new();

    for account in accounts.iter().filter(|account| account.status != "closed") {
        match account.product_id.as_str() {
            "everyday_savings" => {
                has_everyday_savings = true;
                if account.status == "inactive" {
                    pending_labels.push("Everyday Savings");
                }
            }
            "high_yield_savings" => {
                has_high_yield_savings = true;
                if account.status == "inactive" {
                    pending_labels.push("High-Yield Savings");
                }
            }
            _ => {}
        }
    }

    let notice = if !pending_labels.is_empty() {
        format!(
            "{} application received. Please wait up to 2 business days for admin approval. We will email you when it is ready.",
            pending_labels.join(" and ")
        )
    } else {
        String::new()
    };

    SavingsApplicationState {
        can_apply_everyday_savings: !has_everyday_savings,
        can_apply_high_yield_savings: !has_high_yield_savings,
        has_notice: !notice.is_empty(),
        notice,
    }
}

// Renders the dashboard screen with data prepared by the service layer.
pub async fn dashboard(data: web::Data<AppState>, session: Session) -> Result<HttpResponse> {
    let user = match require_customer(&data, &session).await {
        Ok(user) => user,
        Err(response) => return Ok(response),
    };
    let customer_id = user.customer_id_or_nil();

    let customer = match customer_repository::get_customer_by_id(&data.db, &customer_id).await {
        Ok(customer) => customer,
        Err(_) => return render_error("Profile unavailable", "Could not load your customer profile.".to_string()),
    };

    match services::load_customer_dashboard(&data.db, customer_id).await {
        Ok(dashboard) => {
            let account_state = savings_application_state(&dashboard.accounts);
            let limit_summary = load_dashboard_limit_summary(&data, customer_id).await;

            render(DashboardTemplate {
                full_name: customer.full_name,
                accounts: dashboard.accounts.clone(),
                has_accounts: !dashboard.accounts.is_empty(),
                can_apply_everyday_savings: account_state.can_apply_everyday_savings,
                can_apply_high_yield_savings: account_state.can_apply_high_yield_savings,
                account_application_notice: account_state.notice,
                has_account_application_notice: account_state.has_notice,
                daily_limit_display: limit_summary.daily_limit_display,
                outgoing_today_display: limit_summary.outgoing_today_display,
                remaining_today_display: limit_summary.remaining_today_display,
                create_account_error: String::new(),
                has_create_account_error: false,
            })
        },
        Err(message) => render_error("Dashboard unavailable", message),
    }
}

// Handles the create bank account form action and redirects after the service result.
pub async fn create_bank_account(
    data: web::Data<AppState>,
    session: Session,
    form: web::Form<CreateBankAccountForm>,
) -> Result<HttpResponse> {
    let user = match require_customer(&data, &session).await {
        Ok(user) => user,
        Err(response) => return Ok(response),
    };
    let customer_id = user.customer_id_or_nil();

    match services::create_bank_account(&data.db, customer_id, &form.into_inner().account_type).await {
        Ok(_) => Ok(redirect("/customer/dashboard")),
        Err(error) => match services::load_customer_dashboard(&data.db, customer_id).await {
            Ok(dashboard) => {
                let full_name = customer_repository::get_customer_by_id(&data.db, &customer_id)
                    .await
                    .map(|customer| customer.full_name)
                    .unwrap_or_else(|_| user.username.clone());
                let account_state = savings_application_state(&dashboard.accounts);
                let limit_summary = load_dashboard_limit_summary(&data, customer_id).await;

                render(DashboardTemplate {
                    full_name,
                    accounts: dashboard.accounts.clone(),
                    has_accounts: !dashboard.accounts.is_empty(),
                    can_apply_everyday_savings: account_state.can_apply_everyday_savings,
                    can_apply_high_yield_savings: account_state.can_apply_high_yield_savings,
                    account_application_notice: account_state.notice,
                    has_account_application_notice: account_state.has_notice,
                    daily_limit_display: limit_summary.daily_limit_display,
                    outgoing_today_display: limit_summary.outgoing_today_display,
                    remaining_today_display: limit_summary.remaining_today_display,
                    create_account_error: error,
                    has_create_account_error: true,
                })
            }
            Err(message) => render_error("Account creation failed", message),
        },
    }
}

// Renders the deposit page screen with data prepared by the service layer.
pub async fn deposit_page(data: web::Data<AppState>, session: Session) -> Result<HttpResponse> {
    let user = match require_customer(&data, &session).await {
        Ok(user) => user,
        Err(response) => return Ok(response),
    };
    let customer_id = user.customer_id_or_nil();

    let accounts = match services::list_active_customer_products(&data.db, customer_id).await {
        Ok(accounts) if !accounts.is_empty() => accounts,
        _ => return render_error("Account unavailable", "No active bank account was found.".to_string()),
    };

    let account = accounts[0].clone();
    render(DepositTemplate {
        accounts,
        selected_account_number: account.account_number.clone(),
        balance: display_money_without_symbol(account.balance_display()),
        error: String::new(),
        has_error: false,
        success: String::new(),
        has_success: false,
    })
}

// Handles the deposit form action and redirects after the service result.
pub async fn deposit(
    app_state: web::Data<AppState>,
    session: Session,
    form: web::Form<DepositForm>,
) -> Result<HttpResponse> {
    let user = match require_customer(&app_state, &session).await {
        Ok(user) => user,
        Err(response) => return Ok(response),
    };
    let customer_id = user.customer_id_or_nil();

    let form_data = form.into_inner();
    let selected_account_number = form_data.account_number.clone();

    match services::deposit(&app_state, customer_id, form_data).await {
        Ok(account) => {
            let accounts = services::list_active_customer_products(&app_state.db, customer_id)
                .await
                .unwrap_or_else(|_| vec![account.clone()]);
            render(DepositTemplate {
                accounts,
                selected_account_number: account.account_number.clone(),
                balance: display_money_without_symbol(account.balance_display()),
                error: String::new(),
                has_error: false,
                success: "Deposit completed successfully.".to_string(),
                has_success: true,
            })
        }
        Err(error) => {
            let accounts = match services::list_active_customer_products(&app_state.db, customer_id).await {
                Ok(accounts) if !accounts.is_empty() => accounts,
                _ => return render_error("Account unavailable", "No active bank account was found.".to_string()),
            };
            let selected = accounts
                .iter()
                .find(|account| account.account_number == selected_account_number)
                .cloned()
                .unwrap_or_else(|| accounts[0].clone());

            render(DepositTemplate {
                accounts,
                selected_account_number,
                balance: display_money_without_symbol(selected.balance_display()),
                error,
                has_error: true,
                success: String::new(),
                has_success: false,
            })
        }
    }
}

// Renders the transfer page screen with data prepared by the service layer.
pub async fn transfer_page(data: web::Data<AppState>, session: Session) -> Result<HttpResponse> {
    let user = match require_customer(&data, &session).await {
        Ok(user) => user,
        Err(response) => return Ok(response),
    };
    let customer_id = user.customer_id_or_nil();

    let accounts = match services::list_active_customer_products(&data.db, customer_id).await {
        Ok(accounts) if !accounts.is_empty() => accounts,
        _ => return render_error("Account unavailable", "No active bank account was found.".to_string()),
    };

    let account = accounts[0].clone();
    render(TransferTemplate {
        accounts,
        selected_account_number: account.account_number.clone(),
        balance: display_money_without_symbol(account.balance_display()),
        error: String::new(),
        has_error: false,
    })
}

// Handles the transfer form action and redirects after the service result.
pub async fn transfer(
    app_state: web::Data<AppState>,
    session: Session,
    form: web::Form<TransferForm>,
) -> Result<HttpResponse> {
    let user = match require_customer(&app_state, &session).await {
        Ok(user) => user,
        Err(response) => return Ok(response),
    };
    let customer_id = user.customer_id_or_nil();

    let form_data = form.into_inner();
    let selected_account_number = form_data.account_number.clone();

    match services::transfer(&app_state, customer_id, form_data).await {
        Ok(true) => Ok(redirect("/customer/transactions")),
        Ok(false) => render_error("Transfer failed", "Transfer failed due to an unknown rule.".to_string()),
        Err(error) => {
            let accounts = match services::list_active_customer_products(&app_state.db, customer_id).await {
                Ok(accounts) if !accounts.is_empty() => accounts,
                _ => return render_error("Account unavailable", "No active bank account was found.".to_string()),
            };
            let selected = accounts
                .iter()
                .find(|account| account.account_number == selected_account_number)
                .cloned()
                .unwrap_or_else(|| accounts[0].clone());

            render(TransferTemplate {
                accounts,
                selected_account_number,
                balance: display_money_without_symbol(selected.balance_display()),
                error,
                has_error: true,
            })
        }
    }
}

// Handles the transactions request.
pub async fn transactions(data: web::Data<AppState>, session: Session) -> Result<HttpResponse> {
    let user = match require_customer(&data, &session).await {
        Ok(user) => user,
        Err(response) => return Ok(response),
    };

    match services::list_transactions(&data.db, user.customer_id_or_nil()).await {
        Ok(transactions) => render(TransactionsTemplate {
            has_transactions: !transactions.is_empty(),
            transactions,
        }),
        Err(message) => render_error("Transactions unavailable", message),
    }
}

// Renders the statements page screen with data prepared by the service layer.
pub async fn statements_page(
    data: web::Data<AppState>,
    session: Session,
    query: web::Query<StatementRequest>,
) -> Result<HttpResponse> {
    let user = match require_customer(&data, &session).await {
        Ok(user) => user,
        Err(response) => return Ok(response),
    };

    match services::load_statement_page(&data.db, user.customer_id_or_nil(), query.into_inner()).await {
        Ok(page) => render(StatementTemplate {
            has_accounts: !page.accounts.is_empty(),
            selected_account_id: page.selected_account_id,
            start_date: page.start_date,
            end_date: page.end_date,
            has_transactions: !page.transactions.is_empty(),
            transactions: page.transactions,
            accounts: page.accounts,
            has_error: !page.error.is_empty(),
            error: page.error,
        }),
        Err(message) => render_error("Statements unavailable", message),
    }
}

// Handles the download statement pdf request.
pub async fn download_statement_pdf(
    data: web::Data<AppState>,
    session: Session,
    query: web::Query<StatementRequest>,
) -> Result<HttpResponse> {
    let user = match require_customer(&data, &session).await {
        Ok(user) => user,
        Err(response) => return Ok(response),
    };

    match services::build_bank_statement(&data.db, user.customer_id_or_nil(), query.into_inner()).await {
        Ok(statement) => {
            let filename = services::statement_pdf_filename(&statement);
            let pdf = services::render_statement_pdf(&statement);
            Ok(HttpResponse::Ok()
                .append_header(("Content-Type", "application/pdf"))
                .append_header(("Content-Disposition", format!("attachment; filename=\"{}\"", filename)))
                .body(pdf))
        }
        Err(message) => render_error("Statement download failed", message),
    }
}

// Handles the loan activity request.
pub async fn loan_activity(data: web::Data<AppState>, session: Session) -> Result<HttpResponse> {
    let user = match require_customer(&data, &session).await {
        Ok(user) => user,
        Err(response) => return Ok(response),
    };

    match services::list_loan_activity(&data.db, user.customer_id_or_nil()).await {
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

// Handles the fixed deposit activity request.
pub async fn fixed_deposit_activity(
    data: web::Data<AppState>,
    session: Session,
) -> Result<HttpResponse> {
    let user = match require_customer(&data, &session).await {
        Ok(user) => user,
        Err(response) => return Ok(response),
    };

    match services::list_fixed_deposit_activity(&data.db, user.customer_id_or_nil()).await {
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

// Renders the cards page screen with data prepared by the service layer.
pub async fn cards_page(data: web::Data<AppState>, session: Session) -> Result<HttpResponse> {
    let user = match require_customer(&data, &session).await {
        Ok(user) => user,
        Err(response) => return Ok(response),
    };
    let customer_id = user.customer_id_or_nil();

    match services::load_card_dashboard(&data.db, customer_id).await {
        Ok(page) => render(CardDashboardTemplate {
            cards: page.cards,
            has_cards: page.has_cards,
            accounts: page.accounts,
            has_accounts: page.has_accounts,
            error: String::new(),
            has_error: false,
            success: String::new(),
            has_success: false,
        }),
        Err(error) => render_error("Cards unavailable", error),
    }
}

// Handles the create card form action and redirects after the service result.
pub async fn create_card(
    data: web::Data<AppState>,
    session: Session,
    form: web::Form<CardApplicationForm>,
) -> Result<HttpResponse> {
    let user = match require_customer(&data, &session).await {
        Ok(user) => user,
        Err(response) => return Ok(response),
    };
    let customer_id = user.customer_id_or_nil();

    match services::create_card(&data.db, customer_id, form.into_inner()).await {
        Ok(_) => Ok(redirect("/customer/cards")),
        Err(error) => match services::load_card_dashboard(&data.db, customer_id).await {
            Ok(page) => render(CardDashboardTemplate {
                cards: page.cards,
                has_cards: page.has_cards,
                accounts: page.accounts,
                has_accounts: page.has_accounts,
                error,
                has_error: true,
                success: String::new(),
                has_success: false,
            }),
            Err(message) => render_error("Cards unavailable", message),
        },
    }
}

// Handles the freeze card form action and redirects after the service result.
pub async fn freeze_card(
    data: web::Data<AppState>,
    session: Session,
    path: web::Path<String>,
) -> Result<HttpResponse> {
    let user = match require_customer(&data, &session).await {
        Ok(user) => user,
        Err(response) => return Ok(response),
    };
    let card_id = match uuid::Uuid::parse_str(&path.into_inner()) {
        Ok(value) => value,
        Err(_) => return Ok(redirect("/customer/cards")),
    };
    let _ = services::set_card_status(&data.db, user.customer_id_or_nil(), card_id, "frozen").await;
    Ok(redirect("/customer/cards"))
}

// Handles the activate card form action and redirects after the service result.
pub async fn activate_card(
    data: web::Data<AppState>,
    session: Session,
    path: web::Path<String>,
) -> Result<HttpResponse> {
    let user = match require_customer(&data, &session).await {
        Ok(user) => user,
        Err(response) => return Ok(response),
    };
    let card_id = match uuid::Uuid::parse_str(&path.into_inner()) {
        Ok(value) => value,
        Err(_) => return Ok(redirect("/customer/cards")),
    };
    let _ = services::set_card_status(&data.db, user.customer_id_or_nil(), card_id, "active").await;
    Ok(redirect("/customer/cards"))
}

// Handles the render paynow dashboard request.
async fn render_paynow_dashboard(
    data: &web::Data<AppState>,
    customer_id: uuid::Uuid,
    error: String,
    success: String,
) -> Result<HttpResponse> {
    match services::load_paynow_dashboard(&data.db, customer_id).await {
        Ok(page) => render(PayNowTemplate {
            accounts: page.accounts.clone(),
            has_accounts: !page.accounts.is_empty(),
            registrations: page.registrations.clone(),
            has_registrations: !page.registrations.is_empty(),
            error: error.clone(),
            has_error: !error.is_empty(),
            success: success.clone(),
            has_success: !success.is_empty(),
        }),
        Err(message) => render_error("PayNow unavailable", message),
    }
}

// Renders the paynow page screen with data prepared by the service layer.
pub async fn paynow_page(data: web::Data<AppState>, session: Session) -> Result<HttpResponse> {
    let user = match require_customer(&data, &session).await {
        Ok(user) => user,
        Err(response) => return Ok(response),
    };

    render_paynow_dashboard(&data, user.customer_id_or_nil(), String::new(), String::new()).await
}

// Handles the register paynow form action and redirects after the service result.
pub async fn register_paynow(
    data: web::Data<AppState>,
    session: Session,
    form: web::Form<PayNowRegisterForm>,
) -> Result<HttpResponse> {
    let user = match require_customer(&data, &session).await {
        Ok(user) => user,
        Err(response) => return Ok(response),
    };
    let customer_id = user.customer_id_or_nil();

    match services::register_paynow(&data.db, customer_id, form.into_inner()).await {
        Ok(()) => {
            render_paynow_dashboard(
                &data,
                customer_id,
                String::new(),
                "PayNow registration completed successfully.".to_string(),
            )
            .await
        }
        Err(error) => render_paynow_dashboard(&data, customer_id, error, String::new()).await,
    }
}

// Handles the transfer paynow form action and redirects after the service result.
pub async fn transfer_paynow(
    data: web::Data<AppState>,
    session: Session,
    form: web::Form<PayNowTransferForm>,
) -> Result<HttpResponse> {
    let user = match require_customer(&data, &session).await {
        Ok(user) => user,
        Err(response) => return Ok(response),
    };
    let customer_id = user.customer_id_or_nil();

    match services::transfer_paynow(&data.db, customer_id, form.into_inner()).await {
        Ok(()) => {
            render_paynow_dashboard(
                &data,
                customer_id,
                String::new(),
                "PayNow transfer completed successfully.".to_string(),
            )
            .await
        }
        Err(error) => render_paynow_dashboard(&data, customer_id, error, String::new()).await,
    }
}

// Renders the profile page screen with data prepared by the service layer.
pub async fn profile_page(data: web::Data<AppState>, session: Session) -> Result<HttpResponse> {
    let user = match require_customer(&data, &session).await {
        Ok(user) => user,
        Err(response) => return Ok(response),
    };
    let customer_id = user.customer_id_or_nil();

    let customer = match customer_repository::get_customer_by_id(&data.db, &customer_id).await {
        Ok(customer) => customer,
        Err(_) => return render_error("Profile unavailable", "Could not load your customer profile.".to_string()),
    };

    let paynow_dashboard = match services::load_paynow_dashboard(&data.db, customer_id).await {
        Ok(page) => page,
        Err(_) => services::PayNowDashboard {
            accounts: vec![],
            registrations: vec![],
        },
    };

    let active_paynow = paynow_dashboard
        .registrations
        .iter()
        .find(|registration| registration.status == "active" && registration.paynow_type == "phone_number");

    let date_of_birth = customer.date_of_birth_display();
    let last_login = user.last_login_display();
    let paynow_id = active_paynow
        .map(|registration| registration.paynow_id.clone())
        .unwrap_or_default();
    let paynow_linked_product_id = active_paynow
        .map(|registration| registration.linked_account_id.to_string())
        .unwrap_or_default();
    let has_paynow = !paynow_id.is_empty();
    let accounts = paynow_dashboard.accounts;
    let has_accounts = !accounts.is_empty();

    render(ProfileTemplate {
        full_name: customer.full_name,
        email: customer.email,
        phone: customer.phone_number,
        date_of_birth,
        last_login,
        accounts,
        has_accounts,
        paynow_id,
        paynow_linked_product_id,
        has_paynow,
    })
}

// Handles the update profile form action and redirects after the service result.
pub async fn update_profile(
    data: web::Data<AppState>,
    session: Session,
    form: web::Form<ProfileForm>,
) -> Result<HttpResponse> {
    let user = match require_customer(&data, &session).await {
        Ok(user) => user,
        Err(response) => return Ok(response),
    };
    let customer_id = user.customer_id_or_nil();

    match services::update_customer_profile(&data.db, customer_id, user.id, form.into_inner()).await {
        Ok(_) => Ok(redirect("/customer/profile")),
        Err(error) => render_error("Profile update failed", error),
    }
}

// Handles the render transaction controls dashboard request.
async fn render_transaction_controls_dashboard(
    data: &web::Data<AppState>,
    customer_id: uuid::Uuid,
    error: String,
    success: String,
) -> Result<HttpResponse> {
    match services::load_transaction_controls_page(&data.db, customer_id).await {
        Ok(page) => render(TransactionControlsTemplate {
            controls: page.controls,
            alerts: page.alerts.clone(),
            has_alerts: !page.alerts.is_empty(),
            error: error.clone(),
            has_error: !error.is_empty(),
            success: success.clone(),
            has_success: !success.is_empty(),
        }),
        Err(message) => render_error("Transaction controls unavailable", message),
    }
}

// Renders the transaction controls page screen with data prepared by the service layer.
pub async fn transaction_controls_page(data: web::Data<AppState>, session: Session) -> Result<HttpResponse> {
    let user = match require_customer(&data, &session).await {
        Ok(user) => user,
        Err(response) => return Ok(response),
    };

    render_transaction_controls_dashboard(&data, user.customer_id_or_nil(), String::new(), String::new()).await
}

// Handles the update transaction limit form action and redirects after the service result.
pub async fn update_transaction_limit(
    data: web::Data<AppState>,
    session: Session,
    form: web::Form<TransactionLimitForm>,
) -> Result<HttpResponse> {
    let user = match require_customer(&data, &session).await {
        Ok(user) => user,
        Err(response) => return Ok(response),
    };
    let customer_id = user.customer_id_or_nil();

    match services::update_daily_transaction_limit(&data.db, customer_id, form.into_inner()).await {
        Ok(controls) => {
            let message = format!(
                "Daily transaction limit updated to {}. You can change it again after the 24-hour cooldown.",
                controls.daily_limit_display()
            );
            render_transaction_controls_dashboard(&data, customer_id, String::new(), message).await
        }
        Err(error) => render_transaction_controls_dashboard(&data, customer_id, error, String::new()).await,
    }
}

// Handles the update money lock form action and redirects after the service result.
pub async fn update_money_lock(
    data: web::Data<AppState>,
    session: Session,
    form: web::Form<MoneyLockForm>,
) -> Result<HttpResponse> {
    let user = match require_customer(&data, &session).await {
        Ok(user) => user,
        Err(response) => return Ok(response),
    };
    let customer_id = user.customer_id_or_nil();

    let form_data = form.into_inner();
    let action = form_data.action.clone();
    match services::update_money_lock(&data.db, customer_id, form_data).await {
        Ok(_) => {
            let message = if action == "enable" {
                "Money Lock enabled. Outgoing transfers are now blocked.".to_string()
            } else {
                "Money Lock unlocked. Outgoing transfers are available again.".to_string()
            };
            render_transaction_controls_dashboard(&data, customer_id, String::new(), message).await
        }
        Err(error) => render_transaction_controls_dashboard(&data, customer_id, error, String::new()).await,
    }
}

// Handles the render giro dashboard request.
async fn render_giro_dashboard(
    data: &web::Data<AppState>,
    customer_id: uuid::Uuid,
    error: String,
    success: String,
) -> Result<HttpResponse> {
    match services::load_giro_dashboard(&data.db, customer_id).await {
        Ok(page) => render(GiroTemplate {
            accounts: page.accounts.clone(),
            has_accounts: !page.accounts.is_empty(),
            arrangements: page.arrangements.clone(),
            has_arrangements: !page.arrangements.is_empty(),
            error: error.clone(),
            has_error: !error.is_empty(),
            success: success.clone(),
            has_success: !success.is_empty(),
        }),
        Err(message) => render_error("GIRO unavailable", message),
    }
}

// Renders the giro page screen with data prepared by the service layer.
pub async fn giro_page(data: web::Data<AppState>, session: Session) -> Result<HttpResponse> {
    let user = match require_customer(&data, &session).await {
        Ok(user) => user,
        Err(response) => return Ok(response),
    };

    render_giro_dashboard(&data, user.customer_id_or_nil(), String::new(), String::new()).await
}

// Handles the create giro arrangement form action and redirects after the service result.
pub async fn create_giro_arrangement(
    data: web::Data<AppState>,
    session: Session,
    form: web::Form<GiroArrangementForm>,
) -> Result<HttpResponse> {
    let user = match require_customer(&data, &session).await {
        Ok(user) => user,
        Err(response) => return Ok(response),
    };
    let customer_id = user.customer_id_or_nil();

    match services::create_giro_arrangement(&data.db, customer_id, form.into_inner()).await {
        Ok(()) => render_giro_dashboard(&data, customer_id, String::new(), "GIRO arrangement created successfully.".to_string()).await,
        Err(error) => render_giro_dashboard(&data, customer_id, error, String::new()).await,
    }
}

// Handles the cancel giro arrangement form action and redirects after the service result.
pub async fn cancel_giro_arrangement(
    data: web::Data<AppState>,
    session: Session,
    path: web::Path<String>,
) -> Result<HttpResponse> {
    let user = match require_customer(&data, &session).await {
        Ok(user) => user,
        Err(response) => return Ok(response),
    };
    let customer_id = user.customer_id_or_nil();
    let arrangement_id = match uuid::Uuid::parse_str(&path.into_inner()) {
        Ok(value) => value,
        Err(_) => return render_giro_dashboard(&data, customer_id, "Invalid GIRO arrangement selected.".to_string(), String::new()).await,
    };

    match services::cancel_giro_arrangement(&data.db, customer_id, arrangement_id).await {
        Ok(()) => render_giro_dashboard(&data, customer_id, String::new(), "GIRO arrangement cancelled.".to_string()).await,
        Err(error) => render_giro_dashboard(&data, customer_id, error, String::new()).await,
    }
}

