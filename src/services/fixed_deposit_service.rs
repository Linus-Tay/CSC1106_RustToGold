use crate::forms::{CreateFixedDepositForm, FixedDepositPlanForm};
use crate::models::{AccountWorkflow, BankAccount, FixedDeposit, FixedDepositCalculator, FixedDepositPlan, FixedDepositSummary, Money, SimpleFixedDepositCalculator};
use crate::repositories::{account_repository, fixed_deposit_repository};
use sqlx::PgPool;

#[derive(Debug, Clone)]
pub struct FixedDepositDashboardData {
    pub account: BankAccount,
    pub summary: FixedDepositSummary,
    pub fixed_deposits: Vec<FixedDeposit>,
    pub plans: Vec<FixedDepositPlan>,
}

pub async fn load_fixed_deposit_dashboard(
    db: &PgPool,
    user_id: i64,
) -> Result<FixedDepositDashboardData, String> {
    fixed_deposit_repository::refresh_matured_fixed_deposits(db)
        .await
        .map_err(|_| "Could not refresh fixed deposit maturity statuses.".to_string())?;

    let account = account_repository::find_primary_account_by_user_id(db, user_id)
        .await
        .map_err(|_| "Could not load your bank account.".to_string())?
        .ok_or_else(|| "No bank account was found for this customer.".to_string())?;

    let summary = fixed_deposit_repository::summary_by_user_id(db, user_id)
        .await
        .map_err(|_| "Could not load fixed deposit summary.".to_string())?;

    let fixed_deposits = fixed_deposit_repository::list_by_user_id(db, user_id)
        .await
        .map_err(|_| "Could not load fixed deposit records.".to_string())?;

    let plans = fixed_deposit_repository::list_active_plans(db)
        .await
        .map_err(|_| "Could not load fixed deposit plans.".to_string())?;

    Ok(FixedDepositDashboardData {
        account,
        summary,
        fixed_deposits,
        plans,
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
        .map_err(|_| "Could not load fixed deposit plans.".to_string())?;

    Ok((account, plans))
}

pub async fn create_fixed_deposit(
    db: &PgPool,
    user_id: i64,
    form: CreateFixedDepositForm,
) -> Result<FixedDeposit, String> {
    let amount = Money::parse_dollars(&form.amount)?;

    let account = account_repository::find_primary_account_by_user_id(db, user_id)
        .await
        .map_err(|_| "Could not load your bank account.".to_string())?
        .ok_or_else(|| "No bank account was found for this customer.".to_string())?;

    if !account.is_open_for_customer_actions() {
        return Err("Your account is not open for fixed deposit placement.".to_string());
    }

    if account.balance_cents < amount.cents() {
        return Err("Insufficient available balance for this fixed deposit.".to_string());
    }

    let plan = fixed_deposit_repository::find_plan_by_id(db, form.plan_id)
        .await
        .map_err(|_| "Could not load selected fixed deposit plan.".to_string())?
        .ok_or_else(|| "Selected fixed deposit plan does not exist.".to_string())?;

    if !plan.is_active() {
        return Err("Selected fixed deposit plan is not active.".to_string());
    }

    if amount.cents() < plan.minimum_amount_cents {
        return Err(format!(
            "Minimum amount for this plan is {}.",
            plan.minimum_amount_display()
        ));
    }

    let interest_cents = SimpleFixedDepositCalculator::calculate_interest_cents(
        amount.cents(),
        plan.interest_rate_bps,
        plan.duration_months,
    );
    let payout_cents = SimpleFixedDepositCalculator::calculate_matured_payout_cents(
        amount.cents(),
        interest_cents,
    );

    let (fixed_deposit, _, _) = fixed_deposit_repository::create_fixed_deposit(
        db,
        user_id,
        account.id,
        plan.id,
        amount.cents(),
        plan.interest_rate_bps,
        interest_cents,
        payout_cents,
        plan.duration_months,
    )
    .await
    .map_err(|_| "Fixed deposit creation failed. Please try again.".to_string())?;

    Ok(fixed_deposit)
}

pub async fn withdraw_fixed_deposit(
    db: &PgPool,
    user_id: i64,
    fixed_deposit_id: i64,
) -> Result<FixedDeposit, String> {
    fixed_deposit_repository::refresh_matured_fixed_deposits(db)
        .await
        .map_err(|_| "Could not refresh fixed deposit maturity statuses.".to_string())?;

    let (fixed_deposit, _, _) = fixed_deposit_repository::withdraw_fixed_deposit(db, user_id, fixed_deposit_id)
        .await
        .map_err(|_| "Fixed deposit withdrawal failed. It may already be withdrawn or paid out.".to_string())?;

    Ok(fixed_deposit)
}

pub async fn list_all_fixed_deposits(db: &PgPool) -> Result<Vec<FixedDeposit>, String> {
    fixed_deposit_repository::refresh_matured_fixed_deposits(db)
        .await
        .map_err(|_| "Could not refresh fixed deposit maturity statuses.".to_string())?;

    fixed_deposit_repository::list_all(db)
        .await
        .map_err(|_| "Could not load all fixed deposits.".to_string())
}

pub async fn list_all_fixed_deposit_plans(db: &PgPool) -> Result<Vec<FixedDepositPlan>, String> {
    fixed_deposit_repository::list_all_plans(db)
        .await
        .map_err(|_| "Could not load fixed deposit plans.".to_string())
}

pub async fn create_fixed_deposit_plan(
    db: &PgPool,
    form: FixedDepositPlanForm,
) -> Result<FixedDepositPlan, String> {
    let (name, duration_months, interest_rate_bps, minimum_amount_cents, status) = validate_plan_form(form)?;

    fixed_deposit_repository::create_plan(
        db,
        &name,
        duration_months,
        interest_rate_bps,
        minimum_amount_cents,
        &status,
    )
    .await
    .map_err(|_| "Could not create fixed deposit plan. The plan name may already exist.".to_string())
}

pub async fn update_fixed_deposit_plan(
    db: &PgPool,
    plan_id: i64,
    form: FixedDepositPlanForm,
) -> Result<FixedDepositPlan, String> {
    let (name, duration_months, interest_rate_bps, minimum_amount_cents, status) = validate_plan_form(form)?;

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
    .map_err(|_| "Could not update fixed deposit plan.".to_string())
}

fn validate_plan_form(form: FixedDepositPlanForm) -> Result<(String, i32, i32, i64, String), String> {
    let name = form.name.trim().to_string();
    if name.len() < 3 {
        return Err("Plan name must be at least 3 characters.".to_string());
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
        .map_err(|_| "Interest rate must be a valid number, for example 3.20.".to_string())?;

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

    Ok((name, duration_months, interest_rate_bps, minimum_amount.cents(), status))
}
