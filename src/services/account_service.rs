use crate::forms::auth_forms::AccountCreationForm;
use crate::models::{AccountWorkflow, BankAccount, Customer, Money, Transaction, User};
use crate::repositories::customer_repository::NewCustomer;
use crate::repositories::{account_repository, customer_repository, transaction_repository, user_repository};
use crate::services::support::clean_optional_text;
use crate::AppState;
use chrono::NaiveDate;
use sqlx::PgPool;
use uuid::Uuid;

pub async fn load_customer_dashboard(
    db: &PgPool,
    user_id: Uuid,
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

pub async fn list_transactions(db: &PgPool, user_id: Uuid) -> Result<Vec<Transaction>, String> {
    transaction_repository::find_recent_transactions_by_user_id(db, user_id, 50)
        .await
        .map_err(|_| "Could not load transaction history.".to_string())
}

// pub async fn deposit(app_state: &AppState, user_id: i64, form: DepositForm) -> Result<BankAccount, String> {
//     let amount = Money::parse_dollars(&form.amount)?;
//     let description = clean_optional_text(&form.description);
//     let current_account = account_repository::find_primary_account_by_user_id(&app_state.db, user_id)
//         .await
//         .map_err(|_| "Could not load your bank account.".to_string())?
//         .ok_or_else(|| "No bank account was found for this customer.".to_string())?;

//     if !current_account.is_open_for_customer_actions() {
//         return Err("This account is not open for deposits.".to_string());
//     }

//     if current_account.projected_balance_after_deposit(amount).is_none() {
//         return Err("This deposit cannot be applied to the account.".to_string());
//     }

//     let _guard = app_state.account_mutex.lock().await;

//     let (updated_account, _) = account_repository::deposit_to_primary_account(
//         &app_state.db,
//         user_id,
//         amount.cents(),
//         description.as_deref(),
//     )
//     .await
//     .map_err(|_| "Deposit failed. Please try again.".to_string())?;

//     Ok(updated_account)
// }
