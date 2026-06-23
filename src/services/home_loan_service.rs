use crate::forms::{HomeLoanApplicationForm, HomeLoanPaymentForm};
use crate::models::{AdminHomeLoanRecord, BankAccount, HomeLoanApplication, HomeLoanSummary, Money};
use crate::repositories::{account_repository, home_loan_repository};
use sqlx::PgPool;

//home loan interest are usually lower cuz its secured
//simple interest no float
const HOME_LOAN_INTEREST_RATE_BPS: i32 = 170;

pub struct HomeLoanDashboard {
    pub account: BankAccount,
    pub summary: HomeLoanSummary,
    pub applications: Vec<HomeLoanApplication>,
}

pub async fn apply_home_loan(
    db: &PgPool,
    user_id: i64,
    form: HomeLoanApplicationForm,
) -> Result<HomeLoanApplication, String> {
    let amount: Money = Money::parse_dollars(&form.amount)?;
    let amount_cents: i64 = amount.cents();
    
    let account = account_repository::find_primary_account_by_user_id(db, user_id)
        .await
        .map_err(|_| "Could not load your bank account.".to_string())?
        .ok_or_else(|| "No active bank account found.".to_string())?;

    if account.status != "active" {
        return Err("Your account is not active.".to_string());
    }

    let home_loan_years = form
        .home_loan_years
        .trim()
        .parse::<i32>()
        .map_err(|_| "Invalid home loan term.".to_string())?; //jic if customer can enter 

    let (max_amount_cents, allowed_years): (i64, Vec<i32>) = match form.house_type.as_str() {
        "hdb_1_or_2_room" => (20_000_000, vec![5, 10, 15]),
        "hdb_3_or_larger" => (50_000_000, vec![10, 15, 20, 25]),
        "condo" => (100_000_000, vec![10, 15, 20, 25, 30]),
        "landed" => (200_000_000, vec![15, 20, 25, 30, 35]),
        _ => return Err("Invalid house type selected.".to_string()),
    };

   // if less customer shld borrow from personal loan
    if amount_cents < 2000000 {
        return Err("Home loan must at least be $20,000. Otherwise, kindly borrow through your personal loan channel.".to_string(),);
    }
   
    if amount_cents > max_amount_cents {
    return Err(format!(
        "Requested amount exceeds the maximum allowed. Maximum: {}.",
        Money::from_cents(max_amount_cents).display()
    ));
    }

    if !allowed_years.contains(&home_loan_years) {
        return Err("Selected loan duration is not allowed for this property type.".to_string());
    }

    let home_loan_duration_months: i32 = home_loan_years * 12;

    let application = home_loan_repository::create_application(
        db,
        user_id,
        account.id,
        &form.house_type,
        amount_cents,
        HOME_LOAN_INTEREST_RATE_BPS,
        home_loan_duration_months,
    )
    .await
    .map_err(|_| "Home loan application could not be submitted.".to_string())?;

Ok(application)
}


pub async fn approve_home_loan(
    db: &PgPool,
    staff_user_id: i64,
    application_id: i64,
) -> Result<HomeLoanApplication, String> {
    let application = home_loan_repository::find_by_id(db, application_id)
        .await
        .map_err(|_| "Could not load home loan application.".to_string())?
        .ok_or_else(|| "Home loan application not found.".to_string())?;

    if application.status != "pending_review" {
        return Err("Only pending home loan applications can be approved.".to_string());
    }

    let approved_amount_cents = application.requested_amount_cents;

    let home_loan_interest_cents = calculate_home_loan_interest_cents(
        approved_amount_cents,
        application.interest_rate_bps,
        application.term_months,
    );

    let home_loan_plus_interest_amount_cents = calculate_home_loan_plus_interest_amount(
        approved_amount_cents,
        home_loan_interest_cents,
    );

    let monthly_payment_cents = calculate_monthly_home_payment_cents(
        home_loan_plus_interest_amount_cents,
        application.term_months,
    );

    let (updated_application, _, _) = home_loan_repository::approve_application(
        db,
        application_id,
        staff_user_id,
        approved_amount_cents,
        home_loan_plus_interest_amount_cents,
        monthly_payment_cents,
    )
    .await
    .map_err(|_| "Home loan approval failed. Please try again.".to_string())?;

    Ok(updated_application)
}

pub async fn load_home_loan_dashboard(
    db: &PgPool,
    user_id: i64,
) -> Result<HomeLoanDashboard, String> {
    let account = account_repository::find_primary_account_by_user_id(db, user_id)
        .await
        .map_err(|_| "Could not load your bank account.".to_string())?
        .ok_or_else(|| "No active bank account found.".to_string())?;

    let summary = home_loan_repository::summary_by_user_id(db, user_id)
        .await
        .map_err(|_| "Could not load home loan summary.".to_string())?;

    let applications = home_loan_repository::list_by_user_id(db, user_id)
        .await
        .map_err(|_| "Could not load home loan applications.".to_string())?;

    Ok(HomeLoanDashboard {
        account,
        summary,
        applications,
    })
}

pub async fn list_all_home_loans_for_admin(
    db: &PgPool,
) -> Result<Vec<AdminHomeLoanRecord>, String> {
    home_loan_repository::list_all_for_admin(db)
        .await
        .map_err(|_| "Could not load home loan applications.".to_string())
}

pub async fn reject_home_loan(
    db: &PgPool,
    application_id: i64,
) -> Result<HomeLoanApplication, String> {
    home_loan_repository::reject_application(
        db,
        application_id,
        "Rejected by staff.",
    )
    .await
    .map_err(|_| "Home loan rejection failed. Please try again.".to_string())
}

pub async fn pay_home_loan(
    db: &PgPool,
    user_id: i64,
    application_id: i64,
    form: HomeLoanPaymentForm,
) -> Result<HomeLoanApplication, String> {
    let payment_cents = match form.amount {
        Some(value) if !value.trim().is_empty() => {
            Some(Money::parse_dollars(&value)?.cents())
        }
        _ => None,
    };

    home_loan_repository::pay_home_loan(db, user_id, application_id, payment_cents)
        .await
        .map_err(|_| {
            "Home loan repayment failed. Please check your balance or loan status.".to_string()
        })
}



fn calculate_home_loan_interest_cents(
    principal_home_loan_cents: i64,
    home_loan_interest_rate_bps: i32,
    home_loan_duration_months: i32,
) -> i64 {
    principal_home_loan_cents
        * home_loan_interest_rate_bps as i64
        * home_loan_duration_months as i64
        / 120000
}

fn calculate_home_loan_plus_interest_amount(
    principal_home_loan_cents: i64,
    home_loan_interest_cents: i64,
) -> i64 {
    principal_home_loan_cents + home_loan_interest_cents
}

fn calculate_monthly_home_payment_cents(
    home_loan_plus_interest_cents: i64,
    home_loan_duration_months: i32,
) -> i64 {
    (home_loan_plus_interest_cents + home_loan_duration_months as i64 - 1)
        / home_loan_duration_months as i64
}

