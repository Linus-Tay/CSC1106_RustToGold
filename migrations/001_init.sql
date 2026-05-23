DROP TABLE IF EXISTS fixed_deposits;
DROP TABLE IF EXISTS fixed_deposit_plans;
DROP TABLE IF EXISTS transactions;
DROP TABLE IF EXISTS bank_accounts;
DROP TABLE IF EXISTS users;

CREATE TABLE users (
    id BIGSERIAL PRIMARY KEY,
    full_name VARCHAR(120) NOT NULL,
    email VARCHAR(255) NOT NULL UNIQUE,
    phone_number VARCHAR(30) NOT NULL,
    date_of_birth DATE NOT NULL,
    password_hash TEXT NOT NULL,
    role VARCHAR(30) NOT NULL DEFAULT 'customer',
    status VARCHAR(30) NOT NULL DEFAULT 'active',
    last_login_at TIMESTAMP NULL,
    created_at TIMESTAMP NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP NOT NULL DEFAULT NOW(),
    CONSTRAINT users_role_check CHECK (role IN ('customer', 'staff', 'admin')),
    CONSTRAINT users_status_check CHECK (status IN ('active', 'suspended', 'closed'))
);

CREATE TABLE bank_accounts (
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

CREATE TABLE fixed_deposit_plans (
    id BIGSERIAL PRIMARY KEY,
    name VARCHAR(120) NOT NULL UNIQUE,
    duration_months INTEGER NOT NULL,
    interest_rate_bps INTEGER NOT NULL,
    minimum_amount_cents BIGINT NOT NULL,
    status VARCHAR(30) NOT NULL DEFAULT 'active',
    created_at TIMESTAMP NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP NOT NULL DEFAULT NOW(),
    CONSTRAINT fd_plans_duration_check CHECK (duration_months BETWEEN 1 AND 60),
    CONSTRAINT fd_plans_interest_check CHECK (interest_rate_bps BETWEEN 1 AND 2000),
    CONSTRAINT fd_plans_minimum_check CHECK (minimum_amount_cents > 0),
    CONSTRAINT fd_plans_status_check CHECK (status IN ('active', 'inactive'))
);

CREATE TABLE fixed_deposits (
    id BIGSERIAL PRIMARY KEY,
    user_id BIGINT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    account_id BIGINT NOT NULL REFERENCES bank_accounts(id) ON DELETE CASCADE,
    plan_id BIGINT NOT NULL REFERENCES fixed_deposit_plans(id),
    principal_cents BIGINT NOT NULL,
    interest_rate_bps INTEGER NOT NULL,
    interest_cents BIGINT NOT NULL DEFAULT 0,
    penalty_cents BIGINT NOT NULL DEFAULT 0,
    payout_cents BIGINT NOT NULL DEFAULT 0,
    start_date DATE NOT NULL DEFAULT CURRENT_DATE,
    maturity_date DATE NOT NULL,
    status VARCHAR(30) NOT NULL DEFAULT 'active',
    created_at TIMESTAMP NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP NOT NULL DEFAULT NOW(),
    CONSTRAINT fixed_deposits_principal_positive CHECK (principal_cents > 0),
    CONSTRAINT fixed_deposits_interest_non_negative CHECK (interest_cents >= 0),
    CONSTRAINT fixed_deposits_penalty_non_negative CHECK (penalty_cents >= 0),
    CONSTRAINT fixed_deposits_payout_non_negative CHECK (payout_cents >= 0),
    CONSTRAINT fixed_deposits_status_check CHECK (status IN ('active', 'matured', 'withdrawn', 'paid_out', 'cancelled'))
);

CREATE TABLE transactions (
    id BIGSERIAL PRIMARY KEY,
    account_id BIGINT NOT NULL REFERENCES bank_accounts(id) ON DELETE CASCADE,
    user_id BIGINT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    transaction_type VARCHAR(40) NOT NULL,
    amount_cents BIGINT NOT NULL,
    balance_after_cents BIGINT NOT NULL,
    description TEXT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT NOW(),
    CONSTRAINT transactions_type_check CHECK (transaction_type IN ('deposit', 'withdrawal', 'transfer_in', 'transfer_out', 'fixed_deposit_opening', 'fixed_deposit_payout', 'fixed_deposit_early_withdrawal')),
    CONSTRAINT transactions_amount_positive CHECK (amount_cents > 0),
    CONSTRAINT transactions_balance_after_non_negative CHECK (balance_after_cents >= 0)
);

CREATE INDEX idx_users_email ON users(email);
CREATE INDEX idx_bank_accounts_user_id ON bank_accounts(user_id);
CREATE INDEX idx_transactions_user_id_created_at ON transactions(user_id, created_at DESC);
CREATE INDEX idx_fixed_deposits_user_id_created_at ON fixed_deposits(user_id, created_at DESC);
CREATE INDEX idx_fixed_deposits_status_maturity ON fixed_deposits(status, maturity_date);
CREATE INDEX idx_fixed_deposit_plans_status ON fixed_deposit_plans(status);

INSERT INTO fixed_deposit_plans (name, duration_months, interest_rate_bps, minimum_amount_cents, status)
VALUES
    ('3 Month Starter FD', 3, 180, 50000, 'active'),
    ('6 Month Growth FD', 6, 250, 100000, 'active'),
    ('12 Month Premium FD', 12, 320, 200000, 'active'),
    ('24 Month Wealth FD', 24, 380, 500000, 'active');
