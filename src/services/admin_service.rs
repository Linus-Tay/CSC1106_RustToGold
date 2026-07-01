// Service layer: keeps banking validation and workflow rules away from templates and SQL.

use crate::models::{
    AdminAuditLogRecord, AdminCustomerAccountRecord, AdminCustomerApplication,
    AdminDashboardSummary, AdminHomeLoanRecord, AdminPersonalLoanRecord, AdminStaffUser,
};
use crate::repositories::admin_repository;
use crate::AppState;
use argon2::{
    password_hash::{rand_core::OsRng, PasswordHasher, SaltString},
    Argon2,
};
use sqlx::PgPool;
use uuid::Uuid;

// Loads admin dashboard data and applies page-level business rules.
pub async fn load_admin_dashboard(db: &PgPool) -> Result<AdminDashboardSummary, String> {
    admin_repository::dashboard_summary(db)
        .await
        .map_err(|error| {
            eprintln!("admin dashboard summary failed: {error:?}");
            "Could not load the admin dashboard.".to_string()
        })
}

// Returns admin customer applications records in the shape needed by the UI.
pub async fn list_admin_customer_applications(
    db: &PgPool,
) -> Result<Vec<AdminCustomerApplication>, String> {
    admin_repository::list_customer_applications(db)
        .await
        .map_err(|error| {
            eprintln!("admin customer applications failed: {error:?}");
            "Could not load customer applications.".to_string()
        })
}

// Validates and coordinates the approve customer application workflow.
pub async fn approve_customer_application(
    app_state: &AppState,
    staff_user_id: Uuid,
    customer_id: Uuid,
) -> Result<(), String> {
    crate::services::approve_customer_with_product(app_state, customer_id)
        .await
        .map(|_| ())
        .map_err(|error| {
            eprintln!("customer application approve failed: {error}");
            error
        })?;

    let _ = admin_repository::record_audit_log(
        &app_state.db,
        Some(staff_user_id),
        "approve_customer_application",
        "customer",
        Some(customer_id.to_string()),
        Some("Customer KYC approved and account creation link issued".to_string()),
    )
    .await;

    Ok(())
}

// Validates and coordinates the reject customer application workflow.
pub async fn reject_customer_application(
    db: &PgPool,
    staff_user_id: Uuid,
    customer_id: Uuid,
) -> Result<(), String> {
    admin_repository::reject_customer_application(db, customer_id)
        .await
        .map_err(|error| {
            eprintln!("customer application reject failed: {error:?}");
            "Could not reject the customer application.".to_string()
        })?;

    let _ = admin_repository::record_audit_log(
        db,
        Some(staff_user_id),
        "reject_customer_application",
        "customer",
        Some(customer_id.to_string()),
        Some("Customer KYC application rejected".to_string()),
    )
    .await;

    Ok(())
}

// Returns admin personal loans records in the shape needed by the UI.
pub async fn list_admin_personal_loans(
    db: &PgPool,
) -> Result<Vec<AdminPersonalLoanRecord>, String> {
    admin_repository::list_personal_loans(db)
        .await
        .map_err(|error| {
            eprintln!("admin personal loans failed: {error:?}");
            "Could not load personal loan applications.".to_string()
        })
}

// Returns admin home loans records in the shape needed by the UI.
pub async fn list_admin_home_loans(db: &PgPool) -> Result<Vec<AdminHomeLoanRecord>, String> {
    admin_repository::list_home_loans(db)
        .await
        .map_err(|error| {
            eprintln!("admin home loans failed: {error:?}");
            "Could not load home loan applications.".to_string()
        })
}

// Validates and coordinates the approve personal loan workflow.
pub async fn approve_personal_loan(
    db: &PgPool,
    staff_user_id: Uuid,
    loan_id: Uuid,
) -> Result<(), String> {
    admin_repository::approve_personal_loan(db, staff_user_id, loan_id)
        .await
        .map_err(|error| {
            eprintln!("personal loan approve failed: {error:?}");
            "Could not approve the personal loan.".to_string()
        })
}

// Validates and coordinates the reject personal loan workflow.
pub async fn reject_personal_loan(
    db: &PgPool,
    staff_user_id: Uuid,
    loan_id: Uuid,
) -> Result<(), String> {
    admin_repository::reject_personal_loan(db, staff_user_id, loan_id)
        .await
        .map_err(|error| {
            eprintln!("personal loan reject failed: {error:?}");
            "Could not reject the personal loan.".to_string()
        })
}

// Returns admin staff records in the shape needed by the UI.
pub async fn list_admin_staff(db: &PgPool) -> Result<Vec<AdminStaffUser>, String> {
    admin_repository::list_staff_users(db)
        .await
        .map_err(|error| {
            eprintln!("staff list failed: {error:?}");
            "Could not load staff users.".to_string()
        })
}

// Validates and coordinates the create staff user workflow.
pub async fn create_staff_user(
    db: &PgPool,
    actor_user_id: Uuid,
    username: String,
    full_name: String,
    email: String,
    phone_number: String,
    role: String,
    password: String,
) -> Result<(), String> {
    let username = username.trim().to_lowercase();
    let full_name = full_name.trim().to_string();
    let email = email.trim().to_lowercase();
    let phone_number = phone_number.trim().to_string();
    let role = normalise_staff_role(&role)?;

    validate_staff_fields(&username, &full_name, &email, &phone_number)?;
    if password.len() < 8 {
        return Err("Staff password must be at least 8 characters.".to_string());
    }

    let password_hash = hash_password(&password)?;
    admin_repository::create_staff_user(
        db,
        &username,
        &full_name,
        &email,
        &phone_number,
        role,
        &password_hash,
        actor_user_id,
    )
    .await
    .map_err(|error| {
        eprintln!("staff create failed: {error:?}");
        "Could not create staff user. Check if username or email already exists.".to_string()
    })
}

