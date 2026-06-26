DROP TABLE IF EXISTS transactions;
--DROP TABLE IF EXISTS bank_accounts;
DROP TABLE IF EXISTS users;
DROP TABLE IF EXISTS customer_products;
DROP TABLE IF EXISTS account_creation_links;
DROP TABLE IF EXISTS customers;

    -- CREATE TYPE gender_type AS ENUM ('MALE', 'FEMALE');
    -- CREATE TYPE residency_type AS ENUM ('CITIZEN', 'PR', 'FOREIGNER');
    -- CREATE TYPE employment_type AS ENUM ('EMPLOYED', 'SELF_EMPLOYED', 'UNEMPLOYED', 'STUDENT', 'RETIRED');
    -- CREATE TYPE contact_method_type AS ENUM ('EMAIL', 'PHONE');
    -- CREATE TYPE kyc_status_type AS ENUM ('PENDING', 'APPROVED', 'REJECTED');
    -- CREATE TYPE statement_type AS ENUM ('DIGITAL', 'PHYSICAL');
    -- CREATE TYPE account_status_type AS ENUM ('ACTIVE', 'INACTIVE', 'PENDING');

CREATE TABLE customers (
    id                   UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    full_name            VARCHAR(100) NOT NULL,
    nric                 VARCHAR(20) UNIQUE NOT NULL,
    date_of_birth        DATE NOT NULL,
    gender               VARCHAR(10) NOT NULL,
    nationality          VARCHAR(50) NOT NULL,
    residency            VARCHAR(50) NOT NULL,
    race                 VARCHAR(50),
    email                VARCHAR(255) UNIQUE NOT NULL,
    phone_number         VARCHAR(20) NOT NULL,
    residential_address  TEXT NOT NULL,
    mailing_address      TEXT,
    preferred_contact    VARCHAR(20) DEFAULT 'email',
    employment_status    VARCHAR(50) NOT NULL,
    occupation           VARCHAR(100),
    employer_name        VARCHAR(100),
    industry             VARCHAR(100),
    monthly_income_range VARCHAR(50),
    kyc_status           VARCHAR(20) DEFAULT 'pending',
    created_at           TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    updated_at           TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
    CONSTRAINT gender_check CHECK (gender IN ('male', 'female')),
    CONSTRAINT residency_check CHECK (residency IN ('citizen', 'pr', 'foreigner')),
    CONSTRAINT employment_check CHECK (employment_status IN ('employed', 'self_employed', 'unemployed', 'student', 'nsf', 'retired')),
    CONSTRAINT kyc_status_check CHECK (kyc_status IN ('pending', 'approved', 'rejected')),
    CONSTRAINT preferred_contact_check CHECK (preferred_contact IN ('email', 'telegram'))
);

CREATE TABLE customer_products (
    id                   UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    customer_id          UUID NOT NULL REFERENCES customers(id),
    product_id           VARCHAR(50) NOT NULL,
    product_type         VARCHAR(20) NOT NULL,
    account_number       VARCHAR(20) UNIQUE NOT NULL,
    status               VARCHAR(20) DEFAULT 'inactive',
	balance_cents		 BIGINT NOT NULL DEFAULT 0,
    created_at           TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    updated_at           TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
    CONSTRAINT product_status_check CHECK (status IN ('active', 'inactive', 'closed')),
    CONSTRAINT          transactions_amount_positive CHECK (balance_cents >= 0),
    CONSTRAINT          product_type_check CHECK (product_type IN ('savings', 'spending', 'fixed_deposit', 'investment'))
);

CREATE TABLE users (
    id                  UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    email               VARCHAR(255) NOT NULL UNIQUE,
    password_hash       TEXT NOT NULL,
    role                VARCHAR(30) NOT NULL DEFAULT 'customer',
    status              VARCHAR(30)  NOT NULL DEFAULT 'active',
    last_login_at       TIMESTAMP NULL,
    created_at          TIMESTAMP NOT NULL DEFAULT NOW(),
    updated_at          TIMESTAMP NOT NULL DEFAULT NOW(),
    CONSTRAINT users_role_check CHECK (role IN ('customer', 'staff', 'admin')),
    CONSTRAINT users_status_check CHECK (status IN ('active', 'suspended', 'closed'))
);

-- CREATE TABLE bank_accounts (
--     id BIGSERIAL PRIMARY KEY,
--     user_id BIGINT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
--     account_number VARCHAR(50) NOT NULL UNIQUE,
--     account_type VARCHAR(30) NOT NULL DEFAULT 'savings',
--     balance_cents BIGINT NOT NULL DEFAULT 0,
--     status VARCHAR(30) NOT NULL DEFAULT 'active',
--     created_at TIMESTAMP NOT NULL DEFAULT NOW(),
--     updated_at TIMESTAMP NOT NULL DEFAULT NOW(),
--     CONSTRAINT bank_accounts_type_check CHECK (account_type IN ('savings', 'current')),
--     CONSTRAINT bank_accounts_status_check CHECK (status IN ('active', 'frozen', 'closed')),
--     CONSTRAINT bank_accounts_balance_non_negative CHECK (balance_cents >= 0)
-- );

CREATE TABLE transactions (
    id                  UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    product_id          UUID NOT NULL REFERENCES customer_products(id) ON DELETE CASCADE,
    customer_id         UUID NOT NULL REFERENCES customers(id) ON DELETE CASCADE,
    transaction_type    VARCHAR(30) NOT NULL,
    amount_cents        BIGINT NOT NULL,
    balance_after_cents BIGINT NOT NULL,
    description         TEXT NULL,
    created_at          TIMESTAMP NOT NULL DEFAULT NOW(),
    CONSTRAINT          transactions_type_check CHECK (transaction_type IN ('deposit', 'withdrawal', 'transfer_in', 'transfer_out')),
    CONSTRAINT          transactions_amount_positive CHECK (amount_cents > 0),
    CONSTRAINT          transactions_balance_after_non_negative CHECK (balance_after_cents >= 0)
);

CREATE TABLE account_creation_links (
    id          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    customer_id UUID NOT NULL REFERENCES customers(id) ON DELETE CASCADE,
    status      VARCHAR(20) DEFAULT 'pending',
    expires_at  TIMESTAMP WITH TIME ZONE NOT NULL,
    created_at  TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
    CONSTRAINT status_check CHECK (status IN ('pending', 'expired', 'used'))
);

CREATE INDEX idx_users_email ON users(email);
--CREATE INDEX idx_bank_accounts_user_id ON bank_accounts(user_id);
CREATE INDEX idx_customer_products_customer_id ON customer_products(customer_id);
CREATE INDEX idx_transactions_customer_id_created_at ON transactions(customer_id, created_at DESC);
