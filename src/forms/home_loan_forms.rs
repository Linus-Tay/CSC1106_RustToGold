// Form layer: request payload structs received from HTML forms.

use serde::Deserialize;

#[derive(Debug, Deserialize)]
// Form payload for the HomeLoanApplicationForm request.
pub struct HomeLoanApplicationForm {
    pub property_type: String,
    pub property_value: String,
    #[serde(default)]
    pub down_payment: String,
    pub term_years: i32,
    pub account_number: String,
}

#[derive(Debug, Deserialize)]
// Form payload for the HomeLoanPaymentForm request.
pub struct HomeLoanPaymentForm {
    pub amount: String,
    pub account_number: String,
}
