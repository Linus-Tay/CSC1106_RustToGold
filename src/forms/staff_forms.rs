use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct CreateStaffForm {
    pub full_name: String,
    pub email: String,
    pub phone_number: String,
    pub date_of_birth: String,
    pub password: String,
    pub status: String,
}

#[derive(Debug, Deserialize)]
pub struct UpdateStaffForm {
    pub full_name: String,
    pub email: String,
    pub phone_number: String,
    pub date_of_birth: String,
    pub status: String,
    /// Optional: if blank, password is not changed
    pub password: String,
}
