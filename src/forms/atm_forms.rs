// Form layer: request payload structs received from HTML forms.

use serde::Deserialize;

#[derive(Debug, Deserialize)]
// Form payload for the LoginForm request.
pub struct CardInsertionForm {
    pub card_number: String
}


#[derive(Debug, Deserialize)]
// Form payload for the LoginForm request.
pub struct PinValidationForm {
    pub pin: String
}

#[derive(Debug, Deserialize)]
// Form payload for the LoginForm request.
pub struct ATMDepositForm {
    pub amount: String
}

pub struct ATMWithdrawalForm {
    pub amount: String
}