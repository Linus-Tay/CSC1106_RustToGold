use crate::models::{
    AdminCustomerApplication, AdminDashboardSummary, AdminHomeLoanRecord, AdminPersonalLoanRecord,
};
use crate::repositories::admin_repository;
use sqlx::PgPool;
use uuid::Uuid;

pub async fn load_admin_dashboard(db: &PgPool) -> Result<AdminDashboardSummary, String> {
    admin_repository::dashboard_summary(db)
        .await
        .map_err(|error| {
            eprintln!("admin dashboard summary failed: {error:?}");
            "Could not load the admin dashboard.".to_string()
        })
}

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

pub async fn approve_customer_application(db: &PgPool, customer_id: Uuid) -> Result<(), String> {
    admin_repository::approve_customer_application(db, customer_id)
        .await
        .map_err(|error| {
            eprintln!("customer application approve failed: {error:?}");
            "Could not approve the customer application.".to_string()
        })
}

pub async fn reject_customer_application(db: &PgPool, customer_id: Uuid) -> Result<(), String> {
    admin_repository::reject_customer_application(db, customer_id)
        .await
        .map_err(|error| {
            eprintln!("customer application reject failed: {error:?}");
            "Could not reject the customer application.".to_string()
        })
}

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

pub async fn list_admin_home_loans(db: &PgPool) -> Result<Vec<AdminHomeLoanRecord>, String> {
    admin_repository::list_home_loans(db)
        .await
        .map_err(|error| {
            eprintln!("admin home loans failed: {error:?}");
            "Could not load home loan applications.".to_string()
        })
}

pub async fn approve_personal_loan(
    db: &PgPool,
    staff_user_id: i64,
    loan_id: Uuid,
) -> Result<(), String> {
    admin_repository::approve_personal_loan(db, staff_user_id, loan_id)
        .await
        .map_err(|error| {
            eprintln!("personal loan approve failed: {error:?}");
            "Could not approve the personal loan.".to_string()
        })
}

pub async fn reject_personal_loan(
    db: &PgPool,
    staff_user_id: i64,
    loan_id: Uuid,
) -> Result<(), String> {
    admin_repository::reject_personal_loan(db, staff_user_id, loan_id)
        .await
        .map_err(|error| {
            eprintln!("personal loan reject failed: {error:?}");
            "Could not reject the personal loan.".to_string()
        })
}
