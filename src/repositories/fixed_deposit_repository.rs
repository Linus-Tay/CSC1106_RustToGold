use crate::models::{BankAccount, FixedDeposit, FixedDepositPlan, FixedDepositSummary, Transaction};
use sqlx::{PgPool, Postgres, Transaction as DbTransaction};

pub async fn seed_default_plans(db: &PgPool) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        INSERT INTO fixed_deposit_plans (name, duration_months, interest_rate_bps, minimum_amount_cents, status)
        VALUES
            ('3 Month Starter FD', 3, 180, 50000, 'active'),
            ('6 Month Growth FD', 6, 250, 100000, 'active'),
            ('12 Month Premium FD', 12, 320, 200000, 'active'),
            ('24 Month Wealth FD', 24, 380, 500000, 'active')
        ON CONFLICT (name) DO NOTHING
        "#,
    )
    .execute(db)
    .await?;

    Ok(())
}

pub async fn list_active_plans(db: &PgPool) -> Result<Vec<FixedDepositPlan>, sqlx::Error> {
    sqlx::query_as::<_, FixedDepositPlan>(
        r#"
        SELECT id, name, duration_months, interest_rate_bps, minimum_amount_cents, status, created_at, updated_at
        FROM fixed_deposit_plans
        WHERE status = 'active'
        ORDER BY duration_months ASC, minimum_amount_cents ASC
        "#,
    )
    .fetch_all(db)
    .await
}

pub async fn list_all_plans(db: &PgPool) -> Result<Vec<FixedDepositPlan>, sqlx::Error> {
    sqlx::query_as::<_, FixedDepositPlan>(
        r#"
        SELECT id, name, duration_months, interest_rate_bps, minimum_amount_cents, status, created_at, updated_at
        FROM fixed_deposit_plans
        ORDER BY duration_months ASC, id ASC
        "#,
    )
    .fetch_all(db)
    .await
}

pub async fn find_plan_by_id(db: &PgPool, plan_id: i64) -> Result<Option<FixedDepositPlan>, sqlx::Error> {
    sqlx::query_as::<_, FixedDepositPlan>(
        r#"
        SELECT id, name, duration_months, interest_rate_bps, minimum_amount_cents, status, created_at, updated_at
        FROM fixed_deposit_plans
        WHERE id = $1
        "#,
    )
    .bind(plan_id)
    .fetch_optional(db)
    .await
}

pub async fn create_plan(
    db: &PgPool,
    name: &str,
    duration_months: i32,
    interest_rate_bps: i32,
    minimum_amount_cents: i64,
    status: &str,
) -> Result<FixedDepositPlan, sqlx::Error> {
    sqlx::query_as::<_, FixedDepositPlan>(
        r#"
        INSERT INTO fixed_deposit_plans (name, duration_months, interest_rate_bps, minimum_amount_cents, status)
        VALUES ($1, $2, $3, $4, $5)
        RETURNING id, name, duration_months, interest_rate_bps, minimum_amount_cents, status, created_at, updated_at
        "#,
    )
    .bind(name)
    .bind(duration_months)
    .bind(interest_rate_bps)
    .bind(minimum_amount_cents)
    .bind(status)
    .fetch_one(db)
    .await
}

pub async fn update_plan(
    db: &PgPool,
    plan_id: i64,
    name: &str,
    duration_months: i32,
    interest_rate_bps: i32,
    minimum_amount_cents: i64,
    status: &str,
) -> Result<FixedDepositPlan, sqlx::Error> {
    sqlx::query_as::<_, FixedDepositPlan>(
        r#"
        UPDATE fixed_deposit_plans
        SET name = $1,
            duration_months = $2,
            interest_rate_bps = $3,
            minimum_amount_cents = $4,
            status = $5,
            updated_at = NOW()
        WHERE id = $6
        RETURNING id, name, duration_months, interest_rate_bps, minimum_amount_cents, status, created_at, updated_at
        "#,
    )
    .bind(name)
    .bind(duration_months)
    .bind(interest_rate_bps)
    .bind(minimum_amount_cents)
    .bind(status)
    .bind(plan_id)
    .fetch_one(db)
    .await
}

pub async fn refresh_matured_fixed_deposits(db: &PgPool) -> Result<u64, sqlx::Error> {
    let result = sqlx::query(
        r#"
        UPDATE fixed_deposits
        SET status = 'matured', updated_at = NOW()
        WHERE status = 'active' AND maturity_date <= CURRENT_DATE
        "#,
    )
    .execute(db)
    .await?;

    Ok(result.rows_affected())
}

