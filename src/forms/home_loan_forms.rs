use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct HomeLoanApplicationForm {
    pub amount: String,
    pub home_loan_years: String,
    pub house_type: String,
}

#[derive(Debug, Deserialize, Default)]
pub struct HomeLoanPaymentForm {
    pub amount: Option<String>,
}


