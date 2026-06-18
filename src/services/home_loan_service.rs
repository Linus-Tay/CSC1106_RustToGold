// ======================================================================
// HOME LOAN WORKFLOW
//
// Customer submits application
//            │
//            ▼
// status = pending_review
//            │
//            ▼
// Staff dashboard
//      ├──────────────┐
//      │              │
//   Approve        Reject
//      │              │
//      ▼              ▼
// approve_application()   status = rejected
//      │
//      ▼
// Money credited to account
// Transaction created
// Repayment schedule generated
// Customer can repay monthly
// Remaining reaches 0
// Status becomes completed
//
// ======================================================================
use crate::forms::HomeLoanApplicationForm;
use crate::models::{HomeLoanApplication, Money};
use crate::repositories::{account_repository, home_loan_repository};
use sqlx::PgPool;

const HOME_LOAN_INTEREST_RATE_BPS: i32 = 170;

pub async fn apply_home_loan(
    db: &PgPool,
    user_id: i64,
    form: HomeLoanApplicationForm,
) -> Result<HomeLoanApplication, String> {
    let amount = Money::parse_dollars(&form.amount)?;
    let amount_cents = amount.cents();

    let term_years = form
        .term_years
        .trim()
        .parse::<i32>()
        .map_err(|_| "Invalid home loan term.".to_string())?;

    let (max_amount_cents, allowed_terms): (i64, Vec<i32>) = match form.house_type.as_str() {
        "hdb_1_2_room" => (20_000_000, vec![5, 10, 15]),
        "hdb_3_plus" => (50_000_000, vec![10, 15, 20, 25]),
        "condo" => (100_000_000, vec![10, 15, 20, 25, 30]),
        "landed" => (200_000_000, vec![15, 20, 25, 30, 35]),
        _ => return Err("Invalid house type selected.".to_string()),
    };

    if amount_cents < 100 {
        return Err("Home loan amount must be at least $1.00.".to_string());
    }

    if amount_cents > max_amount_cents {
        return Err(format!(
            "Requested amount exceeds the maximum allowed for this property type. Maximum: {}",
            Money::from_cents(max_amount_cents).display()
        ));
    }

    if !allowed_terms.contains(&term_years) {
        return Err("Selected loan duration is not allowed for this property type.".to_string());
    }

    let account = account_repository::find_primary_account_by_user_id(db, user_id)
        .await
        .map_err(|_| "Could not load bank account.".to_string())?
        .ok_or_else(|| "No active bank account found.".to_string())?;

    let term_months = term_years * 12;

    let application = home_loan_repository::create_application(
        db,
        user_id,
        account.id,
        &form.house_type,
        amount_cents,
        HOME_LOAN_INTEREST_RATE_BPS,
        term_months,
    )
    .await
    .map_err(|_| "Home loan application could not be submitted.".to_string())?;

    Ok(application)
}

pub async fn list_home_loan_applications(
    db: &PgPool,
    user_id: i64,
) -> Result<Vec<HomeLoanApplication>, String> {
    home_loan_repository::list_by_user_id(db, user_id)
        .await
        .map_err(|_| "Could not load home loan applications.".to_string())
}

pub async fn pay_home_loan(
    db: &PgPool,
    user_id: i64,
    application_id: i64,
) -> Result<HomeLoanApplication, String> {
    let (application, _, _) =
        home_loan_repository::pay_home_loan(db, user_id, application_id)
            .await
            .map_err(|_| {
                "Home loan repayment failed. Please check your balance or loan status.".to_string()
            })?;

    Ok(application)
}
