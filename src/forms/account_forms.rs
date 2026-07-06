use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct TransferForm {
    pub amount: String,
    pub method: String,
    pub note: String,
    pub account_number: String,
    pub recipient_info: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateBankAccountForm {
    pub account_type: String,
    pub nickname: String,
}
