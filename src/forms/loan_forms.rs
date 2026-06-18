use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct LoanApplicationForm {
    pub amount: String,
    pub term_months: String,
}

#[derive(Debug, Deserialize)]
pub struct LoanPaymentForm {
    pub amount: Option<String>,
}