// Validates and coordinates the update staff user workflow.
pub async fn update_staff_user(
    db: &PgPool,
    actor_user_id: Uuid,
    staff_user_id: Uuid,
    full_name: String,
    email: String,
    phone_number: String,
    role: String,
    status: String,
    password: Option<String>,
) -> Result<(), String> {
    let full_name = full_name.trim().to_string();
    let email = email.trim().to_lowercase();
    let phone_number = phone_number.trim().to_string();
    let role = normalise_staff_role(&role)?;
    let status = normalise_user_status(&status)?;

    if full_name.len() < 2 || !email.contains('@') || phone_number.len() < 8 {
        return Err("Enter a valid name, email and phone number.".to_string());
    }

    let password_hash = match password.map(|value| value.trim().to_string()).filter(|value| !value.is_empty()) {
        Some(value) => {
            if value.len() < 8 {
                return Err("New password must be at least 8 characters.".to_string());
            }
            Some(hash_password(&value)?)
        }
        None => None,
    };

    admin_repository::update_staff_user(
        db,
        staff_user_id,
        &full_name,
        &email,
        &phone_number,
        role,
        status,
        password_hash.as_deref(),
        actor_user_id,
    )
    .await
    .map_err(|error| {
        eprintln!("staff update failed: {error:?}");
        "Could not update staff user.".to_string()
    })
}

// Runs business logic for delete staff user.
pub async fn delete_staff_user(
    db: &PgPool,
    actor_user_id: Uuid,
    staff_user_id: Uuid,
) -> Result<(), String> {
    if actor_user_id == staff_user_id {
        return Err("You cannot delete your own admin session account.".to_string());
    }

    admin_repository::delete_staff_user(db, staff_user_id, actor_user_id)
        .await
        .map_err(|error| {
            eprintln!("staff delete failed: {error:?}");
            "Could not delete staff user. Admin users cannot be deleted here.".to_string()
        })
}

// Returns admin customer accounts records in the shape needed by the UI.
pub async fn list_admin_customer_accounts(
    db: &PgPool,
) -> Result<Vec<AdminCustomerAccountRecord>, String> {
    admin_repository::list_customer_accounts(db)
        .await
        .map_err(|error| {
            eprintln!("customer account list failed: {error:?}");
            "Could not load customer accounts.".to_string()
        })
}

// Validates and coordinates the set customer user status workflow.
pub async fn set_customer_user_status(
    db: &PgPool,
    actor_user_id: Uuid,
    target_user_id: Uuid,
    status: &str,
) -> Result<(), String> {
    let status = normalise_user_status(status)?;
    admin_repository::set_user_status(db, target_user_id, status, actor_user_id)
        .await
        .map_err(|error| {
            eprintln!("customer user status update failed: {error:?}");
            "Could not update the customer user status.".to_string()
        })
}

// Validates and coordinates the set customer product status workflow.
pub async fn set_customer_product_status(
    db: &PgPool,
    actor_user_id: Uuid,
    product_id: Uuid,
    status: &str,
) -> Result<(), String> {
    let status = normalise_product_status(status)?;
    admin_repository::set_product_status(db, product_id, status, actor_user_id)
        .await
        .map_err(|error| {
            eprintln!("customer product status update failed: {error:?}");
            "Could not update the customer product status.".to_string()
        })
}

// Returns audit logs records in the shape needed by the UI.
pub async fn list_audit_logs(db: &PgPool) -> Result<Vec<AdminAuditLogRecord>, String> {
    admin_repository::list_audit_logs(db)
        .await
        .map_err(|error| {
            eprintln!("audit log list failed: {error:?}");
            "Could not load audit logs.".to_string()
        })
}

// Checks staff fields rules before the workflow continues.
fn validate_staff_fields(username: &str, full_name: &str, email: &str, phone_number: &str) -> Result<(), String> {
    if username.len() < 4 {
        return Err("Staff username must be at least 4 characters.".to_string());
    }
    if full_name.len() < 2 {
        return Err("Staff full name must be at least 2 characters.".to_string());
    }
    if !email.contains('@') || email.len() < 5 {
        return Err("Enter a valid staff email address.".to_string());
    }
    if phone_number.len() < 8 {
        return Err("Enter a valid staff phone number.".to_string());
    }
    Ok(())
}

// Normalises staff role before validation or storage.
fn normalise_staff_role(value: &str) -> Result<&'static str, String> {
    match value.trim() {
        "admin" => Ok("admin"),
        "staff" | "" => Ok("staff"),
        _ => Err("Staff role must be staff or admin.".to_string()),
    }
}

// Normalises user status before validation or storage.
fn normalise_user_status(value: &str) -> Result<&'static str, String> {
    match value.trim() {
        "active" => Ok("active"),
        "suspended" => Ok("suspended"),
        "closed" => Ok("closed"),
        _ => Err("User status must be active, suspended or closed.".to_string()),
    }
}

// Normalises product status before validation or storage.
fn normalise_product_status(value: &str) -> Result<&'static str, String> {
    match value.trim() {
        "active" => Ok("active"),
        "inactive" => Ok("inactive"),
        "frozen" => Ok("frozen"),
        "closed" => Ok("closed"),
        _ => Err("Product status must be active, inactive, frozen or closed.".to_string()),
    }
}

// Hashes sensitive input before it is stored.
fn hash_password(password: &str) -> Result<String, String> {
    let salt = SaltString::generate(&mut OsRng);
    Argon2::default()
        .hash_password(password.as_bytes(), &salt)
        .map(|hash| hash.to_string())
        .map_err(|_| "Could not hash staff password.".to_string())
}
