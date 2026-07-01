// Form layer: request payload structs received from HTML forms.

use serde::Deserialize;

#[derive(Debug, Deserialize)]
// Form payload for the DepositForm request.
pub struct DepositForm {
    pub amount: String,
    pub description: String,
    pub account_number: String,
}

#[derive(Debug, Deserialize)]
// Form payload for the TransferForm request.
pub struct TransferForm {
    pub amount: String,
    pub method: String,
    pub note: String,
    pub account_number: String,
    pub recipient_info: String,
}

#[derive(Debug, Deserialize)]
// Form payload for the CreateBankAccountForm request.
pub struct CreateBankAccountForm {
    pub account_type: String,
    pub nickname: String,
}
