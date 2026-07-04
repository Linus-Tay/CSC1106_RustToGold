use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct TransactionLimitForm {
    pub daily_limit: String,
}

#[derive(Debug, Deserialize)]
pub struct MoneyLockForm {
    pub action: String,
}
