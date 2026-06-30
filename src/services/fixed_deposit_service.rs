use crate::forms::{CreateFixedDepositForm, FixedDepositPlanForm};
use crate::models::{
    FixedDeposit, FixedDepositAdminRecord, FixedDepositPlan, FixedDepositSummary, Money, Product,
};
use crate::repositories::{fixed_deposit_repository, loan_repository};
use chrono::{Duration, Utc};
use sqlx::PgPool;
use uuid::Uuid;

pub struct FixedDepositDashboard {
    pub account: Product,
    pub summary: FixedDepositSummary,
    pub fixed_deposits: Vec<FixedDeposit>,
}

pub async fn load_fixed_deposit_dashboard(
    db: &PgPool,
    customer_id: Uuid,
) -> Result<FixedDepositDashboard, String> {
    fixed_deposit_repository::mark_customer_matured(db, customer_id)
        .await
        .map_err(|_| "Could not refresh fixed deposit maturity statuses.".to_string())?;

    let account = loan_repository::find_primary_active_product(db, customer_id)
        .await
        .map_err(|_| "No active customer account was found for fixed deposits.".to_string())?;

    let fixed_deposits = fixed_deposit_repository::list_fixed_deposits_by_customer(db, customer_id)
        .await
        .map_err(|_| "Could not load fixed deposits.".to_string())?;

    let summary = FixedDepositSummary::from_fixed_deposits(&fixed_deposits);

    Ok(FixedDepositDashboard {
        account,
        summary,
        fixed_deposits,
    })
}

pub async fn load_fixed_deposit_create_page(
    db: &PgPool,
    customer_id: Uuid,
) -> Result<(Product, Vec<FixedDepositPlan>), String> {
    let account = loan_repository::find_primary_active_product(db, customer_id)
        .await
        .map_err(|_| "No active customer account was found for fixed deposits.".to_string())?;

    let plans = fixed_deposit_repository::list_active_plans(db)
        .await
        .map_err(|_| "Could not load fixed deposit plans.".to_string())?;

    Ok((account, plans))
}

pub async fn create_fixed_deposit(
    db: &PgPool,
    customer_id: Uuid,
    form: CreateFixedDepositForm,
) -> Result<FixedDeposit, String> {
    let amount = Money::parse_dollars(&form.amount)?;
    let plan = fixed_deposit_repository::find_plan_by_id(db, form.plan_id)
        .await
        .map_err(|_| "Selected fixed deposit plan was not found.".to_string())?;

    if !plan.is_active {
        return Err("This fixed deposit plan is not active.".to_string());
    }

    if amount.cents() < plan.minimum_amount_cents {
        return Err(format!(
            "Minimum placement for this plan is {}.",
            Money::from_cents(plan.minimum_amount_cents).display()
        ));
    }

    let account = loan_repository::find_primary_active_product(db, customer_id)
        .await
        .map_err(|_| "No active customer account was found.".to_string())?;

    if account.balance_cents < amount.cents() {
        return Err("Insufficient balance to place this fixed deposit.".to_string());
    }

    let interest_cents =
        projected_interest_cents(amount.cents(), plan.annual_rate_bps, plan.tenure_months);
    let maturity_date = Utc::now().date_naive() + Duration::days((plan.tenure_months as i64) * 30);

    fixed_deposit_repository::create_fixed_deposit(
        db,
        customer_id,
        account.id,
        &plan,
        amount.cents(),
        interest_cents,
        maturity_date,
    )
    .await
    .map_err(|error| {
        println!("fixed deposit create failed: {}", error);
        "Could not create the fixed deposit.".to_string()
    })
}

pub async fn withdraw_fixed_deposit(
    db: &PgPool,
    customer_id: Uuid,
    fixed_deposit_id: Uuid,
) -> Result<String, String> {
    fixed_deposit_repository::withdraw_fixed_deposit(db, customer_id, fixed_deposit_id)
        .await
        .map_err(|error| {
            println!("fixed deposit withdrawal failed: {}", error);
            "Could not withdraw this fixed deposit.".to_string()
        })
}

pub async fn list_admin_fixed_deposits(
    db: &PgPool,
) -> Result<Vec<FixedDepositAdminRecord>, String> {
    fixed_deposit_repository::list_all_fixed_deposit_records(db)
        .await
        .map_err(|_| "Could not load fixed deposit records.".to_string())
}

pub async fn list_admin_plans(db: &PgPool) -> Result<Vec<FixedDepositPlan>, String> {
    fixed_deposit_repository::list_all_plans(db)
        .await
        .map_err(|_| "Could not load fixed deposit plans.".to_string())
}

pub async fn create_plan(
    db: &PgPool,
    form: FixedDepositPlanForm,
) -> Result<FixedDepositPlan, String> {
    let plan_name = form.plan_name.trim();
    if plan_name.is_empty() {
        return Err("Plan name is required.".to_string());
    }

    validate_plan_numbers(form.tenure_months, form.annual_rate_bps)?;
    let minimum_amount = Money::parse_dollars(&form.minimum_amount)?;

    fixed_deposit_repository::create_plan(
        db,
        plan_name,
        form.tenure_months,
        form.annual_rate_bps,
        minimum_amount.cents(),
        form.is_active.is_some(),
    )
    .await
    .map_err(|error| {
        println!("fixed deposit plan create failed: {}", error);
        "Could not create the fixed deposit plan.".to_string()
    })
}

pub async fn update_plan(
    db: &PgPool,
    plan_id: i64,
    form: FixedDepositPlanForm,
) -> Result<FixedDepositPlan, String> {
    let plan_name = form.plan_name.trim();
    if plan_name.is_empty() {
        return Err("Plan name is required.".to_string());
    }

    validate_plan_numbers(form.tenure_months, form.annual_rate_bps)?;
    let minimum_amount = Money::parse_dollars(&form.minimum_amount)?;

    fixed_deposit_repository::update_plan(
        db,
        plan_id,
        plan_name,
        form.tenure_months,
        form.annual_rate_bps,
        minimum_amount.cents(),
        form.is_active.is_some(),
    )
    .await
    .map_err(|error| {
        println!("fixed deposit plan update failed: {}", error);
        "Could not update the fixed deposit plan.".to_string()
    })
}

fn validate_plan_numbers(tenure_months: i32, annual_rate_bps: i32) -> Result<(), String> {
    if !(1..=60).contains(&tenure_months) {
        return Err("Tenure must be between 1 and 60 months.".to_string());
    }

    if !(1..=1000).contains(&annual_rate_bps) {
        return Err("Annual rate must be between 0.01% and 10.00%.".to_string());
    }

    Ok(())
}

fn projected_interest_cents(principal_cents: i64, annual_rate_bps: i32, tenure_months: i32) -> i64 {
    principal_cents
        .saturating_mul(annual_rate_bps as i64)
        .saturating_mul(tenure_months as i64)
        / 12
        / 10_000
}
