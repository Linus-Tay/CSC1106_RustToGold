// ======================================================================
// STAFF DASHBOARD INTEGRATION
//
// APPROVE:
// Call approve_application(application_id, staff_user_id, ...).
//
// This function automatically:
//  - changes status to approved
//  - credits loan amount into customer's account
//  - creates loan_disbursement transaction
//  - generates repayment schedule
//  - sets monthly payment and next due date
//
// REJECT:
// Do NOT call approve_application().
// Simply update:
//
// status = "rejected"
// staff_remarks = "..."
// updated_at = NOW()
//
// Rejected applications must NOT:
//  - credit money
//  - create transactions
//  - generate repayment schedule
//
// ======================================================================
use crate::models::{BankAccount, HomeLoanApplication, Transaction};
use sqlx::PgPool;

pub async fn list_by_user_id(
    db: &PgPool,
    user_id: i64,
) -> Result<Vec<HomeLoanApplication>, sqlx::Error> {
    sqlx::query_as::<_, HomeLoanApplication>(
        r#"
        SELECT id, user_id, account_id, house_type, requested_amount_cents,
               interest_rate_bps, term_months, status, staff_remarks,
               created_at, updated_at, approved_amount_cents, approved_by, approved_at,
               total_repayment_cents, remaining_cents, monthly_payment_cents, next_due_date
        FROM home_loan_applications
        WHERE user_id = $1
        ORDER BY created_at DESC, id DESC
        "#,
    )
    .bind(user_id)
    .fetch_all(db)
    .await
}

pub async fn create_application(
    db: &PgPool,
    user_id: i64,
    account_id: i64,
    house_type: &str,
    requested_amount_cents: i64,
    interest_rate_bps: i32,
    term_months: i32,
) -> Result<HomeLoanApplication, sqlx::Error> {
    sqlx::query_as::<_, HomeLoanApplication>(
        r#"
        INSERT INTO home_loan_applications (
            user_id, account_id, house_type, requested_amount_cents,
            interest_rate_bps, term_months, status
        )
        VALUES ($1, $2, $3, $4, $5, $6, 'pending_review')
        RETURNING id, user_id, account_id, house_type, requested_amount_cents,
                  interest_rate_bps, term_months, status, staff_remarks,
                  created_at, updated_at, approved_amount_cents, approved_by, approved_at,
                  total_repayment_cents, remaining_cents, monthly_payment_cents, next_due_date
        "#,
    )
    .bind(user_id)
    .bind(account_id)
    .bind(house_type)
    .bind(requested_amount_cents)
    .bind(interest_rate_bps)
    .bind(term_months)
    .fetch_one(db)
    .await
}

pub async fn approve_application(
    db: &PgPool,
    application_id: i64,
    staff_user_id: i64,
    approved_amount_cents: i64,
    total_repayment_cents: i64,
    monthly_payment_cents: i64,
) -> Result<(HomeLoanApplication, BankAccount, Transaction), sqlx::Error> {
    let mut tx = db.begin().await?;

    let application = sqlx::query_as::<_, HomeLoanApplication>(
        r#"
        SELECT id, user_id, account_id, house_type, requested_amount_cents,
               interest_rate_bps, term_months, status, staff_remarks,
               created_at, updated_at, approved_amount_cents, approved_by, approved_at,
               total_repayment_cents, remaining_cents, monthly_payment_cents, next_due_date
        FROM home_loan_applications
        WHERE id = $1 AND status = 'pending_review'
        FOR UPDATE
        "#,
    )
    .bind(application_id)
    .fetch_one(&mut *tx)
    .await?;

    let account = sqlx::query_as::<_, BankAccount>(
        r#"
        SELECT id, user_id, account_number, account_type, balance_cents, status, created_at, updated_at
        FROM bank_accounts
        WHERE id = $1 AND user_id = $2 AND status = 'active'
        FOR UPDATE
        "#,
    )
    .bind(application.account_id)
    .bind(application.user_id)
    .fetch_one(&mut *tx)
    .await?;

    let new_balance = account.balance_cents + approved_amount_cents;

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

    let updated_application = sqlx::query_as::<_, HomeLoanApplication>(
        r#"
        UPDATE home_loan_applications
        SET status = 'approved',
            approved_amount_cents = $1,
            approved_by = $2,
            approved_at = NOW(),
            total_repayment_cents = $3,
            remaining_cents = $3,
            monthly_payment_cents = $4,
            next_due_date = (CURRENT_DATE + make_interval(months => 1))::DATE,
            staff_remarks = 'Approved by staff.',
            updated_at = NOW()
        WHERE id = $5
        RETURNING id, user_id, account_id, house_type, requested_amount_cents,
                  interest_rate_bps, term_months, status, staff_remarks,
                  created_at, updated_at, approved_amount_cents, approved_by, approved_at,
                  total_repayment_cents, remaining_cents, monthly_payment_cents, next_due_date
        "#,
    )
    .bind(approved_amount_cents)
    .bind(staff_user_id)
    .bind(total_repayment_cents)
    .bind(monthly_payment_cents)
    .bind(application_id)
    .fetch_one(&mut *tx)
    .await?;

    let transaction = sqlx::query_as::<_, Transaction>(
        r#"
        INSERT INTO transactions (account_id, user_id, transaction_type, amount_cents, balance_after_cents, description)
        VALUES ($1, $2, 'loan_disbursement', $3, $4, $5)
        RETURNING id, account_id, user_id, transaction_type, amount_cents, balance_after_cents, description, created_at
        "#,
    )
    .bind(account.id)
    .bind(application.user_id)
    .bind(approved_amount_cents)
    .bind(new_balance)
    .bind(format!("Home loan application #{} approved", application_id))
    .fetch_one(&mut *tx)
    .await?;

    tx.commit().await?;

    Ok((updated_application, updated_account, transaction))
}

