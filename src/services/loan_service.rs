// Service layer: keeps banking validation and workflow rules away from templates and SQL.

use crate::forms::{LoanApplicationForm, LoanPaymentForm};
use crate::models::{Money, PersonalLoan, Product};
use crate::repositories::{loan_repository, product_repository};
use crate::services::support::clean_optional_text;
use sqlx::PgPool;
use uuid::Uuid;

// Data carrier for the LoanDashboard workflow.
pub struct LoanDashboard {
    pub account: Product,
    pub accounts: Vec<Product>,
    pub loans: Vec<PersonalLoan>,
}

// Loads loan dashboard data and applies page-level business rules.
pub async fn load_loan_dashboard(db: &PgPool, customer_id: Uuid) -> Result<LoanDashboard, String> {
    let accounts = product_repository::list_active_products_by_customer(db, &customer_id)
        .await
        .map_err(|_| "Could not load active customer accounts.".to_string())?;

    let account = accounts
        .first()
        .cloned()
        .ok_or_else(|| "No active customer account was found for loan repayments.".to_string())?;

    let loans = loan_repository::list_personal_loans_by_customer(db, customer_id)
        .await
        .map_err(|_| "Could not load personal loans.".to_string())?;

    Ok(LoanDashboard { account, accounts, loans })
}

// Validates and coordinates the apply personal loan workflow.
pub async fn apply_personal_loan(
    db: &PgPool,
    customer_id: Uuid,
    form: LoanApplicationForm,
) -> Result<PersonalLoan, String> {
    let amount = Money::parse_dollars(&form.amount)?;
    if amount.cents() > 200_000_00 {
        return Err("Personal loan amount is above the allowed limit for this simulation.".to_string());
    }

    let purpose = clean_optional_text(&form.purpose)
        .filter(|value| !value.trim().is_empty())
        .ok_or_else(|| "Loan purpose is required.".to_string())?;

    let term_months = form.term_months;
    if !(6..=84).contains(&term_months) {
        return Err("Choose a loan term between 6 and 84 months.".to_string());
    }

    let account = product_repository::get_active_product_for_customer_by_account_number(
        db,
        &customer_id,
        form.account_number.trim(),
    )
    .await
    .map_err(|_| "Choose an active account to receive the loan disbursement.".to_string())?;

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

// Validates and coordinates the pay personal loan workflow.
pub async fn pay_personal_loan(
    db: &PgPool,
    customer_id: Uuid,
    loan_id: Uuid,
    form: LoanPaymentForm,
) -> Result<PersonalLoan, String> {
    let amount = Money::parse_dollars(&form.amount)?;

    let current_loan = loan_repository::list_personal_loans_by_customer(db, customer_id)
        .await
        .map_err(|_| "Could not load the personal loan before repayment.".to_string())?
        .into_iter()
        .find(|loan| loan.id == loan_id && loan.status == "active")
        .ok_or_else(|| "This personal loan is not active or cannot be repaid.".to_string())?;

    if amount.cents() > current_loan.outstanding_cents {
        return Err(format!(
            "Repayment cannot exceed the outstanding loan amount of {}.",
            Money::from_cents(current_loan.outstanding_cents).display()
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
            "Insufficient balance for this repayment. Selected account balance is {}.",
            Money::from_cents(account.balance_cents).display()
        ));
    }

    loan_repository::pay_personal_loan(db, customer_id, loan_id, account.id, amount.cents())
        .await
        .map_err(|error| {
            println!("personal loan payment failed: {}", error);
            "Could not apply the personal loan repayment.".to_string()
        })
}

// Returns the stored money amount in cents.
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
