use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct LoanApplicationForm {
    pub amount: String,
    pub purpose: String,
    pub term_months: i32,
}

#[derive(Debug, Deserialize)]
pub struct LoanPaymentForm {
    pub amount: String,
}
