use crate::models::{BankAccount, Loan, Transaction};
use sqlx::{PgPool, Postgres, Transaction as DbTransaction};

pub async fn list_by_user_id(db: &PgPool, user_id: i64) -> Result<Vec<Loan>, sqlx::Error> {
    sqlx::query_as::<_, Loan>(
        r#"
        SELECT id, user_id, account_id, principal_cents, interest_rate_bps, interest_cents,
               total_repayment_cents, remaining_cents, monthly_payment_cents, term_months,
               next_due_date, status, created_at, updated_at
        FROM loans
        WHERE user_id = $1
        ORDER BY created_at DESC, id DESC
        "#,
    )
    .bind(user_id)
    .fetch_all(db)
    .await
}

pub async fn total_outstanding_by_user_id(db: &PgPool, user_id: i64) -> Result<i64, sqlx::Error> {
    let total: i64 = sqlx::query_scalar(
        r#"
        SELECT COALESCE(SUM(remaining_cents), 0)::BIGINT
        FROM loans
        WHERE user_id = $1 AND status = 'active'
        "#,
    )
    .bind(user_id)
    .fetch_one(db)
    .await?;

    Ok(total)
}

pub async fn create_loan(
    db: &PgPool,
    user_id: i64,
    account_id: i64,
    principal_cents: i64,
    interest_rate_bps: i32,
    interest_cents: i64,
    total_repayment_cents: i64,
    monthly_payment_cents: i64,
    term_months: i32,
) -> Result<(Loan, BankAccount, Transaction), sqlx::Error> {
    let mut tx = db.begin().await?;

    let account = lock_account_by_id(&mut tx, account_id, user_id).await?;
    let new_balance = account.balance_cents + principal_cents;

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

    let loan = sqlx::query_as::<_, Loan>(
        r#"
        INSERT INTO loans (
            user_id, account_id, principal_cents, interest_rate_bps, interest_cents,
            total_repayment_cents, remaining_cents, monthly_payment_cents,
            term_months, next_due_date, status
        )
        VALUES (
            $1, $2, $3, $4, $5,
            $6, $6, $7,
            $8, (CURRENT_DATE + make_interval(months => 1))::DATE, 'active'
        )
        RETURNING id, user_id, account_id, principal_cents, interest_rate_bps, interest_cents,
                  total_repayment_cents, remaining_cents, monthly_payment_cents, term_months,
                  next_due_date, status, created_at, updated_at
        "#,
    )
    .bind(user_id)
    .bind(account_id)
    .bind(principal_cents)
    .bind(interest_rate_bps)
    .bind(interest_cents)
    .bind(total_repayment_cents)
    .bind(monthly_payment_cents)
    .bind(term_months)
    .fetch_one(&mut *tx)
    .await?;

    let transaction = sqlx::query_as::<_, Transaction>(
        r#"
        INSERT INTO transactions (account_id, user_id, transaction_type, amount_cents, balance_after_cents, description)
        VALUES ($1, $2, 'loan_disbursement', $3, $4, $5)
        RETURNING id, account_id, user_id, transaction_type, amount_cents, balance_after_cents, description, created_at
        "#,
    )
    .bind(account_id)
    .bind(user_id)
    .bind(principal_cents)
    .bind(new_balance)
    .bind(format!("Personal loan #{} approved", loan.id))
    .fetch_one(&mut *tx)
    .await?;

    tx.commit().await?;
    Ok((loan, updated_account, transaction))
}

pub async fn pay_loan(
    db: &PgPool,
    user_id: i64,
    loan_id: i64,
    payment_cents: Option<i64>,
) -> Result<(Loan, BankAccount, Transaction), sqlx::Error> {
    let mut tx = db.begin().await?;

    let loan = lock_loan(&mut tx, user_id, loan_id).await?;
    let account = lock_account_by_id(&mut tx, loan.account_id, user_id).await?;

    let amount_to_pay = payment_cents.unwrap_or(loan.monthly_payment_cents).min(loan.remaining_cents);

    if account.balance_cents < amount_to_pay {
        return Err(sqlx::Error::RowNotFound);
    }

    let new_balance = account.balance_cents - amount_to_pay;
    let new_remaining = loan.remaining_cents - amount_to_pay;
    let final_status = if new_remaining <= 0 { "completed" } else { "active" };

    let months_covered = if amount_to_pay >= loan.monthly_payment_cents {
        (amount_to_pay / loan.monthly_payment_cents) as i32
    } else {
        0
    };

    let updated_loan = sqlx::query_as::<_, Loan>(
        r#"
        UPDATE loans
        SET remaining_cents = $1,
            status = $2,
            next_due_date = CASE
                WHEN $2 = 'completed' THEN next_due_date
                ELSE (next_due_date + make_interval(months => $3))::DATE
            END,
            updated_at = NOW()
        WHERE id = $4
        RETURNING id, user_id, account_id, principal_cents, interest_rate_bps, interest_cents,
                  total_repayment_cents, remaining_cents, monthly_payment_cents, term_months,
                  next_due_date, status, created_at, updated_at
        "#,
    )
    .bind(new_remaining)
    .bind(final_status)
    .bind(months_covered)
    .bind(loan_id)
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

    let transaction = sqlx::query_as::<_, Transaction>(
        r#"
        INSERT INTO transactions (account_id, user_id, transaction_type, amount_cents, balance_after_cents, description)
        VALUES ($1, $2, 'loan_repayment', $3, $4, $5)
        RETURNING id, account_id, user_id, transaction_type, amount_cents, balance_after_cents, description, created_at
        "#,
    )
    .bind(account.id)
    .bind(user_id)
    .bind(amount_to_pay)
    .bind(new_balance)
    .bind(format!("Personal loan #{} repayment", loan_id))
    .fetch_one(&mut *tx)
    .await?;

    tx.commit().await?;
    Ok((updated_loan, updated_account, transaction))
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

async fn lock_loan(
    tx: &mut DbTransaction<'_, Postgres>,
    user_id: i64,
    loan_id: i64,
) -> Result<Loan, sqlx::Error> {
    sqlx::query_as::<_, Loan>(
        r#"
        SELECT id, user_id, account_id, principal_cents, interest_rate_bps, interest_cents,
               total_repayment_cents, remaining_cents, monthly_payment_cents, term_months,
               next_due_date, status, created_at, updated_at
        FROM loans
        WHERE id = $1 AND user_id = $2 AND status = 'active'
        FOR UPDATE
        "#,
    )
    .bind(loan_id)
    .bind(user_id)
    .fetch_one(&mut **tx)
    .await
}

pub async fn has_three_month_overdue_loan(
    db: &PgPool,
    user_id: i64,
) -> Result<bool, sqlx::Error> {
    sqlx::query_scalar(
        r#"
        SELECT EXISTS (
            SELECT 1
            FROM loans
            WHERE user_id = $1
              AND status = 'active'
              AND next_due_date <= (CURRENT_DATE - INTERVAL '3 months')::DATE
        )
        "#,
    )
    .bind(user_id)
    .fetch_one(db)
    .await
}