use crate::models::{AdminHomeLoanRecord, BankAccount, HomeLoanApplication, HomeLoanSummary, Transaction};
use sqlx::{PgPool, Postgres, Transaction as DbTransaction};

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

pub async fn list_pending_for_admin(
    db: &PgPool,
) -> Result<Vec<HomeLoanApplication>, sqlx::Error> {
    sqlx::query_as::<_, HomeLoanApplication>(
        r#"
        SELECT id, user_id, account_id, house_type, requested_amount_cents,
               interest_rate_bps, term_months, status, staff_remarks,
               created_at, updated_at, approved_amount_cents, approved_by, approved_at,
               total_repayment_cents, remaining_cents, monthly_payment_cents, next_due_date
        FROM home_loan_applications
        WHERE status = 'pending_review'
        ORDER BY created_at ASC, id ASC
        "#,
    )
    .fetch_all(db)
    .await
}

pub async fn list_all_for_admin(
    db: &PgPool,
) -> Result<Vec<AdminHomeLoanRecord>, sqlx::Error> {
    sqlx::query_as::<_, AdminHomeLoanRecord>(
        r#"
        SELECT
            h.id,
            h.user_id,
            h.account_id,
            h.house_type,
            h.requested_amount_cents,
            h.interest_rate_bps,
            h.term_months,
            h.status,
            h.staff_remarks,
            h.approved_amount_cents,
            h.approved_by,
            h.approved_at,
            h.total_repayment_cents,
            h.remaining_cents,
            h.monthly_payment_cents,
            h.next_due_date,

            u.full_name AS customer_name,
            u.email AS customer_email,
            b.account_number

        FROM home_loan_applications h
        JOIN users u ON u.id = h.user_id
        JOIN bank_accounts b ON b.id = h.account_id

        ORDER BY h.created_at DESC, h.id DESC
        "#,
    )
    .fetch_all(db)
    .await
}

