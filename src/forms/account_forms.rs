use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct DepositForm {
    pub amount: String,
    pub description: String,
    pub account_number: String
}

#[derive(Debug, Deserialize)]
pub struct TransferForm {
    pub amount: String,
    pub method: String,
    pub recipient_info: String   
}