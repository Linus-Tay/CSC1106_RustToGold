use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct LoginForm {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Deserialize)]

pub struct TwoFactorForm {
    pub code: String
}

#[derive(Debug, Deserialize)]
pub struct AccountCreationForm {
    pub username: String,
    pub password: String,
    pub confirm_password: String,
    pub notify_transactions: Option<String>,
    pub notify_login: Option<String>,
    pub notify_promotions: Option<String>,
}
