use crate::forms::{LoginForm, SignupForm};
use crate::models::User;
use crate::repositories::{account_repository, user_repository};
use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use chrono::NaiveDate;
use sqlx::PgPool;

pub async fn register_customer(db: &PgPool, form: SignupForm) -> Result<User, String> {
    let full_name = form.full_name.trim();
    let email = form.email.trim().to_lowercase();
    let phone_number = form.phone_number.trim();

    if full_name.len() < 2 {
        return Err("Full name must be at least 2 characters.".to_string());
    }

    if !email.contains('@') || email.len() < 5 {
        return Err("Enter a valid email address.".to_string());
    }

    if phone_number.len() < 8 {
        return Err("Enter a valid phone number.".to_string());
    }

    let date_of_birth = NaiveDate::parse_from_str(form.date_of_birth.trim(), "%Y-%m-%d")
        .map_err(|_| "Enter a valid date of birth.".to_string())?;

    if form.password.len() < 8 {
        return Err("Password must be at least 8 characters.".to_string());
    }

    if form.password != form.confirm_password {
        return Err("Passwords do not match.".to_string());
    }

    if form.simulation_confirmed.is_none() {
        return Err("Please confirm this is only an academic banking simulation.".to_string());
    }

    let existing_user = user_repository::find_user_by_email(db, &email)
        .await
        .map_err(|_| "Could not check whether this email already exists.".to_string())?;

    if existing_user.is_some() {
        return Err("An account with this email already exists.".to_string());
    }

    let password_hash = hash_password(&form.password)?;
    let user = user_repository::create_customer(db, full_name, &email, phone_number, date_of_birth, &password_hash)
        .await
        .map_err(|_| "Could not create your customer account.".to_string())?;

    account_repository::create_primary_account(db, user.id)
        .await
        .map_err(|_| "Customer was created, but the bank account could not be created.".to_string())?;

    Ok(user)
}

pub async fn authenticate_user(db: &PgPool, form: LoginForm) -> Result<User, String> {
    let email = form.email.trim().to_lowercase();

    let user = user_repository::find_user_by_email(db, &email)
        .await
        .map_err(|_| "Could not load your account.".to_string())?
        .ok_or_else(|| "Invalid email or password.".to_string())?;

    if !user.is_active() {
        return Err("This account is not active.".to_string());
    }

    if !verify_password(&form.password, &user.password_hash) {
        return Err("Invalid email or password.".to_string());
    }

    user_repository::update_last_login(db, user.id)
        .await
        .map_err(|_| "Login succeeded, but the last-login timestamp could not be updated.".to_string())?;

    Ok(user)
}

fn hash_password(password: &str) -> Result<String, String> {
    let salt = SaltString::generate(&mut OsRng);
    Argon2::default()
        .hash_password(password.as_bytes(), &salt)
        .map(|hash| hash.to_string())
        .map_err(|_| "Could not hash password.".to_string())
}

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
