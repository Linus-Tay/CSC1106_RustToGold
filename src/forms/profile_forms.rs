use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct ProfileForm {
    pub full_name: String,
    pub phone_number: String,
}
