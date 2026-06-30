use crate::forms::{AccountCreationForm, LoginForm, SignupForm};
use crate::models::{Customer, Product, User};
use crate::repositories::{customer_repository, product_repository, user_repository};
use actix_web::Error;
use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use chrono::NaiveDate;
use sqlx::PgPool;
use uuid::Uuid;

pub async fn submit_customer_application(
    db: &PgPool,
    form: SignupForm,
) -> Result<(Customer, Product), String> {
    let full_name = form.full_name.trim();
    let email = form.email.trim().to_lowercase();
    let phone_number = form.phone_number.trim();
    let nric_fin = form.nric_fin.trim().to_uppercase();
    let nationality = form.nationality.trim();
    let residential_status = form.residential_status.trim();
    let residential_address = form.residential_address.trim();
    let employment_status = form.employment_status.trim();
    let account_type = normalise_account_type(&form.selected_account_type);

    if full_name.len() < 2 {
        return Err("Full name must be at least 2 characters.".to_string());
    }

    if nric_fin.len() < 5 {
        return Err("Enter a valid NRIC or FIN.".to_string());
    }

    if !email.contains('@') || email.len() < 5 {
        return Err("Enter a valid email address.".to_string());
    }

    if phone_number.len() < 8 {
        return Err("Enter a valid mobile number.".to_string());
    }

    let date_of_birth = NaiveDate::parse_from_str(form.date_of_birth.trim(), "%Y-%m-%d")
        .map_err(|_| "Enter a valid date of birth.".to_string())?;

    if nationality.is_empty() || residential_status.is_empty() || residential_address.is_empty() {
        return Err("Please complete your personal details.".to_string());
    }

    if employment_status.is_empty() {
        return Err("Please select your employment status.".to_string());
    }

    if form.opening_for_self.is_none()
        || form.not_acting_for_others.is_none()
        || form.funds_legitimate.is_none()
        || form.terms_agreed.is_none()
        || form.accuracy_confirmed.is_none()
    {
        return Err("Please confirm the account opening declarations.".to_string());
    }

    if customer_repository::get_customer_by_nric(db, &nric_fin)
        .await
        .map_err(|error| {
            eprintln!("SIGNUP NRIC lookup failed for {nric_fin}: {error:?}");
            "Could not check whether this identity document already exists.".to_string()
        })?
        .is_some()
    {
        return Err("An account application already exists for this NRIC or FIN.".to_string());
    }

    if user_repository::find_user_by_login(db, &email)
        .await
        .map_err(|error| {
            eprintln!("SIGNUP email lookup failed for {email}: {error:?}");
            "Could not check whether this email already exists.".to_string()
        })?
        .is_some()
    {
        return Err("An online banking profile with this email already exists.".to_string());
    }

    let new_customer = customer_repository::NewCustomer {
        full_name,
        nric: &nric_fin,
        date_of_birth,
        gender: "Not collected",
        nationality,
        residency: residential_status,
        race: None,
        email: &email,
        phone_number,
        residential_address,
        mailing_address: optional_trimmed(form.mailing_address.as_ref()),
        preferred_contact: Some("email"),
        employment_status,
        occupation: optional_trimmed(form.occupation.as_ref()),
        employer_name: optional_trimmed(form.employer_name.as_ref()),
        industry: None,
        monthly_income_range: optional_trimmed(form.monthly_income_range.as_ref()),
        kyc_status: Some("pending"),
    };

    let account_number = crate::services::generate_account_number(db).await;
    customer_repository::create_customer_and_product(
        db,
        &new_customer,
        account_type,
        "savings",
        &account_number,
    )
    .await
    .map_err(|error| {
        eprintln!("SIGNUP pending application insert failed for {email}: {error:?}");
        "Could not submit your account application.".to_string()
    })
}

// Kept so older controller imports do not break. This now submits an application and returns an error
// asking the user to wait for admin approval instead of creating an online banking user immediately.
pub async fn register_customer(db: &PgPool, form: SignupForm) -> Result<User, String> {
    let _ = submit_customer_application(db, form).await?;
    Err("Application submitted. Please wait for admin approval and use the account creation email to set up online banking.".to_string())
}

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
        return Err("This username is already taken.".to_string());
    }

    let customer = customer_repository::get_customer_by_id(db, customer_id)
        .await
        .map_err(|_| "Could not load the approved customer profile.".to_string())?;

    let password_hash = hash_password(&form.password).map_err(|_| "Cannot create user".to_string())?;

    user_repository::create_customer_user(
        db,
        customer.id,
        &username,
        &customer.full_name,
        customer_email,
        &customer.phone_number,
        customer.date_of_birth,
        &password_hash,
    )
    .await
    .map_err(|error| {
        eprintln!("Error creating online banking user: {error:?}");
        "Could not create online banking user.".to_string()
    })
}

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
