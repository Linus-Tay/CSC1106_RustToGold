use crate::models::{Product, Transaction};
use crate::repositories::{loan_repository, transaction_repository, user_repository};
use sqlx::PgPool;

pub async fn load_customer_dashboard(db: &PgPool, user_id: i64) -> Result<Product, String> {
    let user = user_repository::find_user_by_id(db, user_id)
        .await
        .map_err(|_| "Could not load your customer profile.".to_string())?
        .ok_or_else(|| "No online banking profile was found.".to_string())?;

    loan_repository::find_primary_active_product(db, user.customer_id)
        .await
        .map_err(|_| "Could not load your active customer product account.".to_string())
}

pub async fn list_transactions(db: &PgPool, user_id: i64) -> Result<Vec<Transaction>, String> {
    transaction_repository::find_customer_cash_transactions_by_user_id(db, user_id, 50)
        .await
        .map_err(|_| "Could not load deposit and transfer history.".to_string())
}

pub async fn list_loan_activity(db: &PgPool, user_id: i64) -> Result<Vec<Transaction>, String> {
    transaction_repository::find_customer_loan_transactions_by_user_id(db, user_id, 50)
        .await
        .map_err(|_| "Could not load loan activity.".to_string())
}

pub async fn list_fixed_deposit_activity(
    db: &PgPool,
    user_id: i64,
) -> Result<Vec<Transaction>, String> {
    transaction_repository::find_customer_fixed_deposit_transactions_by_user_id(db, user_id, 50)
        .await
        .map_err(|_| "Could not load fixed deposit activity.".to_string())
}
