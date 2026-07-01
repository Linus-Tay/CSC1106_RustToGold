// Form layer: request payload structs received from HTML forms.

use serde::Deserialize;

#[derive(Debug, Deserialize)]
// Form payload for the CardApplicationForm request.
pub struct CardApplicationForm {
    pub linked_account_id: String,
    pub card_type: String,
    pub display_name: Option<String>,
}
