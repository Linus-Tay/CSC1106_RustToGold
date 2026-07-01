use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct CardApplicationForm {
    pub linked_account_id: String,
    pub card_type: String,
    pub display_name: Option<String>,
}
