use crate::forms::{HomeLoanApplicationForm, HomeLoanPaymentForm};
use crate::models::{AdminHomeLoanRecord, HomeLoanApplication, HomeLoanSummary, Money, Product};
use crate::repositories::{admin_repository, home_loan_repository, loan_repository};
use sqlx::PgPool;
use uuid::Uuid;

pub struct HomeLoanDashboard {
    pub account: Product,
    pub summary: HomeLoanSummary,
    pub applications: Vec<HomeLoanApplication>,
}

pub async fn load_home_loan_dashboard(
    db: &PgPool,
    customer_id: Uuid,
) -> Result<HomeLoanDashboard, String> {
    let account = loan_repository::find_primary_active_product(db, customer_id)
        .await
        .map_err(|_| "No active customer account was found for home loan repayments.".to_string())?;

    let applications = home_loan_repository::list_home_loans_by_customer(db, customer_id)
        .await
        .map_err(|_| "Could not load home loan applications.".to_string())?;

    let summary = HomeLoanSummary::from_applications(&applications);

    Ok(HomeLoanDashboard {
        account,
        summary,
        applications,
    })
}

pub async fn submit_home_loan_application(
    db: &PgPool,
    customer_id: Uuid,
    form: HomeLoanApplicationForm,
) -> Result<HomeLoanApplication, String> {
    let property_type = form.property_type.trim();
    if property_type.is_empty() {
        return Err("Property type is required.".to_string());
    }

    let property_value = Money::parse_dollars(&form.property_value)?;
    let down_payment = Money::parse_dollars(&form.down_payment)?;

    if down_payment.cents() >= property_value.cents() {
        return Err("Down payment must be lower than the property value.".to_string());
    }

    let term_years = form.term_years;
    if !(5..=35).contains(&term_years) {
        return Err("Choose a home loan term between 5 and 35 years.".to_string());
    }

    let account = loan_repository::find_primary_active_product(db, customer_id)
        .await
        .map_err(|_| "No active customer account was found for this application.".to_string())?;

    let loan_amount_cents = property_value.cents() - down_payment.cents();
    let annual_rate_bps = 325;
    let monthly_payment_cents = estimated_monthly_payment_cents(
        loan_amount_cents,
        annual_rate_bps,
        term_years * 12,
    );

    home_loan_repository::create_home_loan_application(
        db,
        customer_id,
        Some(account.id),
        property_type,
        property_value.cents(),
        down_payment.cents(),
        loan_amount_cents,
        annual_rate_bps,
        term_years,
        monthly_payment_cents,
    )
    .await
    .map_err(|error| {
        println!("home loan application failed: {}", error);
        "Could not submit the home loan application.".to_string()
    })
}

pub async fn pay_home_loan(
    db: &PgPool,
    customer_id: Uuid,
    application_id: Uuid,
    form: HomeLoanPaymentForm,
) -> Result<HomeLoanApplication, String> {
    let amount = Money::parse_dollars(&form.amount)?;
    let account = loan_repository::find_primary_active_product(db, customer_id)
        .await
        .map_err(|_| "No active customer account was found for repayment.".to_string())?;

    if account.balance_cents < amount.cents() {
        return Err("Insufficient balance for this home loan repayment.".to_string());
    }

    home_loan_repository::pay_home_loan(db, customer_id, application_id, amount.cents())
        .await
        .map_err(|error| {
            println!("home loan repayment failed: {}", error);
            "Could not apply the home loan repayment.".to_string()
        })
}

pub async fn list_admin_home_loans(db: &PgPool) -> Result<Vec<AdminHomeLoanRecord>, String> {
    admin_repository::list_home_loans(db)
        .await
        .map_err(|error| {
            eprintln!("admin home loan list failed: {error:?}");
            "Could not load home loan applications.".to_string()
        })
}

pub async fn approve_home_loan(
    db: &PgPool,
    staff_user_id: i64,
    application_id: Uuid,
) -> Result<HomeLoanApplication, String> {
    home_loan_repository::approve_home_loan(db, staff_user_id, application_id)
        .await
        .map_err(|error| {
            println!("home loan approve failed: {}", error);
            "Could not approve the home loan application.".to_string()
        })
}

pub async fn reject_home_loan(
    db: &PgPool,
    staff_user_id: i64,
    application_id: Uuid,
) -> Result<HomeLoanApplication, String> {
    home_loan_repository::reject_home_loan(db, staff_user_id, application_id)
        .await
        .map_err(|error| {
            println!("home loan reject failed: {}", error);
            "Could not reject the home loan application.".to_string()
        })
}

fn estimated_monthly_payment_cents(principal_cents: i64, annual_rate_bps: i32, term_months: i32) -> i64 {
    let months = term_months.max(1) as i64;
    let simple_interest = principal_cents
        .saturating_mul(annual_rate_bps as i64)
        .saturating_mul(months)
        / 12
        / 10_000;

    (principal_cents + simple_interest + months - 1) / months
}
