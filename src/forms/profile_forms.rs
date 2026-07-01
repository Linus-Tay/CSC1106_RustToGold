// Form layer: request payload structs received from HTML forms.

use serde::Deserialize;

#[derive(Debug, Deserialize)]
// Form payload for the ProfileForm request.
pub struct ProfileForm {
    pub full_name: String,
    pub phone_number: String,
    #[serde(default)]
    pub paynow_id: String,
    #[serde(default)]
    pub linked_product_id: String,
    #[serde(default)]
    pub current_password: String,
    #[serde(default)]
    pub new_password: String,
    #[serde(default)]
    pub confirm_password: String,
}
