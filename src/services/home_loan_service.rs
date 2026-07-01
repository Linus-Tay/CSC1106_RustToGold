use crate::forms::{HomeLoanApplicationForm, HomeLoanPaymentForm};
use crate::models::{AdminHomeLoanRecord, HomeLoanApplication, HomeLoanSummary, Money, Product};
use crate::repositories::{admin_repository, home_loan_repository, product_repository};
use sqlx::PgPool;
use uuid::Uuid;

pub struct HomeLoanDashboard {
    pub account: Product,
    pub accounts: Vec<Product>,
    pub summary: HomeLoanSummary,
    pub applications: Vec<HomeLoanApplication>,
}

pub async fn load_home_loan_dashboard(
    db: &PgPool,
    customer_id: Uuid,
) -> Result<HomeLoanDashboard, String> {
    let accounts = product_repository::list_active_products_by_customer(db, &customer_id)
        .await
        .map_err(|_| "Could not load active customer accounts.".to_string())?;

    let account = accounts
        .first()
        .cloned()
        .ok_or_else(|| "No active customer account was found for home loan repayments.".to_string())?;

    let applications = home_loan_repository::list_home_loans_by_customer(db, customer_id)
        .await
        .map_err(|_| "Could not load home loan applications.".to_string())?;

    let summary = HomeLoanSummary::from_applications(&applications);

    Ok(HomeLoanDashboard {
        account,
        accounts,
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
    if property_value.cents() > 2_000_000_00 {
        return Err("Property value is above the allowed limit for this simulation.".to_string());
    }

    let down_payment_cents = property_value.cents() / 5;
    let down_payment = Money::from_cents(down_payment_cents);

    let term_years = form.term_years;
    if !(5..=35).contains(&term_years) {
        return Err("Choose a home loan term between 5 and 35 years.".to_string());
    }

    let account = product_repository::get_active_product_for_customer_by_account_number(
        db,
        &customer_id,
        form.account_number.trim(),
    )
    .await
    .map_err(|_| "Choose an active account for the required 20% down payment.".to_string())?;

    if account.balance_cents < down_payment.cents() {
        return Err(format!(
            "Home loan requires a 20% down payment of {}. Selected account balance is {}.",
            down_payment.display(),
            Money::from_cents(account.balance_cents).display()
        ));
    }

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

    let application = home_loan_repository::list_home_loans_by_customer(db, customer_id)
        .await
        .map_err(|_| "Could not load the home loan before repayment.".to_string())?
        .into_iter()
        .find(|application| application.id == application_id && application.status == "approved")
        .ok_or_else(|| "This home loan is not approved or cannot be repaid.".to_string())?;

    if amount.cents() > application.outstanding_cents {
        return Err(format!(
            "Repayment cannot exceed the outstanding home loan amount of {}.",
            Money::from_cents(application.outstanding_cents).display()
        ));
    }

    let account = product_repository::get_active_product_for_customer_by_account_number(
        db,
        &customer_id,
        form.account_number.trim(),
    )
    .await
    .map_err(|_| "Choose an active account for repayment.".to_string())?;

    if account.balance_cents < amount.cents() {
        return Err(format!(
            "Insufficient balance for this home loan repayment. Selected account balance is {}.",
            Money::from_cents(account.balance_cents).display()
        ));
    }

    home_loan_repository::pay_home_loan(db, customer_id, application_id, account.id, amount.cents())
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
    staff_user_id: Uuid,
    application_id: Uuid,
) -> Result<HomeLoanApplication, String> {
    let approved = home_loan_repository::approve_home_loan(db, staff_user_id, application_id)
        .await
        .map_err(|error| {
            println!("home loan approve failed: {}", error);
            "Could not approve the home loan application.".to_string()
        })?;

    let _ = admin_repository::record_audit_log(
        db,
        Some(staff_user_id),
        "approve_home_loan",
        "home_loan_application",
        Some(application_id.to_string()),
        Some("Home loan application approved".to_string()),
    )
    .await;

    Ok(approved)
}

pub async fn reject_home_loan(
    db: &PgPool,
    staff_user_id: Uuid,
    application_id: Uuid,
) -> Result<HomeLoanApplication, String> {
    let rejected = home_loan_repository::reject_home_loan(db, staff_user_id, application_id)
        .await
        .map_err(|error| {
            println!("home loan reject failed: {}", error);
            "Could not reject the home loan application.".to_string()
        })?;

    let _ = admin_repository::record_audit_log(
        db,
        Some(staff_user_id),
        "reject_home_loan",
        "home_loan_application",
        Some(application_id.to_string()),
        Some("Home loan application rejected".to_string()),
    )
    .await;

    Ok(rejected)
}

fn estimated_monthly_payment_cents(principal_cents: i64, annual_rate_bps: i32, term_months: i32) -> i64 {
    let months = term_months.max(1) as i64;
    let simple_interest = principal_cents
        .saturating_mul(annual_rate_bps as i64)
        .saturating_mul(term_months.max(1) as i64)
        / 12
        / 10_000;

    (principal_cents + simple_interest + months - 1) / months
}
