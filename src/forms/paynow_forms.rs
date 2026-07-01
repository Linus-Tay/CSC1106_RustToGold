use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct PayNowRegisterForm {
    pub paynow_type: String,
    pub paynow_id: String,
    pub linked_product_id: String,
}

#[derive(Debug, Deserialize)]
pub struct PayNowTransferForm {
    pub from_product_id: String,
    pub recipient_type: String,
    pub recipient_id: String,
    pub amount: String,
    pub note: String,
}