pub async fn pay_home_loan(
    db: &PgPool,
    user_id: i64,
    application_id: i64,
) -> Result<(HomeLoanApplication, BankAccount, Transaction), sqlx::Error> {
    let mut tx = db.begin().await?;

    let application = sqlx::query_as::<_, HomeLoanApplication>(
        r#"
        SELECT id, user_id, account_id, house_type, requested_amount_cents,
               interest_rate_bps, term_months, status, staff_remarks,
               created_at, updated_at, approved_amount_cents, approved_by, approved_at,
               total_repayment_cents, remaining_cents, monthly_payment_cents, next_due_date
        FROM home_loan_applications
        WHERE id = $1 AND user_id = $2 AND status = 'approved'
        FOR UPDATE
        "#,
    )
    .bind(application_id)
    .bind(user_id)
    .fetch_one(&mut *tx)
    .await?;

    let payment = application
        .monthly_payment_cents
        .unwrap_or(0)
        .min(application.remaining_cents.unwrap_or(0));

    if payment <= 0 {
        return Err(sqlx::Error::RowNotFound);
    }

    let account = sqlx::query_as::<_, BankAccount>(
        r#"
        SELECT id, user_id, account_number, account_type, balance_cents, status, created_at, updated_at
        FROM bank_accounts
        WHERE id = $1 AND user_id = $2 AND status = 'active'
        FOR UPDATE
        "#,
    )
    .bind(application.account_id)
    .bind(user_id)
    .fetch_one(&mut *tx)
    .await?;

    if account.balance_cents < payment {
        return Err(sqlx::Error::RowNotFound);
    }

    let new_balance = account.balance_cents - payment;
    let new_remaining = application.remaining_cents.unwrap_or(0) - payment;
    let final_status = if new_remaining <= 0 { "completed" } else { "approved" };

    let updated_application = sqlx::query_as::<_, HomeLoanApplication>(
        r#"
        UPDATE home_loan_applications
        SET remaining_cents = $1,
            status = $2,
            next_due_date = CASE
                WHEN $2 = 'completed' THEN next_due_date
                ELSE (next_due_date + make_interval(months => 1))::DATE
            END,
            updated_at = NOW()
        WHERE id = $3
        RETURNING id, user_id, account_id, house_type, requested_amount_cents,
                  interest_rate_bps, term_months, status, staff_remarks,
                  created_at, updated_at, approved_amount_cents, approved_by, approved_at,
                  total_repayment_cents, remaining_cents, monthly_payment_cents, next_due_date
        "#,
    )
    .bind(new_remaining)
    .bind(final_status)
    .bind(application_id)
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
    .bind(payment)
    .bind(new_balance)
    .bind(format!("Home loan application #{} repayment", application_id))
    .fetch_one(&mut *tx)
    .await?;

    tx.commit().await?;

    Ok((updated_application, updated_account, transaction))
}