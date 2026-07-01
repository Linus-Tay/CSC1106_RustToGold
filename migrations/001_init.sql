-- RustToGold 001_init.sql
-- Fresh reset script for account applications, admin approval, online banking setup, and product modules.
-- WARNING: This drops and recreates local development tables.

CREATE EXTENSION IF NOT EXISTS pgcrypto;

DROP TABLE IF EXISTS audit_logs CASCADE;
DROP TABLE IF EXISTS cards CASCADE;
DROP TABLE IF EXISTS fixed_deposits CASCADE;
DROP TABLE IF EXISTS fixed_deposit_plans CASCADE;
DROP TABLE IF EXISTS home_loan_applications CASCADE;
DROP TABLE IF EXISTS personal_loans CASCADE;
DROP TABLE IF EXISTS transactions CASCADE;
DROP TABLE IF EXISTS registered_paynow CASCADE;
DROP TABLE IF EXISTS account_creation_links CASCADE;
DROP TABLE IF EXISTS customer_products CASCADE;
DROP TABLE IF EXISTS users CASCADE;
DROP TABLE IF EXISTS customers CASCADE;

CREATE TABLE customers (
    id                   UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    full_name            TEXT NOT NULL,
    nric                 TEXT NOT NULL,
    date_of_birth        DATE NOT NULL,
    gender               TEXT NOT NULL,
    nationality          TEXT NOT NULL,
    residency            TEXT NOT NULL,
    race                 TEXT NULL,
    email                TEXT NOT NULL,
    phone_number         TEXT NOT NULL,
    residential_address  TEXT NOT NULL,
    mailing_address      TEXT NULL,
    preferred_contact    TEXT NULL,
    employment_status    TEXT NOT NULL,
    occupation           TEXT NULL,
    employer_name        TEXT NULL,
    industry             TEXT NULL,
    monthly_income_range TEXT NULL,
    kyc_status           TEXT NOT NULL DEFAULT 'pending',
    created_at           TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at           TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,

    CONSTRAINT customers_kyc_status_check
        CHECK (kyc_status IN ('pending', 'approved', 'rejected'))
);

-- Online banking users are created only after admin approval through an account-creation link.
-- customer_id is kept on staff/admin rows too for code simplicity, but only customer users map to real customers.
CREATE TABLE users (
    id             UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    customer_id    UUID NULL REFERENCES customers(id) ON DELETE CASCADE,
    username       TEXT NOT NULL UNIQUE,
    email          TEXT NOT NULL UNIQUE,
    password_hash  TEXT NOT NULL,
    role           TEXT NOT NULL DEFAULT 'customer',
    status         TEXT NOT NULL DEFAULT 'active',
    last_login_at  TIMESTAMP NULL,
    created_at     TIMESTAMP NOT NULL DEFAULT NOW(),
    updated_at     TIMESTAMP NOT NULL DEFAULT NOW(),

    CONSTRAINT users_customer_id_unique UNIQUE (customer_id),
    CONSTRAINT users_role_check
        CHECK (role IN ('customer', 'staff', 'admin')),
    CONSTRAINT users_status_check
        CHECK (status IN ('active', 'suspended', 'closed')),
    CONSTRAINT users_customer_mapping_check
        CHECK ((role = 'customer' AND customer_id IS NOT NULL) OR (role <> 'customer' AND customer_id IS NULL))
);

-- customer_products is the actual account record used by the app.
-- There is no separate bank_accounts table in this flow.
CREATE TABLE customer_products (
    id             UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    customer_id    UUID NOT NULL REFERENCES customers(id) ON DELETE CASCADE,
    product_id     TEXT NOT NULL,
    product_type   TEXT NOT NULL,
    account_number TEXT NOT NULL UNIQUE,
    status         TEXT NOT NULL DEFAULT 'inactive',
    balance_cents  BIGINT NOT NULL DEFAULT 0,
    created_at     TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at     TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,

    CONSTRAINT customer_products_status_check
        CHECK (status IN ('active', 'inactive', 'frozen', 'closed')),
    CONSTRAINT customer_products_balance_non_negative
        CHECK (balance_cents >= 0),
    CONSTRAINT customer_products_type_check
        CHECK (product_type IN ('savings', 'spending', 'fixed_deposit', 'investment'))
);

