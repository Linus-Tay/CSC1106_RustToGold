use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct CardInsertionForm {
    pub card_number: String
}

#[derive(Debug, Deserialize)]
pub struct PinValidationForm {
    pub pin: String
}

#[derive(Debug, Deserialize)]
pub struct ATMDepositForm {
    pub amount: String
}

pub struct ATMWithdrawalForm {
    pub amount: String
}
