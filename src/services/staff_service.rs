use crate::forms::{CreateStaffForm, UpdateStaffForm};
use crate::models::StaffUser;
use crate::repositories::staff_repository;
use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use chrono::NaiveDate;
use sqlx::PgPool;
use crate::services::audit_log_service;
use crate::services::AuditContext;

pub async fn list_all_staff(db: &PgPool) -> Result<Vec<StaffUser>, String> {
    staff_repository::list_all_staff(db)
        .await
        .map_err(|_| "Could not load staff users.".to_string())
}

pub async fn find_staff_by_id(db: &PgPool, user_id: i64) -> Result<StaffUser, String> {
    staff_repository::find_staff_by_id(db, user_id)
        .await
        .map_err(|_| "Could not load staff user.".to_string())?
        .ok_or_else(|| "Staff user not found.".to_string())
}

pub async fn create_staff(db: &PgPool, ctx: &AuditContext, form: CreateStaffForm) -> Result<StaffUser, String> {
    let (full_name, email, phone_number, date_of_birth, status) = validate_staff_fields(
        &form.full_name,
        &form.email,
        &form.phone_number,
        &form.date_of_birth,
        &form.status,
    )?;

    let password = validate_password(&form.password)?;

    let email_taken = staff_repository::email_exists(db, &email)
        .await
        .map_err(|_| "Could not check email availability.".to_string())?;

    if email_taken {
        audit_log_service::record_simple(db, ctx, "create_staff", "staff", None, "failed").await;
        return Err("A user with this email address already exists.".to_string());
    }

    let password_hash = hash_password(&password)?;

    match staff_repository::create_staff(db, &full_name, &email, &phone_number, date_of_birth, &password_hash, &status).await {
        Ok(staff) => {
            audit_log_service::record(
                db, ctx,
                "create_staff", "staff", Some(staff.id),
                None, Some(&staff),
                "success",
            ).await;
            Ok(staff)
        }
        Err(_) => {
            audit_log_service::record_simple(db, ctx, "create_staff", "staff", None, "failed").await;
            Err("Could not create staff user. The email may already be in use.".to_string())
        }
    }
}


pub async fn update_staff(
    db: &PgPool,
    ctx: &AuditContext,
    user_id: i64,
    form: UpdateStaffForm,
) -> Result<StaffUser, String> {
    let (full_name, email, phone_number, date_of_birth, status) = validate_staff_fields(
        &form.full_name,
        &form.email,
        &form.phone_number,
        &form.date_of_birth,
        &form.status,
    )?;

    let email_taken = staff_repository::email_exists_for_other(db, &email, user_id)
        .await
        .map_err(|_| "Could not check email availability.".to_string())?;

    if email_taken {
        audit_log_service::record_simple(db, ctx, "update_staff", "staff", Some(user_id), "failed").await;
        return Err("A user with this email address already exists.".to_string());
    }

    let before = staff_repository::find_staff_by_id(db, user_id)
        .await
        .map_err(|_| "Could not load staff user.".to_string())?
        .ok_or_else(|| "Staff user not found.".to_string())?;

    match staff_repository::update_staff(db, user_id, &full_name, &email, &phone_number, date_of_birth, &status).await {
        Ok(after) => {
            let new_password = form.password.trim().to_string();
            if !new_password.is_empty() {
                let validated = validate_password(&new_password)?;
                let password_hash = hash_password(&validated)?;
                staff_repository::update_staff_password(db, user_id, &password_hash)
                    .await
                    .map_err(|_| "Could not update password.".to_string())?;
            }

            audit_log_service::record(
                db, ctx,
                "update_staff", "staff", Some(user_id),
                Some(&before), Some(&after),
                "success",
            ).await;

            Ok(after)
        }
        Err(_) => {
            audit_log_service::record_simple(db, ctx, "update_staff", "staff", Some(user_id), "failed").await;
            Err("Could not update staff user.".to_string())
        }
    }
}

pub async fn delete_staff(
    db: &PgPool,
    ctx: &AuditContext,
    user_id: i64,
) -> Result<(), String> {
    let staff = staff_repository::find_staff_by_id(db, user_id)
        .await
        .map_err(|_| "Could not load staff user.".to_string())?
        .ok_or_else(|| "Staff user not found.".to_string())?;

    match staff_repository::delete_staff(db, user_id).await {
        Ok(true) => {
            audit_log_service::record(
                db, ctx,
                "delete_staff", "staff", Some(user_id),
                Some(&staff), None,
                "success",
            ).await;
            Ok(())
        }
        _ => {
            audit_log_service::record_simple(db, ctx, "delete_staff", "staff", Some(user_id), "failed").await;
            Err("Staff user not found or could not be deleted.".to_string())
        }
    }
}

// --- Validation helpers ---

fn validate_staff_fields(
    full_name: &str,
    email: &str,
    phone_number: &str,
    date_of_birth: &str,
    status: &str,
) -> Result<(String, String, String, NaiveDate, String), String> {
    let full_name = full_name.trim().to_string();
    if full_name.len() < 2 {
        return Err("Full name must be at least 2 characters.".to_string());
    }
    if full_name.len() > 120 {
        return Err("Full name must be at most 120 characters.".to_string());
    }

    let email = email.trim().to_lowercase();
    if !email.contains('@') || !email.contains('.') {
        return Err("Please enter a valid email address.".to_string());
    }
    if email.len() > 255 {
        return Err("Email address is too long.".to_string());
    }

    let phone_number = phone_number.trim().to_string();
    if phone_number.len() < 7 {
        return Err("Phone number must be at least 7 characters.".to_string());
    }
    if phone_number.len() > 30 {
        return Err("Phone number must be at most 30 characters.".to_string());
    }

    let date_of_birth = NaiveDate::parse_from_str(date_of_birth.trim(), "%Y-%m-%d")
        .map_err(|_| "Date of birth must be in YYYY-MM-DD format.".to_string())?;

    let today = chrono::Utc::now().date_naive();
    let age_years = (today - date_of_birth).num_days() / 365;
    if age_years < 18 {
        return Err("Staff member must be at least 18 years old.".to_string());
    }

    let status = match status.trim() {
        "active" => "active".to_string(),
        "suspended" => "suspended".to_string(),
        "closed" => "closed".to_string(),
        _ => return Err("Status must be active, suspended, or closed.".to_string()),
    };

    Ok((full_name, email, phone_number, date_of_birth, status))
}

fn validate_password(password: &str) -> Result<String, String> {
    let password = password.trim().to_string();
    if password.len() < 8 {
        return Err("Password must be at least 8 characters.".to_string());
    }
    if password.len() > 128 {
        return Err("Password must be at most 128 characters.".to_string());
    }
    Ok(password)
}

fn hash_password(password: &str) -> Result<String, String> {
    let salt = SaltString::generate(&mut OsRng);
    Argon2::default()
        .hash_password(password.as_bytes(), &salt)
        .map(|hash| hash.to_string())
        .map_err(|_| "Could not hash password.".to_string())
}
