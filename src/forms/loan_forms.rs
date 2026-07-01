// Form layer: request payload structs received from HTML forms.

use serde::Deserialize;

#[derive(Debug, Deserialize)]
// Form payload for the LoanApplicationForm request.
pub struct LoanApplicationForm {
    pub amount: String,
    pub purpose: String,
    pub term_months: i32,
    pub account_number: String,
}

#[derive(Debug, Deserialize)]
// Form payload for the LoanPaymentForm request.
pub struct LoanPaymentForm {
    pub amount: String,
    pub account_number: String,
}
