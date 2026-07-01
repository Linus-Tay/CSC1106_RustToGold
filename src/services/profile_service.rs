// Service layer: keeps banking validation and workflow rules away from templates and SQL.

use crate::forms::ProfileForm;
use crate::models::Customer;
use crate::repositories::{customer_repository, paynow_repository, product_repository, user_repository};
use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use sqlx::PgPool;
use uuid::Uuid;

// Validates and coordinates the update customer profile workflow.
pub async fn update_customer_profile(
    db: &PgPool,
    customer_id: Uuid,
    user_id: Uuid,
    form: ProfileForm,
) -> Result<Customer, String> {
    let full_name = form.full_name.trim();
    let phone_number = form.phone_number.trim();

    if full_name.len() < 2 {
        return Err("Full name must be at least 2 characters.".to_string());
    }

    if phone_number.len() < 8 {
        return Err("Enter a valid phone number.".to_string());
    }

    let updated_customer = customer_repository::update_basic_profile(db, customer_id, full_name, phone_number)
        .await
        .map_err(|_| "Could not update your profile.".to_string())?;

    update_paynow_number(db, customer_id, &form).await?;
    update_password_if_requested(db, user_id, &form).await?;

    Ok(updated_customer)
}

// Validates and coordinates the update paynow number workflow.
async fn update_paynow_number(
    db: &PgPool,
    customer_id: Uuid,
    form: &ProfileForm,
) -> Result<(), String> {
    let paynow_id = form.paynow_id.trim();
    if paynow_id.is_empty() {
        return Ok(());
    }

    let paynow_id = normalise_phone_number(paynow_id)?;
    let linked_product_id = Uuid::parse_str(form.linked_product_id.trim())
        .map_err(|_| "Choose a valid account to receive PayNow transfers.".to_string())?;

    product_repository::get_active_product_for_customer_by_id(db, customer_id, linked_product_id)
        .await
        .map_err(|_| "Choose an active account that belongs to you for PayNow.".to_string())?;

    if let Some(existing) = paynow_repository::find_active_by_identifier(db, "phone_number", &paynow_id)
        .await
        .map_err(|_| "Could not check PayNow registration.".to_string())?
    {
        if existing.customer_id != customer_id {
            return Err("This PayNow number is already registered to another customer.".to_string());
        }
    }

    paynow_repository::upsert_phone_registration(db, customer_id, &paynow_id, linked_product_id)
        .await
        .map_err(|error| {
            eprintln!("Profile PayNow update failed: {error:?}");
            "Could not update your PayNow number.".to_string()
        })?;

    Ok(())
}

// Validates and coordinates the update password if requested workflow.
async fn update_password_if_requested(
    db: &PgPool,
    user_id: Uuid,
    form: &ProfileForm,
) -> Result<(), String> {
    let current_password = form.current_password.trim();
    let new_password = form.new_password.trim();
    let confirm_password = form.confirm_password.trim();

    let password_change_requested = !current_password.is_empty()
        || !new_password.is_empty()
        || !confirm_password.is_empty();

    if !password_change_requested {
        return Ok(());
    }

    if current_password.is_empty() {
        return Err("Enter your current password before changing it.".to_string());
    }

    if new_password.len() < 8 {
        return Err("New password must be at least 8 characters.".to_string());
    }

    if new_password != confirm_password {
        return Err("New passwords do not match.".to_string());
    }

    let user = user_repository::find_user_by_id(db, user_id)
        .await
        .map_err(|_| "Could not load your online banking user.".to_string())?
        .ok_or_else(|| "Online banking user was not found.".to_string())?;

    if !verify_password(current_password, &user.password_hash) {
        return Err("Current password is incorrect.".to_string());
    }

    let password_hash = hash_password(new_password)?;
    user_repository::update_password(db, user_id, &password_hash)
        .await
        .map_err(|_| "Could not update your password.".to_string())?;

    Ok(())
}

// Normalises phone number before validation or storage.
fn normalise_phone_number(input: &str) -> Result<String, String> {
    let mut value = input
        .trim()
        .replace(' ', "")
        .replace('-', "")
        .replace('(', "")
        .replace(')', "");

    if let Some(stripped) = value.strip_prefix("+65") {
        value = stripped.to_string();
    } else if value.starts_with("65") && value.len() == 10 {
        value = value[2..].to_string();
    }

    if value.len() != 8 || !value.chars().all(|character| character.is_ascii_digit()) {
        return Err("Enter an 8-digit Singapore mobile number.".to_string());
    }

    if !matches!(value.chars().next(), Some('8') | Some('9')) {
        return Err("PayNow mobile number should start with 8 or 9.".to_string());
    }

    Ok(value)
}

// Hashes sensitive input before it is stored.
fn hash_password(password: &str) -> Result<String, String> {
    let salt = SaltString::generate(&mut OsRng);
    Argon2::default()
        .hash_password(password.as_bytes(), &salt)
        .map(|hash| hash.to_string())
        .map_err(|_| "Could not prepare the new password.".to_string())
}

// Verifies sensitive input against its stored hash.
fn verify_password(password: &str, password_hash: &str) -> bool {
    PasswordHash::new(password_hash)
        .ok()
        .and_then(|parsed_hash| {
            Argon2::default()
                .verify_password(password.as_bytes(), &parsed_hash)
                .ok()
        })
        .is_some()
}
