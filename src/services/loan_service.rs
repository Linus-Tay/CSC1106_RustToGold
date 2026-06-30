use crate::forms::{LoanApplicationForm, LoanPaymentForm};
use crate::models::{Money, PersonalLoan, Product};
use crate::repositories::loan_repository;
use crate::services::support::clean_optional_text;
use sqlx::PgPool;
use uuid::Uuid;

pub struct LoanDashboard {
    pub account: Product,
    pub loans: Vec<PersonalLoan>,
}

pub async fn load_loan_dashboard(db: &PgPool, customer_id: Uuid) -> Result<LoanDashboard, String> {
    let account = loan_repository::find_primary_active_product(db, customer_id)
        .await
        .map_err(|_| "No active customer account was found for loan repayments.".to_string())?;

    let loans = loan_repository::list_personal_loans_by_customer(db, customer_id)
        .await
        .map_err(|_| "Could not load personal loans.".to_string())?;

    Ok(LoanDashboard { account, loans })
}

pub async fn apply_personal_loan(
    db: &PgPool,
    customer_id: Uuid,
    form: LoanApplicationForm,
) -> Result<PersonalLoan, String> {
    let amount = Money::parse_dollars(&form.amount)?;
    let purpose = clean_optional_text(&form.purpose)
        .filter(|value| !value.trim().is_empty())
        .ok_or_else(|| "Loan purpose is required.".to_string())?;

    let term_months = form.term_months;
    if !(6..=84).contains(&term_months) {
        return Err("Choose a loan term between 6 and 84 months.".to_string());
    }

    let account = loan_repository::find_primary_active_product(db, customer_id)
        .await
        .map_err(|_| "No active customer account was found to receive the loan.".to_string())?;

    let annual_rate_bps = 550;
    let monthly_payment_cents =
        estimated_monthly_payment_cents(amount.cents(), annual_rate_bps, term_months);

    loan_repository::create_personal_loan(
        db,
        customer_id,
        account.id,
        &purpose,
        amount.cents(),
        annual_rate_bps,
        term_months,
        monthly_payment_cents,
    )
    .await
    .map_err(|error| {
        println!("personal loan create failed: {}", error);
        "Could not create the personal loan.".to_string()
    })
}

pub async fn pay_personal_loan(
    db: &PgPool,
    customer_id: Uuid,
    loan_id: Uuid,
    form: LoanPaymentForm,
) -> Result<PersonalLoan, String> {
    let amount = Money::parse_dollars(&form.amount)?;
    let account = loan_repository::find_primary_active_product(db, customer_id)
        .await
        .map_err(|_| "No active customer account was found for repayment.".to_string())?;

    if account.balance_cents < amount.cents() {
        return Err("Insufficient balance for this repayment.".to_string());
    }

    loan_repository::pay_personal_loan(db, customer_id, loan_id, amount.cents())
        .await
        .map_err(|error| {
            println!("personal loan payment failed: {}", error);
            "Could not apply the personal loan repayment.".to_string()
        })
}

fn estimated_monthly_payment_cents(
    principal_cents: i64,
    annual_rate_bps: i32,
    term_months: i32,
) -> i64 {
    let months = term_months.max(1) as i64;
    let simple_interest = principal_cents
        .saturating_mul(annual_rate_bps as i64)
        .saturating_mul(months)
        / 12
        / 10_000;

    (principal_cents + simple_interest + months - 1) / months
}
