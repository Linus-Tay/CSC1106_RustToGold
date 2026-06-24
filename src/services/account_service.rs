use crate::forms::DepositForm;
use crate::models::{AccountWorkflow, BankAccount, Money, Transaction, BankAccountWithUser};
use crate::repositories::{account_repository, transaction_repository, admin_transaction_repository};
use crate::services::support::clean_optional_text;
use crate::services::audit_log_service::{self, AuditContext};
use sqlx::PgPool;

const ACCOUNTS_PAGE_SIZE: i64 = 25;

pub async fn load_customer_dashboard(
    db: &PgPool,
    user_id: i64,
) -> Result<(BankAccount, Vec<Transaction>), String> {
    let account = account_repository::find_primary_account_by_user_id(db, user_id)
        .await
        .map_err(|_| "Could not load your bank account.".to_string())?
        .ok_or_else(|| "No bank account was found for this customer.".to_string())?;

    let transactions = transaction_repository::find_recent_transactions_by_user_id(db, user_id, 5)
        .await
        .map_err(|_| "Could not load recent transactions.".to_string())?;

    Ok((account, transactions))
}

pub async fn list_transactions(db: &PgPool, user_id: i64) -> Result<Vec<Transaction>, String> {
    transaction_repository::find_recent_transactions_by_user_id(db, user_id, 50)
        .await
        .map_err(|_| "Could not load transaction history.".to_string())
}

pub async fn deposit(db: &PgPool, user_id: i64, form: DepositForm) -> Result<BankAccount, String> {
    let amount = Money::parse_dollars(&form.amount)?;
    let description = clean_optional_text(&form.description);
    let current_account = account_repository::find_primary_account_by_user_id(db, user_id)
        .await
        .map_err(|_| "Could not load your bank account.".to_string())?
        .ok_or_else(|| "No bank account was found for this customer.".to_string())?;

    if !current_account.is_open_for_customer_actions() {
        return Err("This account is not open for deposits.".to_string());
    }

    if current_account.projected_balance_after_deposit(amount).is_none() {
        return Err("This deposit cannot be applied to the account.".to_string());
    }

    let (updated_account, _) = account_repository::deposit_to_primary_account(
        db,
        user_id,
        amount.cents(),
        description.as_deref(),
    )
    .await
    .map_err(|_| "Deposit failed. Please try again.".to_string())?;

    Ok(updated_account)
}

const ADMIN_PAGE_SIZE: i64 = 25;

pub struct AdminTransactionPage {
    pub transactions: Vec<Transaction>,
    pub total_count: i64,
    pub page: i64,
    pub total_pages: i64,
    pub has_transactions: bool,
}

pub async fn load_admin_transactions(
    db: &PgPool,
    transaction_type: Option<String>,
    user_id: Option<String>,
    account_id: Option<String>,
    page: i64,
) -> Result<AdminTransactionPage, String> {
    let page = page.max(1);
    let offset = (page - 1) * ADMIN_PAGE_SIZE;

    let transaction_type = none_if_blank(transaction_type);
    let user_id = user_id.as_deref().and_then(|s| s.trim().parse::<i64>().ok());
    let account_id = account_id.as_deref().and_then(|s| s.trim().parse::<i64>().ok());

    let transactions = admin_transaction_repository::admin_list_transactions(
        db,
        transaction_type.as_deref(),
        user_id,
        account_id,
        ADMIN_PAGE_SIZE,
        offset,
    )
    .await
    .map_err(|err| {
        eprintln!("[account_service] admin_list_transactions failed: {err:?}");
        "Could not load transactions.".to_string()
    })?;

    let total_count = admin_transaction_repository::admin_count_transactions(
        db,
        transaction_type.as_deref(),
        user_id,
        account_id,
    )
    .await
    .map_err(|err| {
        eprintln!("[account_service] admin_count_transactions failed: {err:?}");
        "Could not count transactions.".to_string()
    })?;

    let total_pages = ((total_count as f64) / (ADMIN_PAGE_SIZE as f64)).ceil().max(1.0) as i64;

    Ok(AdminTransactionPage {
        has_transactions: !transactions.is_empty(),
        transactions,
        total_count,
        page,
        total_pages,
    })
}

fn none_if_blank(value: Option<String>) -> Option<String> {
    value.and_then(|s| if s.trim().is_empty() { None } else { Some(s.trim().to_string()) })
}
 
#[derive(Debug, Clone)]
pub struct AdminAccountPage {
    pub accounts: Vec<BankAccount>,
    pub total_count: i64,
    pub page: i64,
    pub total_pages: i64,
    pub has_accounts: bool,
}
 
pub async fn load_admin_accounts(
    db: &PgPool,
    status: Option<String>,
    page: i64,
) -> Result<AdminAccountPage, String> {
    let page = page.max(1);
    let offset = (page - 1) * ACCOUNTS_PAGE_SIZE;
    let status = status.and_then(|s| if s.trim().is_empty() { None } else { Some(s.trim().to_string()) });
 
    let accounts = account_repository::list_accounts_with_users(
        db,
        status.as_deref(),
        ACCOUNTS_PAGE_SIZE,
        offset,
    )
    .await
    .map_err(|err| {
        eprintln!("[account_service] list_accounts_with_users failed: {err:?}");
        "Could not load accounts.".to_string()
    })?;
 
    let total_count = account_repository::count_accounts(db, status.as_deref())
        .await
        .map_err(|err| {
            eprintln!("[account_service] count_accounts failed: {err:?}");
            "Could not count accounts.".to_string()
        })?;
 
    let total_pages = ((total_count as f64) / (ACCOUNTS_PAGE_SIZE as f64)).ceil().max(1.0) as i64;
 
    Ok(AdminAccountPage {
        has_accounts: !accounts.is_empty(),
        accounts,
        total_count,
        page,
        total_pages,
    })
}
 
pub async fn update_account_status(
    db: &PgPool,
    ctx: &AuditContext,
    account_id: i64,
    new_status: &str,
) -> Result<(), String> {
    // Validate status
    let new_status = match new_status {
        "active" | "frozen" | "closed" | "pending" => new_status,
        _ => return Err("Invalid account status.".to_string()),
    };
 
    // Fetch before state for audit log
    let before = account_repository::find_primary_account_by_user_id(db, account_id)
        .await
        .map_err(|_| "Could not load account.".to_string())?
        .ok_or_else(|| "Account not found.".to_string())?;
 
    let audit_action = match new_status {
        "active"  => "approve_account",
        "frozen"  => "freeze_account",
        "closed"  => "close_account",
        _         => "freeze_account",
    };
 
    match account_repository::update_account_status(db, account_id, new_status).await {
        Ok(after) => {
            audit_log_service::record(
                db, ctx,
                audit_action, "bank_account", Some(account_id),
                Some(&before), Some(&after),
                "success",
            ).await;
            Ok(())
        }
        Err(err) => {
            eprintln!("[account_service] update_account_status failed: {err:?}");
            audit_log_service::record_simple(
                db, ctx,
                audit_action, "bank_account", Some(account_id),
                "failed",
            ).await;
            Err("Could not update account status.".to_string())
        }
    }
}