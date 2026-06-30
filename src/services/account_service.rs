use crate::models::{Product, Transaction};
use crate::repositories::{product_repository, transaction_repository};
use sqlx::PgPool;
use uuid::Uuid;

pub struct CustomerDashboardData {
    pub primary_account: Product,
    pub accounts: Vec<Product>,
}

pub async fn load_customer_dashboard(
    db: &PgPool,
    customer_id: Uuid,
) -> Result<CustomerDashboardData, String> {
    let accounts = product_repository::list_products_by_customer(db, &customer_id)
        .await
        .map_err(|_| "Could not load your bank accounts.".to_string())?;

    let primary_account = accounts
        .iter()
        .find(|account| account.status == "active")
        .cloned()
        .ok_or_else(|| "No active customer product account was found.".to_string())?;

    Ok(CustomerDashboardData {
        primary_account,
        accounts,
    })
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
