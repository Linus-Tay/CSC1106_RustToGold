// Form layer: request payload structs received from HTML forms.

use serde::Deserialize;

#[derive(Debug, Deserialize)]
// Form payload for the PayNowRegisterForm request.
pub struct PayNowRegisterForm {
    pub paynow_type: String,
    pub linked_product_id: String,
}

#[derive(Debug, Deserialize)]
// Form payload for the PayNowTransferForm request.
pub struct PayNowTransferForm {
    pub from_product_id: String,
    pub recipient_type: String,
    pub recipient_id: String,
    pub amount: String,
    pub note: String,
}