CREATE TABLE transactions (
    id                  UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    product_id          UUID NOT NULL REFERENCES customer_products(id) ON DELETE CASCADE,
    transaction_type    TEXT NOT NULL,
    amount_cents        BIGINT NOT NULL,
    balance_after_cents BIGINT NOT NULL,
    description         TEXT NULL,
    created_at          TIMESTAMP NOT NULL DEFAULT NOW(),

    CONSTRAINT transactions_type_check
        CHECK (transaction_type IN (
            'deposit',
            'withdrawal',
            'transfer_in',
            'transfer_out',
            'paynow_transfer_in',
            'paynow_transfer_out',
            'loan_disbursement',
            'loan_payment',
            'home_loan_payment',
            'fixed_deposit_open',
            'fixed_deposit_withdrawal',
            'fixed_deposit_payout'
        )),
    CONSTRAINT transactions_amount_positive
        CHECK (amount_cents > 0),
    CONSTRAINT transactions_balance_after_non_negative
        CHECK (balance_after_cents >= 0)
);

CREATE TABLE personal_loans (
    id                    UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    customer_id           UUID NOT NULL REFERENCES customers(id) ON DELETE CASCADE,
    funding_product_id    UUID NOT NULL REFERENCES customer_products(id) ON DELETE RESTRICT,
    purpose               TEXT NOT NULL,
    principal_cents       BIGINT NOT NULL CHECK (principal_cents > 0),
    annual_rate_bps       INTEGER NOT NULL CHECK (annual_rate_bps > 0),
    term_months           INTEGER NOT NULL CHECK (term_months BETWEEN 1 AND 120),
    monthly_payment_cents BIGINT NOT NULL CHECK (monthly_payment_cents > 0),
    outstanding_cents     BIGINT NOT NULL DEFAULT 0 CHECK (outstanding_cents >= 0),
    status                TEXT NOT NULL DEFAULT 'pending'
        CHECK (status IN ('pending', 'active', 'rejected', 'fully_paid', 'cancelled')),
    reviewed_by           UUID REFERENCES users(id) ON DELETE SET NULL,
    reviewed_at           TIMESTAMPTZ,
    created_at            TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at            TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE home_loan_applications (
    id                    UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    customer_id           UUID NOT NULL REFERENCES customers(id) ON DELETE CASCADE,
    account_product_id    UUID REFERENCES customer_products(id) ON DELETE SET NULL,
    property_type         TEXT NOT NULL,
    property_value_cents  BIGINT NOT NULL CHECK (property_value_cents > 0),
    down_payment_cents    BIGINT NOT NULL CHECK (down_payment_cents >= 0),
    loan_amount_cents     BIGINT NOT NULL CHECK (loan_amount_cents > 0),
    annual_rate_bps       INTEGER NOT NULL CHECK (annual_rate_bps > 0),
    term_years            INTEGER NOT NULL CHECK (term_years BETWEEN 1 AND 40),
    monthly_payment_cents BIGINT NOT NULL CHECK (monthly_payment_cents > 0),
    outstanding_cents     BIGINT NOT NULL DEFAULT 0 CHECK (outstanding_cents >= 0),
    status                TEXT NOT NULL DEFAULT 'pending'
        CHECK (status IN ('pending', 'approved', 'rejected', 'fully_paid')),
    reviewed_by           UUID REFERENCES users(id) ON DELETE SET NULL,
    reviewed_at           TIMESTAMPTZ,
    created_at            TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at            TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CHECK (down_payment_cents < property_value_cents)
);

CREATE TABLE fixed_deposit_plans (
    id                   BIGSERIAL PRIMARY KEY,
    plan_name            TEXT NOT NULL,
    tenure_months        INTEGER NOT NULL CHECK (tenure_months BETWEEN 1 AND 60),
    annual_rate_bps      INTEGER NOT NULL CHECK (annual_rate_bps BETWEEN 1 AND 1000),
    minimum_amount_cents BIGINT NOT NULL CHECK (minimum_amount_cents > 0),
    is_active            BOOLEAN NOT NULL DEFAULT TRUE,
    created_at           TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at           TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE fixed_deposits (
    id                 UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    customer_id        UUID NOT NULL REFERENCES customers(id) ON DELETE CASCADE,
    funding_product_id UUID NOT NULL REFERENCES customer_products(id) ON DELETE RESTRICT,
    plan_id            BIGINT NOT NULL REFERENCES fixed_deposit_plans(id) ON DELETE RESTRICT,
    principal_cents    BIGINT NOT NULL CHECK (principal_cents > 0),
    annual_rate_bps    INTEGER NOT NULL CHECK (annual_rate_bps > 0),
    tenure_months      INTEGER NOT NULL CHECK (tenure_months BETWEEN 1 AND 60),
    interest_cents     BIGINT NOT NULL CHECK (interest_cents >= 0),
    maturity_date      DATE NOT NULL,
    status             TEXT NOT NULL DEFAULT 'active'
        CHECK (status IN ('active', 'matured', 'withdrawn', 'paid_out')),
    created_at         TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at         TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE cards (
    id                 UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    customer_id        UUID NOT NULL REFERENCES customers(id) ON DELETE CASCADE,
    linked_product_id  UUID NOT NULL REFERENCES customer_products(id) ON DELETE RESTRICT,
    card_type          TEXT NOT NULL CHECK (card_type IN ('debit', 'student')),
    display_name       TEXT NOT NULL,
    masked_number      TEXT NOT NULL,
    status             TEXT NOT NULL DEFAULT 'active' CHECK (status IN ('active', 'frozen', 'cancelled')),
    created_at         TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at         TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE account_creation_links (
    id          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    customer_id UUID NOT NULL REFERENCES customers(id) ON DELETE CASCADE,
    status      TEXT NOT NULL DEFAULT 'pending',
    expires_at  TIMESTAMP WITH TIME ZONE NOT NULL,
    created_at  TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,

    CONSTRAINT account_creation_links_status_check
        CHECK (status IN ('pending', 'expired', 'used'))
);


CREATE TABLE audit_logs (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    actor_user_id   UUID NULL REFERENCES users(id) ON DELETE SET NULL,
    action          TEXT NOT NULL,
    entity_type     TEXT NOT NULL,
    entity_id       TEXT NULL,
    details         TEXT NULL,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE registered_paynow (
    id                UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    customer_id       UUID NOT NULL REFERENCES customers(id) ON DELETE CASCADE,
    paynow_type       TEXT NOT NULL,
    paynow_id         TEXT NOT NULL,
    linked_account_id UUID NOT NULL REFERENCES customer_products(id) ON DELETE CASCADE,
    registered_at     TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
    status            TEXT NOT NULL DEFAULT 'active',

    CONSTRAINT registered_paynow_type_check
        CHECK (paynow_type IN ('phone_number', 'nric')),
    CONSTRAINT registered_paynow_status_check
        CHECK (status IN ('active', 'inactive'))
);

CREATE UNIQUE INDEX idx_customers_active_nric_unique
    ON customers (lower(nric))
    WHERE kyc_status <> 'rejected';

CREATE UNIQUE INDEX idx_customers_active_email_unique
    ON customers (lower(email))
    WHERE kyc_status <> 'rejected';

CREATE INDEX idx_users_username ON users(username);
CREATE INDEX idx_users_email ON users(email);
CREATE INDEX idx_users_customer_id ON users(customer_id);
CREATE INDEX idx_customer_products_customer_id ON customer_products(customer_id);
CREATE INDEX idx_customer_products_account_number ON customer_products(account_number);
CREATE INDEX idx_transactions_product_id_created_at ON transactions(product_id, created_at DESC);
CREATE INDEX idx_account_creation_links_customer_status ON account_creation_links(customer_id, status);
CREATE INDEX idx_personal_loans_customer_id ON personal_loans(customer_id);
CREATE INDEX idx_personal_loans_status ON personal_loans(status);
CREATE INDEX idx_home_loan_applications_customer_id ON home_loan_applications(customer_id);
CREATE INDEX idx_home_loan_applications_status ON home_loan_applications(status);
CREATE INDEX idx_fixed_deposits_customer_id ON fixed_deposits(customer_id);
CREATE INDEX idx_fixed_deposits_status_maturity ON fixed_deposits(status, maturity_date);
CREATE INDEX idx_cards_customer_id ON cards(customer_id);
CREATE INDEX idx_cards_linked_product_id ON cards(linked_product_id);
CREATE INDEX idx_registered_paynow_customer_id ON registered_paynow(customer_id);
CREATE INDEX idx_registered_paynow_linked_account_id ON registered_paynow(linked_account_id);
CREATE UNIQUE INDEX idx_registered_paynow_active_identifier
    ON registered_paynow (paynow_type, lower(paynow_id))
    WHERE status = 'active';
CREATE INDEX idx_audit_logs_created_at ON audit_logs(created_at DESC);
CREATE INDEX idx_audit_logs_actor_created_at ON audit_logs(actor_user_id, created_at DESC);

INSERT INTO fixed_deposit_plans (plan_name, tenure_months, annual_rate_bps, minimum_amount_cents, is_active)
VALUES
    ('Starter 6 Month Deposit', 6, 250, 100000, TRUE),
    ('Growth 12 Month Deposit', 12, 325, 500000, TRUE),
    ('Premier 24 Month Deposit', 24, 380, 1000000, TRUE);

-- Seeded admin user for demo/testing.
-- Username: admin
-- Email: admin@rusttogold.test
-- Password: Admin@12345
INSERT INTO users (username, email, password_hash, role, status)
VALUES (
    'admin',
    'admin@rusttogold.test',
    '$argon2id$v=19$m=65536,t=3,p=4$iigvsB3QLIB4HeWPwpF6jQ$i8tsrNgQaQlvXKb41xt0+kMWA6j+0FFzxJi0BOADhNQ',
    'admin',
    'active'
);
