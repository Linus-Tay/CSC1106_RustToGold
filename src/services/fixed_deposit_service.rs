
use crate::forms::{CreateFixedDepositForm, FixedDepositPlanForm};
use crate::models::{
    FixedDeposit, FixedDepositAdminRecord, FixedDepositPlan, FixedDepositSummary, Money, Product,
};
use crate::repositories::{fixed_deposit_repository, product_repository};
use chrono::{Duration, Utc};
use sqlx::PgPool;
use uuid::Uuid;

pub struct FixedDepositDashboard {
    pub account: Product,
    pub accounts: Vec<Product>,
    pub summary: FixedDepositSummary,
    pub fixed_deposits: Vec<FixedDeposit>,
}

// Load load fixed deposit dashboard
pub async fn load_fixed_deposit_dashboard(
    db: &PgPool,
    customer_id: Uuid,
) -> Result<FixedDepositDashboard, String> {
    fixed_deposit_repository::mark_customer_matured(db, customer_id)
        .await
        .map_err(|_| "Could not refresh fixed deposit maturity statuses.".to_string())?;

    let accounts = product_repository::list_active_products_by_customer(db, &customer_id)
        .await
        .map_err(|_| "Could not load active customer accounts.".to_string())?;

    let account = accounts
        .first()
        .cloned()
        .ok_or_else(|| "No active customer account was found for fixed deposits.".to_string())?;

    let fixed_deposits = fixed_deposit_repository::list_fixed_deposits_by_customer(db, customer_id)
        .await
        .map_err(|_| "Could not load fixed deposits.".to_string())?;

    let summary = FixedDepositSummary::from_fixed_deposits(&fixed_deposits);

    Ok(FixedDepositDashboard {
        account,
        accounts,
        summary,
        fixed_deposits,
    })
}

// Load load fixed deposit create page
pub async fn load_fixed_deposit_create_page(
    db: &PgPool,
    customer_id: Uuid,
) -> Result<(Product, Vec<Product>, Vec<FixedDepositPlan>), String> {
    let accounts = product_repository::list_active_products_by_customer(db, &customer_id)
        .await
        .map_err(|_| "Could not load active customer accounts.".to_string())?;

    let account = accounts
        .first()
        .cloned()
        .ok_or_else(|| "No active customer account was found for fixed deposits.".to_string())?;

    let plans = fixed_deposit_repository::list_active_plans(db)
        .await
        .map_err(|_| "Could not load fixed deposit plans.".to_string())?;

    Ok((account, accounts, plans))
}

// Handle create fixed deposit
pub async fn create_fixed_deposit(
    db: &PgPool,
    customer_id: Uuid,
    form: CreateFixedDepositForm,
) -> Result<FixedDeposit, String> {
    let amount = Money::parse_dollars(&form.amount)
        .map_err(|message| format!("Placement amount: {message}"))?;
    let plan = fixed_deposit_repository::find_plan_by_id(db, form.plan_id)
        .await
        .map_err(|_| "Selected fixed deposit plan was not found.".to_string())?;

    if !plan.is_active {
        return Err("This fixed deposit plan is not active.".to_string());
    }

    if amount.cents() < plan.minimum_amount_cents {
        return Err(format!(
            "{} requires a minimum placement of {}. You entered {}.",
            plan.plan_name,
            Money::from_cents(plan.minimum_amount_cents).display(),
            amount.display()
        ));
    }

    let account = product_repository::get_active_product_for_customer_by_account_number(
        db,
        &customer_id,
        form.account_number.trim(),
    )
    .await
    .map_err(|_| "Choose an active funding account for this fixed deposit.".to_string())?;

    if account.balance_cents < amount.cents() {
        return Err(format!(
            "Insufficient balance. Your available balance is {}, but this placement is {}.",
            Money::from_cents(account.balance_cents).display(),
            amount.display()
        ));
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

// Handle withdraw fixed deposit
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

// Load list admin fixed deposits
pub async fn list_admin_fixed_deposits(
    db: &PgPool,
) -> Result<Vec<FixedDepositAdminRecord>, String> {
    fixed_deposit_repository::list_all_fixed_deposit_records(db)
        .await
        .map_err(|_| "Could not load fixed deposit records.".to_string())
}

// Load list admin plans
pub async fn list_admin_plans(db: &PgPool) -> Result<Vec<FixedDepositPlan>, String> {
    fixed_deposit_repository::list_all_plans(db)
        .await
        .map_err(|_| "Could not load fixed deposit plans.".to_string())
}

// Handle create plan
pub async fn create_plan(
    db: &PgPool,
    form: FixedDepositPlanForm,
) -> Result<FixedDepositPlan, String> {
    let plan_name = form.plan_name.trim();
    if plan_name.is_empty() {
        return Err("Plan name is required.".to_string());
    }

    let minimum_amount = Money::parse_dollars(&form.minimum_amount)
        .map_err(|message| format!("Minimum amount: {message}"))?;
    validate_plan_numbers(form.tenure_months, form.annual_rate_bps, minimum_amount.cents())?;

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

// Handle update plan
pub async fn update_plan(
    db: &PgPool,
    plan_id: i64,
    form: FixedDepositPlanForm,
) -> Result<FixedDepositPlan, String> {
    let plan_name = form.plan_name.trim();
    if plan_name.is_empty() {
        return Err("Plan name is required.".to_string());
    }

    let minimum_amount = Money::parse_dollars(&form.minimum_amount)
        .map_err(|message| format!("Minimum amount: {message}"))?;
    validate_plan_numbers(form.tenure_months, form.annual_rate_bps, minimum_amount.cents())?;

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

// Validate validate plan numbers
fn validate_plan_numbers(
    tenure_months: i32,
    annual_rate_bps: i32,
    minimum_amount_cents: i64,
) -> Result<(), String> {
    if !(1..=60).contains(&tenure_months) {
        return Err("Tenure must be between 1 and 60 months.".to_string());
    }

    if !(1..=1000).contains(&annual_rate_bps) {
        return Err("Annual rate must be between 0.01% and 10.00%.".to_string());
    }

    if minimum_amount_cents < 100_000 {
        return Err("Fixed deposit plan minimum amount must be at least $1000.00.".to_string());
    }

    Ok(())
}

// Process projected interest cents
fn projected_interest_cents(principal_cents: i64, annual_rate_bps: i32, tenure_months: i32) -> i64 {
    principal_cents
        .saturating_mul(annual_rate_bps as i64)
        .saturating_mul(tenure_months as i64)
        / 12
        / 10_000
}