pub async fn list_by_user_id(db: &PgPool, user_id: i64) -> Result<Vec<FixedDeposit>, sqlx::Error> {
    sqlx::query_as::<_, FixedDeposit>(
        r#"
        SELECT id, user_id, account_id, plan_id, principal_cents, interest_rate_bps, interest_cents,
               penalty_cents, payout_cents, start_date, maturity_date, status, created_at, updated_at
        FROM fixed_deposits
        WHERE user_id = $1
        ORDER BY created_at DESC, id DESC
        "#,
    )
    .bind(user_id)
    .fetch_all(db)
    .await
}

pub async fn list_all(db: &PgPool) -> Result<Vec<FixedDeposit>, sqlx::Error> {
    sqlx::query_as::<_, FixedDeposit>(
        r#"
        SELECT id, user_id, account_id, plan_id, principal_cents, interest_rate_bps, interest_cents,
               penalty_cents, payout_cents, start_date, maturity_date, status, created_at, updated_at
        FROM fixed_deposits
        ORDER BY created_at DESC, id DESC
        "#,
    )
    .fetch_all(db)
    .await
}

pub async fn summary_by_user_id(db: &PgPool, user_id: i64) -> Result<FixedDepositSummary, sqlx::Error> {
    sqlx::query_as::<_, FixedDepositSummary>(
        r#"
        SELECT
            COUNT(*)::BIGINT AS total_count,
            COUNT(*) FILTER (WHERE status = 'active')::BIGINT AS active_count,
            COUNT(*) FILTER (WHERE status = 'matured')::BIGINT AS matured_count,
            COALESCE(SUM(principal_cents) FILTER (WHERE status IN ('active', 'matured')), 0)::BIGINT AS total_principal_cents,
            COALESCE(SUM(interest_cents) FILTER (WHERE status IN ('active', 'matured')), 0)::BIGINT AS total_interest_cents,
            COALESCE(SUM(payout_cents) FILTER (WHERE status IN ('active', 'matured')), 0)::BIGINT AS total_payout_cents
        FROM fixed_deposits
        WHERE user_id = $1
        "#,
    )
    .bind(user_id)
    .fetch_one(db)
    .await
}

pub async fn create_fixed_deposit(
    db: &PgPool,
    user_id: i64,
    account_id: i64,
    plan_id: i64,
    principal_cents: i64,
    interest_rate_bps: i32,
    interest_cents: i64,
    payout_cents: i64,
    duration_months: i32,
) -> Result<(FixedDeposit, BankAccount, Transaction), sqlx::Error> {
    let mut tx = db.begin().await?;
    let account = lock_account_by_id(&mut tx, account_id, user_id).await?;
    let new_balance = account.balance_cents - principal_cents;

    let updated_account = sqlx::query_as::<_, BankAccount>(
        r#"
        UPDATE bank_accounts
        SET balance_cents = $1, updated_at = NOW()
        WHERE id = $2
        RETURNING id, user_id, account_number, account_type, balance_cents, status, created_at, updated_at
        "#,
    )
    .bind(new_balance)
    .bind(account_id)
    .fetch_one(&mut *tx)
    .await?;

    let fixed_deposit = sqlx::query_as::<_, FixedDeposit>(
        r#"
        INSERT INTO fixed_deposits (
            user_id, account_id, plan_id, principal_cents, interest_rate_bps, interest_cents,
            penalty_cents, payout_cents, start_date, maturity_date, status
        )
        VALUES (
            $1, $2, $3, $4, $5, $6,
            0, $7, CURRENT_DATE, (CURRENT_DATE + make_interval(months => $8))::DATE, 'active'
        )
        RETURNING id, user_id, account_id, plan_id, principal_cents, interest_rate_bps, interest_cents,
                  penalty_cents, payout_cents, start_date, maturity_date, status, created_at, updated_at
        "#,
    )
    .bind(user_id)
    .bind(account_id)
    .bind(plan_id)
    .bind(principal_cents)
    .bind(interest_rate_bps)
    .bind(interest_cents)
    .bind(payout_cents)
    .bind(duration_months)
    .fetch_one(&mut *tx)
    .await?;

    let transaction = sqlx::query_as::<_, Transaction>(
        r#"
        INSERT INTO transactions (account_id, user_id, transaction_type, amount_cents, balance_after_cents, description)
        VALUES ($1, $2, 'fixed_deposit_opening', $3, $4, $5)
        RETURNING id, account_id, user_id, transaction_type, amount_cents, balance_after_cents, description, created_at
        "#,
    )
    .bind(account_id)
    .bind(user_id)
    .bind(principal_cents)
    .bind(new_balance)
    .bind(format!("Fixed deposit #{} created", fixed_deposit.id))
    .fetch_one(&mut *tx)
    .await?;

    tx.commit().await?;
    Ok((fixed_deposit, updated_account, transaction))
}

