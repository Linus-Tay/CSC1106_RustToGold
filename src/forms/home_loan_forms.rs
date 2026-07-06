
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct HomeLoanApplicationForm {
    pub property_type: String,
    pub property_value: String,
    #[serde(default)]
    pub down_payment: String,
    pub term_years: i32,
    pub account_number: String,
}

#[derive(Debug, Deserialize)]
pub struct HomeLoanPaymentForm {
    pub amount: String,
    pub account_number: String,
}
