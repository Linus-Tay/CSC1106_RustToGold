use crate::forms::{CreateFixedDepositForm, FixedDepositPlanForm};
use crate::models::{
    AdminFixedDepositRecord, BankAccount, FixedDeposit, FixedDepositPlan, FixedDepositSummary,
    Money,
};
use crate::repositories::{account_repository, fixed_deposit_repository};
use sqlx::PgPool;

#[derive(Debug, Clone)]
pub struct FixedDepositDashboardData {
    pub account: BankAccount,
    pub summary: FixedDepositSummary,
    pub fixed_deposits: Vec<FixedDeposit>,
}

pub async fn load_fixed_deposit_dashboard(
    db: &PgPool,
    user_id: i64,
) -> Result<FixedDepositDashboardData, String> {
    fixed_deposit_repository::mark_matured_deposits(db)
        .await
        .map_err(|_| "Could not refresh fixed deposit maturity statuses.".to_string())?;

    let account = account_repository::find_primary_account_by_user_id(db, user_id)
        .await
        .map_err(|_| "Could not load your bank account.".to_string())?
        .ok_or_else(|| "No bank account was found for this customer.".to_string())?;

    let summary = fixed_deposit_repository::summary_by_user_id(db, user_id)
        .await
        .map_err(|_| "Could not load the fixed deposit summary.".to_string())?;

    let fixed_deposits = fixed_deposit_repository::list_by_user_id(db, user_id)
        .await
        .map_err(|_| "Could not load fixed deposit records.".to_string())?;

    Ok(FixedDepositDashboardData {
        account,
        summary,
        fixed_deposits,
    })
}

pub async fn load_create_fixed_deposit_page(
    db: &PgPool,
    user_id: i64,
) -> Result<(BankAccount, Vec<FixedDepositPlan>), String> {
    let account = account_repository::find_primary_account_by_user_id(db, user_id)
        .await
        .map_err(|_| "Could not load your bank account.".to_string())?
        .ok_or_else(|| "No bank account was found for this customer.".to_string())?;

    let plans = fixed_deposit_repository::list_active_plans(db)
        .await
        .map_err(|_| "Could not load active fixed deposit plans.".to_string())?;

    Ok((account, plans))
}

pub async fn create_fixed_deposit(
    db: &PgPool,
    user_id: i64,
    form: CreateFixedDepositForm,
) -> Result<FixedDeposit, String> {
    let plan_id = form
        .plan_id
        .trim()
        .parse::<i64>()
        .map_err(|_| "Please select a valid fixed deposit plan.".to_string())?;
    let amount = Money::parse_dollars(&form.amount)?;

    let account = account_repository::find_primary_account_by_user_id(db, user_id)
        .await
        .map_err(|_| "Could not load your bank account.".to_string())?
        .ok_or_else(|| "No bank account was found for this customer.".to_string())?;

    if account.status != "active" {
        return Err("Your bank account is not active for fixed deposit placement.".to_string());
    }

    if account.balance_cents < amount.cents() {
        return Err("Insufficient available balance for this fixed deposit.".to_string());
    }

    let plan = fixed_deposit_repository::find_plan_by_id(db, plan_id)
        .await
        .map_err(|_| "Could not load the selected fixed deposit plan.".to_string())?
        .ok_or_else(|| "The selected fixed deposit plan does not exist.".to_string())?;

    if !plan.is_active() {
        return Err("The selected fixed deposit plan is no longer active.".to_string());
    }

    if amount.cents() < plan.minimum_amount_cents {
        return Err(format!(
            "The minimum amount for {} is {}.",
            plan.name,
            plan.minimum_amount_display()
        ));
    }

    let interest_cents = calculate_simple_interest_cents(
        amount.cents(),
        plan.interest_rate_bps,
        plan.duration_months,
    );
    let expected_payout_cents = amount.cents() + interest_cents;

    fixed_deposit_repository::create_fixed_deposit(
        db,
        user_id,
        account.id,
        plan.id,
        amount.cents(),
        plan.interest_rate_bps,
        interest_cents,
        expected_payout_cents,
        plan.duration_months,
    )
    .await
    .map_err(|_| "Fixed deposit creation failed. Please try again.".to_string())
}