pub async fn find_by_id(
    db: &PgPool,
    application_id: i64,
) -> Result<Option<HomeLoanApplication>, sqlx::Error> {
    sqlx::query_as::<_, HomeLoanApplication>(
        r#"
        SELECT id, user_id, account_id, house_type, requested_amount_cents,
               interest_rate_bps, term_months, status, staff_remarks,
               created_at, updated_at, approved_amount_cents, approved_by, approved_at,
               total_repayment_cents, remaining_cents, monthly_payment_cents, next_due_date
        FROM home_loan_applications
        WHERE id = $1
        "#,
    )
    .bind(application_id)
    .fetch_optional(db)
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

pub async fn summary_by_user_id(
    db: &PgPool,
    user_id: i64,
) -> Result<HomeLoanSummary, sqlx::Error> {
    sqlx::query_as::<_, HomeLoanSummary>(
        r#"
        SELECT
            COUNT(*)::BIGINT AS total_count,

            COUNT(*) FILTER (
                WHERE status = 'pending_review'
            )::BIGINT AS pending_count,

            COUNT(*) FILTER (
                WHERE status = 'approved'
            )::BIGINT AS approved_count,

            COUNT(*) FILTER (
                WHERE status = 'completed'
            )::BIGINT AS completed_count,

            COUNT(*) FILTER (
                WHERE status = 'rejected'
            )::BIGINT AS rejected_count,

            COALESCE(SUM(approved_amount_cents),0)::BIGINT
                AS total_approved_cents,

            COALESCE(SUM(remaining_cents),0)::BIGINT
                AS total_remaining_cents,

            COALESCE(SUM(monthly_payment_cents),0)::BIGINT
                AS total_monthly_payment_cents

        FROM home_loan_applications
        WHERE user_id = $1
        "#,
    )
    .bind(user_id)
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
    let mut transaction = db.begin().await?;

    let application = lock_pending_application(&mut transaction, application_id).await?;
    let account = lock_account_by_id(
        &mut transaction,
        application.account_id,
        application.user_id,
    )
    .await?;

    let new_balance = account.balance_cents + approved_amount_cents;

    let updated_account = sqlx::query_as::<_, BankAccount>(
        r#"
        UPDATE bank_accounts
        SET balance_cents = $1, updated_at = NOW()
        WHERE id = $2
        RETURNING id, user_id, account_number, account_type, balance_cents,
                  status, created_at, updated_at
        "#,
    )
    .bind(new_balance)
    .bind(account.id)
    .fetch_one(&mut *transaction)
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
    .fetch_one(&mut *transaction)
    .await?;

    let transaction_record = sqlx::query_as::<_, Transaction>(
        r#"
        INSERT INTO transactions (
            account_id, user_id, transaction_type, amount_cents,
            balance_after_cents, description
        )
        VALUES ($1, $2, 'home_loan_disbursement', $3, $4, $5)
        RETURNING id, account_id, user_id, transaction_type,
                  amount_cents, balance_after_cents, description, created_at
        "#,
    )
    .bind(account.id)
    .bind(application.user_id)
    .bind(approved_amount_cents)
    .bind(new_balance)
    .bind(format!("Home loan application #{} approved", application_id))
    .fetch_one(&mut *transaction)
    .await?;

    transaction.commit().await?;
    Ok((updated_application, updated_account, transaction_record))
}

pub async fn reject_application(
    db: &PgPool,
    application_id: i64,
    staff_remarks: &str,
) -> Result<HomeLoanApplication, sqlx::Error> {
    sqlx::query_as::<_, HomeLoanApplication>(
        r#"
        UPDATE home_loan_applications
        SET status = 'rejected',
            staff_remarks = $1,
            updated_at = NOW()
        WHERE id = $2 AND status = 'pending_review'
        RETURNING id, user_id, account_id, house_type, requested_amount_cents,
                  interest_rate_bps, term_months, status, staff_remarks,
                  created_at, updated_at, approved_amount_cents, approved_by, approved_at,
                  total_repayment_cents, remaining_cents, monthly_payment_cents, next_due_date
        "#,
    )
    .bind(staff_remarks)
    .bind(application_id)
    .fetch_one(db)
    .await
}

pub async fn pay_home_loan(
    db: &PgPool,
    user_id: i64,
    application_id: i64,
    payment_cents: Option<i64>,
) -> Result<HomeLoanApplication, sqlx::Error> {
    let mut transaction = db.begin().await?;

    let application = lock_approved_application(&mut transaction, user_id, application_id).await?;
    let account = lock_account_by_id(&mut transaction, application.account_id, user_id).await?;

    let monthly_payment = application.monthly_payment_cents.unwrap_or(0);
    let remaining = application.remaining_cents.unwrap_or(0);

    let amount_to_pay = payment_cents
        .unwrap_or(monthly_payment)
        .min(remaining);

    if amount_to_pay <= 0 {
        return Err(sqlx::Error::RowNotFound);
    }

    if account.balance_cents < amount_to_pay {
        return Err(sqlx::Error::RowNotFound);
    }

    let new_balance = account.balance_cents - amount_to_pay;
    let new_remaining = remaining - amount_to_pay;

    let final_status = if new_remaining <= 0 {
        "completed"
    } else {
        "approved"
    };

    let months_covered = if monthly_payment > 0 && amount_to_pay >= monthly_payment {
        (amount_to_pay / monthly_payment) as i32
    } else {
        0
    };

    let updated_application = sqlx::query_as::<_, HomeLoanApplication>(
        r#"
        UPDATE home_loan_applications
        SET remaining_cents = $1,
            status = $2,
            next_due_date = CASE
                WHEN $2 = 'completed' THEN next_due_date
                ELSE (next_due_date + make_interval(months => $3))::DATE
            END,
            updated_at = NOW()
        WHERE id = $4
        RETURNING id, user_id, account_id, house_type, requested_amount_cents,
                  interest_rate_bps, term_months, status, staff_remarks,
                  created_at, updated_at, approved_amount_cents, approved_by, approved_at,
                  total_repayment_cents, remaining_cents, monthly_payment_cents, next_due_date
        "#,
    )
    .bind(new_remaining)
    .bind(final_status)
    .bind(months_covered)
    .bind(application_id)
    .fetch_one(&mut *transaction)
    .await?;

    sqlx::query(
        r#"
        UPDATE bank_accounts
        SET balance_cents = $1, updated_at = NOW()
        WHERE id = $2
        "#,
    )
    .bind(new_balance)
    .bind(account.id)
    .execute(&mut *transaction)
    .await?;

    sqlx::query(
        r#"
        INSERT INTO transactions (
            account_id, user_id, transaction_type, amount_cents,
            balance_after_cents, description
        )
        VALUES ($1, $2, 'home_loan_repayment', $3, $4, $5)
        "#,
    )
    .bind(account.id)
    .bind(user_id)
    .bind(amount_to_pay)
    .bind(new_balance)
    .bind(format!("Home loan application #{} repayment", application_id))
    .execute(&mut *transaction)
    .await?;

    transaction.commit().await?;
    Ok(updated_application)
}

async fn lock_account_by_id(
    transaction: &mut DbTransaction<'_, Postgres>,
    account_id: i64,
    user_id: i64,
) -> Result<BankAccount, sqlx::Error> {
    sqlx::query_as::<_, BankAccount>(
        r#"
        SELECT id, user_id, account_number, account_type, balance_cents,
               status, created_at, updated_at
        FROM bank_accounts
        WHERE id = $1 AND user_id = $2 AND status = 'active'
        FOR UPDATE
        "#,
    )
    .bind(account_id)
    .bind(user_id)
    .fetch_one(&mut **transaction)
    .await
}

async fn lock_pending_application(
    transaction: &mut DbTransaction<'_, Postgres>,
    application_id: i64,
) -> Result<HomeLoanApplication, sqlx::Error> {
    sqlx::query_as::<_, HomeLoanApplication>(
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
    .fetch_one(&mut **transaction)
    .await
}

async fn lock_approved_application(
    transaction: &mut DbTransaction<'_, Postgres>,
    user_id: i64,
    application_id: i64,
) -> Result<HomeLoanApplication, sqlx::Error> {
    sqlx::query_as::<_, HomeLoanApplication>(
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
    .fetch_one(&mut **transaction)
    .await
}
