use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct HomeLoanApplicationForm {
    pub house_type: String,
    pub amount: String,
    pub term_years: String,
}
