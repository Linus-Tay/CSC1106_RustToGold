use crate::forms::auth_forms::AccountCreationForm;
use crate::forms::{LoginForm, SignupForm};
use crate::models::User;
use crate::repositories::{account_repository, customer_repository, user_repository};
use actix_web::Error;
use argon2::{
    password_hash::{rand_core::OsRng, SaltString, PasswordHasher, PasswordHash, PasswordVerifier},
    Argon2,
};
use uuid::Uuid;
use sqlx::PgPool;

pub async fn register_user(db: &PgPool, customer_id: &Uuid, customer_email: &str, form: AccountCreationForm) -> Result<User, String> {
    let username = form.username;
    let password_hash = hash_password(&form.password).map_err(|e| "Cannot create user".to_string())?;
    let user = user_repository::create_user(db, customer_id, &username, customer_email, &password_hash)
    .await
    .map_err(|e| {
        println!("Error creating user: {}", e.to_string());
        "Could not create user".to_string()
    })?;

    Ok(user)
}

pub async fn authenticate_user(db: &PgPool, form: LoginForm) -> Result<User, String> {
    let username = form.username.trim().to_lowercase();

    let user = user_repository::find_user_by_username(db, &username)
        .await
        .map_err(|error| {
            eprintln!("LOGIN username lookup failed for {username}: {error:?}");
            "Could not load your account.".to_string()
        })?
        .ok_or_else(|| "Invalid email or password.".to_string())?;

    if !user.is_active() {
        return Err("This account is not active.".to_string());
    }

    if !verify_password(&form.password, &user.password_hash) {
        return Err("Invalid email or password.".to_string());
    }

    user_repository::update_last_login(db, user.id)
        .await
        .map_err(|error| {
            eprintln!(
                "LOGIN last-login update failed for user_id {}: {:?}",
                user.id, error
            );
            "Login succeeded, but the last-login timestamp could not be updated.".to_string()
        })?;

    Ok(user)
}

fn hash_password(password: &str) -> Result<String, Error> {
    let salt = SaltString::generate(&mut OsRng);
    let hash = Argon2::default()
        .hash_password(password.as_bytes(), &salt)
        .map_err(actix_web::error::ErrorInternalServerError)?;
    Ok(hash.to_string())
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

fn optional_trimmed(value: Option<&String>) -> Option<&str> {
    value
        .map(|value| value.trim())
        .filter(|value| !value.is_empty())
}

fn normalise_account_type(value: &str) -> &'static str {
    match value {
        "high_yield_savings" => "high_yield_savings",
        _ => "everyday_savings",
    }
}
