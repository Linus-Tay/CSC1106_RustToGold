use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct HomeLoanApplicationForm {
    pub property_type: String,
    pub property_value: String,
    pub down_payment: String,
    pub term_years: i32,
}

#[derive(Debug, Deserialize)]
pub struct HomeLoanPaymentForm {
    pub amount: String,
}
