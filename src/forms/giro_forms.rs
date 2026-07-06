
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct GiroArrangementForm {
    pub from_product_id: String,
    pub payee_name: String,
    pub recipient_account_number: String,
    pub amount: String,
    pub frequency: String,
    pub start_date: String,
    pub end_date: String,
    pub note: String,
}
