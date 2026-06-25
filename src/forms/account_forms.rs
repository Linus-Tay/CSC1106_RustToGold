use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct DepositForm {
    pub amount: String,
    pub description: String,
    pub account_number: String
}
