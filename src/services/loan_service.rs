use crate::forms::{LoanApplicationForm, LoanPaymentForm};
use crate::models::{BankAccount, Loan, Money, SimpleLoanCalculator};
use crate::repositories::{account_repository, loan_repository};
use sqlx::PgPool;

const FIXED_INTEREST_RATE_BPS: i32 = 230;

#[derive(Debug, Clone)]
pub struct LoanDashboardData {
    pub account: BankAccount,
    pub loans: Vec<Loan>,
}

pub async fn load_loan_dashboard(
    db: &PgPool,
    user_id: i64,
) -> Result<LoanDashboardData, String> {
    let account = account_repository::find_primary_account_by_user_id(db, user_id)
        .await
        .map_err(|_| "Could not load your bank account.".to_string())?
        .ok_or_else(|| "No bank account was found for this customer.".to_string())?;

    let loans = loan_repository::list_by_user_id(db, user_id)
        .await
        .map_err(|_| "Could not load loan records.".to_string())?;

    Ok(LoanDashboardData { account, loans })
}

pub async fn apply_personal_loan(
    db: &PgPool,
    user_id: i64,
    form: LoanApplicationForm,
) -> Result<Loan, String> {
    let amount = Money::parse_dollars(&form.amount)?;

    let has_long_overdue = loan_repository::has_three_month_overdue_loan(db, user_id)
    .await
    .map_err(|_| "Could not check overdue loan status.".to_string())?;

    if has_long_overdue {
        return Err(
             "You cannot apply for a new loan because you have a loan overdue by 3 months or more."
               .to_string(),
              );
        }

    if amount.cents() < 100 {
        return Err("Loan amount must be at least $1.00.".to_string());
    }

    let term_months = form
        .term_months
        .trim()
        .parse::<i32>()
        .map_err(|_| "Loan term must be valid.".to_string())?;

    if ![6, 12, 24, 36].contains(&term_months) {
        return Err("Loan term must be 6, 12, 24, or 36 months.".to_string());
    }

    let user_profile = crate::repositories::user_repository::find_user_by_id(db, user_id)
        .await
        .map_err(|_| "Could not verify customer profile.".to_string())?
        .ok_or_else(|| "Customer record not found.".to_string())?;

    let max_limit = Loan::calculate_max_borrowing_limit(user_profile.monthly_income_cents);

    let outstanding = loan_repository::total_outstanding_by_user_id(db, user_id)
        .await
        .map_err(|_| "Could not check existing loan balance.".to_string())?;

    let available_limit = max_limit - outstanding;

    if amount.cents() > available_limit {
        return Err(format!(
            "You can only borrow up to {} based on your salary and current active loans.",
            Money::from_cents(std::cmp::max(available_limit, 0)).display()
        ));
    }

    let account = account_repository::find_primary_account_by_user_id(db, user_id)
        .await
        .map_err(|_| "Could not load your bank account.".to_string())?
        .ok_or_else(|| "No bank account was found for this customer.".to_string())?;

    let interest_cents = SimpleLoanCalculator::calculate_interest_cents(
        amount.cents(),
        FIXED_INTEREST_RATE_BPS,
        term_months,
    );

    let total_repayment_cents = amount.cents() + interest_cents;

    let monthly_payment_cents =
        SimpleLoanCalculator::calculate_monthly_payment_cents(total_repayment_cents, term_months);

    let (loan, _, _) = loan_repository::create_loan(
        db,
        user_id,
        account.id,
        amount.cents(),
        FIXED_INTEREST_RATE_BPS,
        interest_cents,
        total_repayment_cents,
        monthly_payment_cents,
        term_months,
    )
    .await
    .map_err(|_| "Loan application failed. Please try again.".to_string())?;

    Ok(loan)
}

pub async fn pay_loan(
    db: &PgPool,
    user_id: i64,
    loan_id: i64,
    form: LoanPaymentForm,
) -> Result<Loan, String> {
    let payment_cents = match form.amount {
        Some(value) if !value.trim().is_empty() => Some(Money::parse_dollars(&value)?.cents()),
        _ => None,
    };

    let (loan, _, _) = loan_repository::pay_loan(db, user_id, loan_id, payment_cents)
        .await
        .map_err(|_| "Loan payment failed. Please check your balance or loan status.".to_string())?;

    Ok(loan)
}