// Service layer: keeps banking validation and workflow rules away from templates and SQL.

use crate::forms::{AccountCreationForm, LoginForm};
use crate::models::{KnownDevice, User};
use crate::repositories::{customer_repository, user_repository};
use crate::services;
use crate::views::templates::Account2FAEmailTemplate;
use actix_web::Error;
use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use rand::RngExt;
use sha2::{Sha256, Digest};
use sqlx::PgPool;
use uuid::Uuid;

// Validates and coordinates the register user workflow.
pub async fn register_user(
    db: &PgPool,
    customer_id: &Uuid,
    customer_email: &str,
    form: AccountCreationForm,
) -> Result<User, String> {
    let username = form.username.trim().to_lowercase();

    if username.len() < 4 {
        return Err("Username must be at least 4 characters.".to_string());
    }

    if form.password.len() < 8 {
        return Err("Password must be at least 8 characters.".to_string());
    }

    if form.password != form.confirm_password {
        return Err("Passwords do not match.".to_string());
    }

    if user_repository::find_user_by_login(db, &username)
        .await
        .map_err(|_| "Could not check username availability.".to_string())?
        .is_some()
    {
        return Err("This username is already in use.".to_string());
    }

    let customer = customer_repository::get_customer_by_id(db, customer_id)
        .await
        .map_err(|_| "Could not load the approved customer profile.".to_string())?;

    let password_hash = hash_password(&form.password).map_err(|_| "Cannot create user".to_string())?;

    user_repository::create_customer_user(
        db,
        customer.id,
        &username,
        customer_email,
        &password_hash,
    )
    .await
    .map_err(|error| {
        eprintln!("Error creating online banking user: {error:?}");
        "Could not create online banking user.".to_string()
    })
}

// Runs business logic for authenticate user.
pub async fn authenticate_user(db: &PgPool, form: LoginForm) -> Result<User, String> {
    let login = form.username.trim().to_lowercase();

    let user = user_repository::find_user_by_login(db, &login)
        .await
        .map_err(|error| {
            eprintln!("LOGIN lookup failed for {login}: {error:?}");
            "Could not load your account.".to_string()
        })?
        .ok_or_else(|| "Invalid username/email or password.".to_string())?;

    if !user.is_active() {
        return Err("This account is not active.".to_string());
    }

    if !verify_password(&form.password, &user.password_hash) {
        return Err("Invalid username/email or password.".to_string());
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
// Runs business logic for authenticate device.
pub async fn authenticate_device(db: &PgPool, raw_token: &str) -> Result<KnownDevice, String> {
    let hashed_token = hash_device_token(raw_token);
    println!("{}", hashed_token);
    let device = customer_repository::find_device_by_hashed_token(db, &hashed_token)
        .await
        .map_err(|error| {
            eprintln!("LOGIN device lookup failed: {error:?}");
            "Could not verify your device.".to_string()
        })?
        .ok_or_else(|| "Not known device".to_string())?;

    if device.is_active() == false {
        println!("is it this?");
        return Err("Device is not active".to_string())
    }

    customer_repository::update_known_device_last_used(db, &device.id)
        .await
        .map_err(|error| {
            eprintln!(
                "DEVICE last-used update failed: {:?}",
                error 
            );
            "Device verified, but the last-used timestamp could not be updated.".to_string()
        })?;

    Ok(device)
}

// Runs business logic to add trusted device.
pub async fn add_trusted_device(db: &PgPool, user_id: &Uuid, raw_token: &str) -> Result<KnownDevice, String> {
    let hashed_token = hash_device_token(raw_token);
    let device = customer_repository::create_known_device(db,  user_id, &hashed_token)
        .await
        .map_err(|error| {
            eprintln!("Failed to add trusted device: {error:?}");
            "Could not add this device.".to_string()
        })?;

    Ok(device)
}


// Runs business logic to trigger 2FA email.
pub async fn generate_and_send_2fa(db: &PgPool, user_id: &Uuid) -> Result<(), String> {
    let mut rng = rand::rng();
    let verification_code: String = (0..6)
        .map(|_| rng.random_range(0..10).to_string())
        .collect();

    let user = user_repository::find_user_by_id(db, *user_id)
    .await
    .map_err(|error| {
        eprintln!("2FA lookup failed for {user_id}: {error:?}");
        "Could not load your account.".to_string()
    })?
    .ok_or_else(|| "Not known device".to_string())?;


    let email_to_send = user.email.clone();
    let subject_to_send = format!(
        "Login Request: {}",
        user.username.clone()
    );
    let template = Account2FAEmailTemplate {
        verification_code: verification_code.clone(),
    };

    println!(
        "2FA EMAIL: sending 2fa email to {email_to_send}."
    );

    if let Err(error) = services::send_template_email(&email_to_send, &subject_to_send, &template).await {
        eprintln!("2fa email failed: {error}");
        return Err("failed to send 2fa email".to_string())
    }

    customer_repository::create_otp_code(db, &user.id, &verification_code)
    .await
    .map_err(|e| {
        eprintln!("2FA add failed for {user_id}: {e:?}");
        "Could not add 2fa code".to_string()
    })?;

    Ok(())
}

// Runs business logic to trigger 2FA email.
pub async fn verify_2fa(db: &PgPool, otp_code: &str, user_id: &Uuid) -> Result<(), String> {
    
    let otp_code = customer_repository::get_otp_code(db, otp_code)
    .await
    .map_err(|_| {
        eprintln!("Failed to retrieve / validate 2fa code");
        "failed to validate 2fa code".to_string()
    })?
    .ok_or_else(|| "Invalid code".to_string())?;

    customer_repository::delete_otp_code(db, &otp_code.id)
    .await
    .map_err(|e| {
        eprintln!("failed to delete 2fa code: {}", e.to_string());
        "failed to delete 2fa code".to_string()
    })?;

    if otp_code.is_active() == false {
        return Err("The code has expired.".to_string());
    }

    if otp_code.get_user_id() != *user_id {
        return Err("Invalid code".to_string());
    }

    Ok(())
}

// Hashes sensitive input before it is stored.
fn hash_password(password: &str) -> Result<String, Error> {
    let salt = SaltString::generate(&mut OsRng);
    let hash = Argon2::default()
        .hash_password(password.as_bytes(), &salt)
        .map_err(actix_web::error::ErrorInternalServerError)?;
    Ok(hash.to_string())
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

// When you generate the UUID and want to save it to the DB:
pub fn hash_device_token(raw_token: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(raw_token.as_bytes());
    let result = hasher.finalize();
    
    // Use the hex crate to encode the byte array into a String
    hex::encode(result) 
}