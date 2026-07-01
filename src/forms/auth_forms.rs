// Form layer: request payload structs received from HTML forms.

use serde::Deserialize;

#[derive(Debug, Deserialize)]
// Form payload for the LoginForm request.
pub struct LoginForm {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Deserialize)]
// Form payload for the AccountCreationForm request.
pub struct AccountCreationForm {
    pub username: String,
    pub password: String,
    pub confirm_password: String,
    pub notify_transactions: Option<String>,
    pub notify_login: Option<String>,
    pub notify_promotions: Option<String>,
}
