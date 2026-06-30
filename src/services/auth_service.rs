use crate::forms::{LoginForm, SignupForm};
use crate::models::User;
use crate::repositories::{
    account_repository, customer_repository, product_repository, user_repository,
};
use argon2::{
    password_hash::{PasswordHash, PasswordVerifier},
    Argon2,
};
use chrono::NaiveDate;
use sqlx::PgPool;

pub async fn register_customer(db: &PgPool, form: SignupForm) -> Result<User, String> {
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

    if form.password_hash.trim().is_empty() {
        return Err("Please create a password for online banking access.".to_string());
    }

    if form.opening_for_self.is_none()
        || form.not_acting_for_others.is_none()
        || form.funds_legitimate.is_none()
        || form.terms_agreed.is_none()
        || form.accuracy_confirmed.is_none()
    {
        return Err("Please confirm the account opening declarations.".to_string());
    }

    let existing_user = user_repository::find_user_by_email(db, &email)
        .await
        .map_err(|error| {
            eprintln!("SIGNUP email lookup failed for {email}: {error:?}");
            "Could not check whether this email already exists.".to_string()
        })?;

    if existing_user.is_some() {
        return Err("An online banking profile with this email already exists.".to_string());
    }

    let existing_customer = customer_repository::get_customer_by_nric(db, &nric_fin.clone())
        .await
        .map_err(|error| {
            eprintln!("SIGNUP NRIC lookup failed for {nric_fin}: {error:?}");
            "Could not check whether this identity document already exists.".to_string()
        })?;

    if existing_customer.is_some() {
        return Err("An account application already exists for this NRIC or FIN.".to_string());
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

    let customer = customer_repository::create_customer(db, &new_customer)
        .await
        .map_err(|error| {
            eprintln!("SIGNUP customer insert failed for {email}: {error:?}");
            "Could not create your customer profile.".to_string()
        })?;

    let user = user_repository::create_customer(
        db,
        customer.id,
        full_name,
        &email,
        phone_number,
        date_of_birth,
        form.password_hash.trim(),
    )
    .await
    .map_err(|error| {
        eprintln!("SIGNUP user insert failed for {email}: {error:?}");
        "Could not create your online banking profile.".to_string()
    })?;

    let bank_account = account_repository::create_primary_account(db, user.id, account_type)
        .await
        .map_err(|error| {
            eprintln!(
                "SIGNUP primary account insert failed for user_id {}: {:?}",
                user.id, error
            );
            "Your profile was created, but the bank account could not be opened.".to_string()
        })?;

    product_repository::insert_product(
        db,
        &customer.id,
        account_type,
        "savings",
        &bank_account.account_number,
    )
    .await
    .map_err(|error| {
        eprintln!(
            "SIGNUP customer product insert failed for customer_id {}: {:?}",
            customer.id, error
        );
        "Your profile was created, but the customer product account could not be opened."
            .to_string()
    })?;

    Ok(user)
}

pub async fn authenticate_user(db: &PgPool, form: LoginForm) -> Result<User, String> {
    let email = form.email.trim().to_lowercase();

    let user = user_repository::find_user_by_email(db, &email)
        .await
        .map_err(|error| {
            eprintln!("LOGIN email lookup failed for {email}: {error:?}");
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