pub async fn withdraw_fixed_deposit(
    db: &PgPool,
    user_id: i64,
    fixed_deposit_id: i64,
) -> Result<FixedDeposit, String> {
    fixed_deposit_repository::mark_matured_deposits(db)
        .await
        .map_err(|_| "Could not refresh fixed deposit maturity statuses.".to_string())?;

    fixed_deposit_repository::withdraw_fixed_deposit(db, user_id, fixed_deposit_id)
        .await
        .map_err(|_| {
            "Withdrawal failed. This fixed deposit may already be withdrawn or paid out.".to_string()
        })
}

pub async fn list_all_fixed_deposit_records(
    db: &PgPool,
) -> Result<Vec<AdminFixedDepositRecord>, String> {
    fixed_deposit_repository::mark_matured_deposits(db)
        .await
        .map_err(|_| "Could not refresh fixed deposit maturity statuses.".to_string())?;

    fixed_deposit_repository::list_all_for_admin(db)
        .await
        .map_err(|_| "Could not load fixed deposit records.".to_string())
}

pub async fn list_all_fixed_deposit_plans(
    db: &PgPool,
) -> Result<Vec<FixedDepositPlan>, String> {
    fixed_deposit_repository::list_all_plans(db)
        .await
        .map_err(|_| "Could not load fixed deposit plans.".to_string())
}

pub async fn create_fixed_deposit_plan(
    db: &PgPool,
    form: FixedDepositPlanForm,
) -> Result<FixedDepositPlan, String> {
    let (name, duration_months, interest_rate_bps, minimum_amount_cents, status) =
        validate_plan_form(form)?;

    fixed_deposit_repository::create_plan(
        db,
        &name,
        duration_months,
        interest_rate_bps,
        minimum_amount_cents,
        &status,
    )
    .await
    .map_err(|_| "Could not create the plan. The plan name may already exist.".to_string())
}

pub async fn update_fixed_deposit_plan(
    db: &PgPool,
    plan_id: i64,
    form: FixedDepositPlanForm,
) -> Result<FixedDepositPlan, String> {
    let (name, duration_months, interest_rate_bps, minimum_amount_cents, status) =
        validate_plan_form(form)?;

    fixed_deposit_repository::update_plan(
        db,
        plan_id,
        &name,
        duration_months,
        interest_rate_bps,
        minimum_amount_cents,
        &status,
    )
    .await
    .map_err(|_| "Could not update this fixed deposit plan.".to_string())
}

// Simple interest: principal x annual rate x months / 12.
fn calculate_simple_interest_cents(
    principal_cents: i64,
    interest_rate_bps: i32,
    duration_months: i32,
) -> i64 {
    principal_cents * interest_rate_bps as i64 * duration_months as i64 / 120_000
}

fn validate_plan_form(
    form: FixedDepositPlanForm,
) -> Result<(String, i32, i32, i64, String), String> {
    let name = form.name.trim().to_string();
    if name.len() < 3 {
        return Err("Plan name must have at least 3 characters.".to_string());
    }

    let duration_months = form
        .duration_months
        .trim()
        .parse::<i32>()
        .map_err(|_| "Duration must be a whole number of months.".to_string())?;
    if !(1..=60).contains(&duration_months) {
        return Err("Duration must be between 1 and 60 months.".to_string());
    }

    let interest_rate = form
        .interest_rate
        .trim()
        .parse::<f64>()
        .map_err(|_| "Interest rate must be a number, for example 3.20.".to_string())?;
    if !(0.01..=20.0).contains(&interest_rate) {
        return Err("Interest rate must be between 0.01% and 20.00%.".to_string());
    }
    let interest_rate_bps = (interest_rate * 100.0).round() as i32;

    let minimum_amount = Money::parse_dollars(&form.minimum_amount)?;

    let status = match form.status.trim() {
        "active" => "active".to_string(),
        "inactive" => "inactive".to_string(),
        _ => return Err("Plan status must be active or inactive.".to_string()),
    };

    Ok((
        name,
        duration_months,
        interest_rate_bps,
        minimum_amount.cents(),
        status,
    ))
}
