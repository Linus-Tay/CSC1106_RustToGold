use crate::forms::ProfileForm;
use crate::models::User;
use crate::repositories::user_repository;
use sqlx::PgPool;

pub async fn update_customer_profile(
    db: &PgPool,
    user_id: i64,
    form: ProfileForm,
) -> Result<User, String> {
    let full_name = form.full_name.trim();
    let phone_number = form.phone_number.trim();

    if full_name.len() < 2 {
        return Err("Full name must be at least 2 characters.".to_string());
    }

    if phone_number.len() < 8 {
        return Err("Enter a valid phone number.".to_string());
    }

    user_repository::update_profile(db, user_id, full_name, phone_number)
        .await
        .map_err(|_| "Could not update your profile.".to_string())
}
