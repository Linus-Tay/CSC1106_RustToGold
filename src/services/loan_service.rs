use crate::forms::{LoanApplicationForm, LoanPaymentForm};
use crate::models::{BankAccount, Loan, Money};
use crate::repositories::{account_repository, loan_repository};
use sqlx::PgPool;


const LOAN_FIXED_INTEREST: i32 = 230;

pub async fn apply_personal_loan(
    db: &PgPool,
    user_id: i64,
    form: LoanApplicationForm,
) -> Result<Loan, String> {
    let principal = Money::parse_dollars(&form.amount)?;


    if principal.cents() < 50000 {
        return Err("Loan amount must be at least $500.00.".to_string());
    }

    let loan_duration_months = form
    .term_months
    .parse::<i32>()
    .map_err(|_| "Loan duration is invalid.".to_string())?;

    let account = account_repository::find_primary_account_by_user_id(db, user_id)
        .await
        .map_err(|_| "Your bank account could not be loaded".to_string())?
        .ok_or_else(|| "No bank account could be found under you. Please contact service.".to_string())?;

    if account.status != "active" {
        return Err("Your account is not active. Please activate before applying for a loan.".to_string());
    }
    

    let user_profile = crate::repositories::user_repository::find_user_by_id(db, user_id)
        .await
        .map_err(|_| "Could not load your profile.".to_string())?
        .ok_or_else(|| "Customer record not found.".to_string())?; 
  
        //MAS rule for one bank 
    let max_limit = user_profile.monthly_income_cents * 4;

    let outstanding_loan = loan_repository::total_outstanding_by_user_id(db, user_id)
        .await
        .map_err(|_| "Could not check your existing loans.".to_string())?;

    // NEED some safeguarding of bank interest
    let has_three_month_overdue = loan_repository::has_three_month_overdue_loan(db, user_id)
    .await
    .map_err(|_| "Could not check overdue loan status.".to_string())?;

    if has_three_month_overdue {
        return Err("You cannot apply for a new loan because you have a loan overdue by 3 months or more. Kindly proceed with the overdue payment before borrowing.".to_string());
    }

    let loan_available_limit = max_limit - outstanding_loan;

    if principal.cents() > loan_available_limit {
        return Err(format!(
            "You can only borrow up to {}.",
            Money::from_cents(std::cmp::max(loan_available_limit, 0)).display()
        ));
    }


    let loan_interest_amount_cents = calculate_personal_loan_interest_cents(
        principal.cents(),
        LOAN_FIXED_INTEREST,
        loan_duration_months,
    );

    let principal_plus_interest_amount_cents = calculate_loan_plus_interest_amount(
        principal.cents(), 
        loan_interest_amount_cents,
    );

    let monthly_payment_cents = calculate_monthly_loan_payment_cents(
        principal_plus_interest_amount_cents,
        loan_duration_months,
    );

   
    let loan = loan_repository::create_loan(
        db,
        user_id,
        account.id,
        principal.cents(),
        LOAN_FIXED_INTEREST,
        loan_interest_amount_cents,
        principal_plus_interest_amount_cents,
        monthly_payment_cents,
        loan_duration_months,
    )
    .await
    .map_err(|_| "Loan application failed. Please try again.".to_string())?;

    Ok(loan)
}


fn calculate_personal_loan_interest_cents(
    principal_loan_cents: i64,
    loan_interest_rate: i32,
    loan_duration_months: i32,
) -> i64 {
    (principal_loan_cents * loan_interest_rate as i64) * loan_duration_months as i64 /120000
} 

fn calculate_loan_plus_interest_amount(
   principal_loan_cents: i64,
    loan_interest_amount_cents: i64,
) -> i64 {
    principal_loan_cents + loan_interest_amount_cents
} 

fn calculate_monthly_loan_payment_cents(
    principal_plus_interest_amount_cents: i64,
    duration_of_loan_months: i32,
) -> i64 {
    (principal_plus_interest_amount_cents + duration_of_loan_months as i64 - 1)
        / duration_of_loan_months as i64
}

pub struct LoanDashboard {
    pub account: BankAccount,
    pub loans: Vec<Loan>,
}

pub async fn load_loan_dashboard(
    db: &PgPool,
    user_id: i64,
) -> Result<LoanDashboard, String> {
    let account = account_repository::find_primary_account_by_user_id(db, user_id)
        .await
        .map_err(|_| "Your bank account could not be loaded.".to_string())?
        .ok_or_else(|| "No bank account found.".to_string())?;

    let loans = loan_repository::list_by_user_id(db, user_id)
        .await
        .map_err(|_| "Could not load your loans.".to_string())?;

    Ok(LoanDashboard { account, loans })
}

pub async fn pay_loan(
    db: &PgPool,
    user_id: i64,
    loan_id: i64,
    form: LoanPaymentForm,
) -> Result<Loan, String> {
    let payment_cents = match form.amount {
        Some(value) if !value.trim().is_empty() => {
            Some(Money::parse_dollars(&value)?.cents())
        }
        _ => None,
    };

    loan_repository::pay_loan(db, user_id, loan_id, payment_cents)
        .await
        .map_err(|_| {
            "Loan repayment failed. Please check your balance or loan status.".to_string()
        })
}