
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct LoanApplicationForm {
    pub amount: String,
    pub purpose: String,
    pub term_months: i32,
    pub account_number: String,
}

#[derive(Debug, Deserialize)]
pub struct LoanPaymentForm {
    pub amount: String,
    pub account_number: String,
}