pub async fn withdraw_fixed_deposit(
    db: &PgPool,
    user_id: i64,
    fixed_deposit_id: i64,
) -> Result<(FixedDeposit, BankAccount, Transaction), sqlx::Error> {
    let mut tx = db.begin().await?;
    let fixed_deposit = lock_fixed_deposit(&mut tx, user_id, fixed_deposit_id).await?;
    let account = lock_account_by_id(&mut tx, fixed_deposit.account_id, user_id).await?;

    let is_matured = fixed_deposit.status == "matured" || fixed_deposit.maturity_date <= chrono::Utc::now().date_naive();
    let penalty_cents = if is_matured { 0 } else { fixed_deposit.interest_cents };
    let payout_cents = if is_matured {
        fixed_deposit.principal_cents + fixed_deposit.interest_cents
    } else {
        fixed_deposit.principal_cents
    };
    let final_status = if is_matured { "paid_out" } else { "withdrawn" };
    let transaction_type = if is_matured { "fixed_deposit_payout" } else { "fixed_deposit_early_withdrawal" };
    let new_balance = account.balance_cents + payout_cents;

    let updated_fixed_deposit = sqlx::query_as::<_, FixedDeposit>(
        r#"
        UPDATE fixed_deposits
        SET penalty_cents = $1,
            payout_cents = $2,
            status = $3,
            updated_at = NOW()
        WHERE id = $4
        RETURNING id, user_id, account_id, plan_id, principal_cents, interest_rate_bps, interest_cents,
                  penalty_cents, payout_cents, start_date, maturity_date, status, created_at, updated_at
        "#,
    )
    .bind(penalty_cents)
    .bind(payout_cents)
    .bind(final_status)
    .bind(fixed_deposit_id)
    .fetch_one(&mut *tx)
    .await?;

    let updated_account = sqlx::query_as::<_, BankAccount>(
        r#"
        UPDATE bank_accounts
        SET balance_cents = $1, updated_at = NOW()
        WHERE id = $2
        RETURNING id, user_id, account_number, account_type, balance_cents, status, created_at, updated_at
        "#,
    )
    .bind(new_balance)
    .bind(account.id)
    .fetch_one(&mut *tx)
    .await?;

    let description = if is_matured {
        format!("Fixed deposit #{} matured payout", fixed_deposit_id)
    } else {
        format!("Fixed deposit #{} early withdrawal", fixed_deposit_id)
    };

    let transaction = sqlx::query_as::<_, Transaction>(
        r#"
        INSERT INTO transactions (account_id, user_id, transaction_type, amount_cents, balance_after_cents, description)
        VALUES ($1, $2, $3, $4, $5, $6)
        RETURNING id, account_id, user_id, transaction_type, amount_cents, balance_after_cents, description, created_at
        "#,
    )
    .bind(account.id)
    .bind(user_id)
    .bind(transaction_type)
    .bind(payout_cents)
    .bind(new_balance)
    .bind(description)
    .fetch_one(&mut *tx)
    .await?;

    tx.commit().await?;
    Ok((updated_fixed_deposit, updated_account, transaction))
}

async fn lock_account_by_id(
    tx: &mut DbTransaction<'_, Postgres>,
    account_id: i64,
    user_id: i64,
) -> Result<BankAccount, sqlx::Error> {
    sqlx::query_as::<_, BankAccount>(
        r#"
        SELECT id, user_id, account_number, account_type, balance_cents, status, created_at, updated_at
        FROM bank_accounts
        WHERE id = $1 AND user_id = $2 AND status = 'active'
        FOR UPDATE
        "#,
    )
    .bind(account_id)
    .bind(user_id)
    .fetch_one(&mut **tx)
    .await
}

async fn lock_fixed_deposit(
    tx: &mut DbTransaction<'_, Postgres>,
    user_id: i64,
    fixed_deposit_id: i64,
) -> Result<FixedDeposit, sqlx::Error> {
    sqlx::query_as::<_, FixedDeposit>(
        r#"
        SELECT id, user_id, account_id, plan_id, principal_cents, interest_rate_bps, interest_cents,
               penalty_cents, payout_cents, start_date, maturity_date, status, created_at, updated_at
        FROM fixed_deposits
        WHERE id = $1 AND user_id = $2 AND status IN ('active', 'matured')
        FOR UPDATE
        "#,
    )
    .bind(fixed_deposit_id)
    .bind(user_id)
    .fetch_one(&mut **tx)
    .await
}
