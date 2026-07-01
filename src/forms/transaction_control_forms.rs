// Form layer: request payload structs received from HTML forms.

use serde::Deserialize;

#[derive(Debug, Deserialize)]
// Form payload for the TransactionLimitForm request.
pub struct TransactionLimitForm {
    pub daily_limit: String,
}

#[derive(Debug, Deserialize)]
// Form payload for the MoneyLockForm request.
pub struct MoneyLockForm {
    pub action: String,
}
