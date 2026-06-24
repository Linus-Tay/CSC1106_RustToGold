-- Base tables for the RustToGold academic banking simulation.
-- This file does not delete existing data, so it is safe to run again.

CREATE TABLE IF NOT EXISTS users (
    id BIGSERIAL PRIMARY KEY,
    full_name VARCHAR(120) NOT NULL,
    email VARCHAR(255) NOT NULL UNIQUE,
    phone_number VARCHAR(30) NOT NULL,
    date_of_birth DATE NOT NULL,
    password_hash TEXT NOT NULL,
    role VARCHAR(30) NOT NULL DEFAULT 'customer',
    status VARCHAR(30) NOT NULL DEFAULT 'active',
    monthly_income_cents BIGINT NOT NULL DEFAULT 0,
    last_login_at TIMESTAMP NULL,
    created_at TIMESTAMP NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP NOT NULL DEFAULT NOW(),
    CONSTRAINT users_role_check CHECK (role IN ('customer', 'staff', 'admin')),
    CONSTRAINT users_status_check CHECK (status IN ('active', 'suspended', 'closed'))
);

CREATE TABLE IF NOT EXISTS bank_accounts (
    id BIGSERIAL PRIMARY KEY,
    user_id BIGINT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    account_number VARCHAR(50) NOT NULL UNIQUE,
    account_type VARCHAR(30) NOT NULL DEFAULT 'savings',
    balance_cents BIGINT NOT NULL DEFAULT 0,
    status VARCHAR(30) NOT NULL DEFAULT 'active',
    created_at TIMESTAMP NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP NOT NULL DEFAULT NOW(),
    CONSTRAINT bank_accounts_type_check CHECK (account_type IN ('savings', 'current')),
    CONSTRAINT bank_accounts_status_check CHECK (status IN ('active', 'frozen', 'closed')),
    CONSTRAINT bank_accounts_balance_non_negative CHECK (balance_cents >= 0)
);

CREATE TABLE IF NOT EXISTS transactions (
    id BIGSERIAL PRIMARY KEY,
    account_id BIGINT NOT NULL REFERENCES bank_accounts(id) ON DELETE CASCADE,
    user_id BIGINT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    transaction_type VARCHAR(40) NOT NULL,
    amount_cents BIGINT NOT NULL,
    balance_after_cents BIGINT NOT NULL,
    description TEXT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT NOW(),
    CONSTRAINT transactions_amount_positive CHECK (amount_cents > 0),
    CONSTRAINT transactions_balance_after_non_negative CHECK (balance_after_cents >= 0),
      CONSTRAINT transactions_type_check CHECK (transaction_type IN (
        'deposit',
        'withdrawal',
        'transfer_in',
        'transfer_out',
        'fixed_deposit_opening',
        'fixed_deposit_payout',
        'fixed_deposit_early_withdrawal',
        'loan_disbursement',
        'loan_repayment',
        'home_loan_disbursement',
        'home_loan_repayment'
    ))
);

CREATE TABLE IF NOT EXISTS loans (
    id BIGSERIAL PRIMARY KEY,
    user_id BIGINT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    account_id BIGINT NOT NULL REFERENCES bank_accounts(id) ON DELETE CASCADE,
    principal_cents BIGINT NOT NULL,
    interest_rate_bps INTEGER NOT NULL,
    interest_cents BIGINT NOT NULL,
    total_repayment_cents BIGINT NOT NULL,
    remaining_cents BIGINT NOT NULL,
    monthly_payment_cents BIGINT NOT NULL,
    term_months INTEGER NOT NULL,
    next_due_date DATE NOT NULL,
    status VARCHAR(30) NOT NULL DEFAULT 'active',
    created_at TIMESTAMP NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP NOT NULL DEFAULT NOW(),

    CONSTRAINT loans_principal_positive CHECK (principal_cents > 0),
    CONSTRAINT loans_remaining_non_negative CHECK (remaining_cents >= 0),
    CONSTRAINT loans_status_check CHECK (status IN ('active', 'completed'))
);

CREATE TABLE IF NOT EXISTS home_loan_applications (
    id BIGSERIAL PRIMARY KEY,
    user_id BIGINT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    account_id BIGINT NOT NULL REFERENCES bank_accounts(id) ON DELETE CASCADE,

    house_type VARCHAR(50) NOT NULL,
    requested_amount_cents BIGINT NOT NULL,
    interest_rate_bps INTEGER NOT NULL,
    term_months INTEGER NOT NULL,

    status VARCHAR(30) NOT NULL DEFAULT 'pending_review',
    staff_remarks TEXT NULL,

    approved_amount_cents BIGINT NULL,
    approved_by BIGINT NULL REFERENCES users(id) ON DELETE SET NULL,
    approved_at TIMESTAMP NULL,

    total_repayment_cents BIGINT NULL,
    remaining_cents BIGINT NULL,
    monthly_payment_cents BIGINT NULL,
    next_due_date DATE NULL,

    created_at TIMESTAMP NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP NOT NULL DEFAULT NOW(),

    CONSTRAINT home_loan_status_check
        CHECK (status IN ('pending_review', 'approved', 'completed', 'rejected')),

    CONSTRAINT home_loan_house_type_check
        CHECK (house_type IN ('hdb_1_or_2_room', 'hdb_3_or_larger', 'condo', 'landed'))

);


CREATE INDEX IF NOT EXISTS idx_home_loan_applications_user_id_created_at
ON home_loan_applications(user_id, created_at DESC);
CREATE INDEX IF NOT EXISTS idx_home_loan_status
ON home_loan_applications(status);
CREATE INDEX IF NOT EXISTS idx_users_email ON users(email);
CREATE INDEX IF NOT EXISTS idx_bank_accounts_user_id ON bank_accounts(user_id);
CREATE INDEX IF NOT EXISTS idx_transactions_user_id_created_at ON transactions(user_id, created_at DESC);
CREATE INDEX IF NOT EXISTS idx_loans_user_id ON loans(user_id);
CREATE INDEX IF NOT EXISTS idx_loans_account_id ON loans(account_id);