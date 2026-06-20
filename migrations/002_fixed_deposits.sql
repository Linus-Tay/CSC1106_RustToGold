-- Fixed Deposit tables and starter plans.
-- Run this after 001_init.sql. It only adds FD data and does not drop existing tables.

CREATE TABLE IF NOT EXISTS fixed_deposit_plans (
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

CREATE TABLE IF NOT EXISTS fixed_deposits (
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

CREATE INDEX IF NOT EXISTS idx_fixed_deposit_plans_status ON fixed_deposit_plans(status);
CREATE INDEX IF NOT EXISTS idx_fixed_deposits_user_id_created_at ON fixed_deposits(user_id, created_at DESC);
CREATE INDEX IF NOT EXISTS idx_fixed_deposits_status_maturity ON fixed_deposits(status, maturity_date);

INSERT INTO fixed_deposit_plans (name, duration_months, interest_rate_bps, minimum_amount_cents, status)
VALUES
    ('3 Month Starter FD', 3, 180, 50000, 'active'),
    ('6 Month Growth FD', 6, 250, 100000, 'active'),
    ('12 Month Premium FD', 12, 320, 200000, 'active'),
    ('24 Month Wealth FD', 24, 380, 500000, 'active')
ON CONFLICT (name) DO NOTHING;
