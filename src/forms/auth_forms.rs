use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct SignupForm {
    pub full_name: String,
    pub email: String,
    pub phone_number: String,
    pub date_of_birth: String,
    pub password: String,
    pub confirm_password: String,
    pub simulation_confirmed: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct LoginForm {
    pub email: String,
    pub password: String,
}
