# RustToGold

RustToGold is a server-side rendered banking web application built with Rust, Actix Web, Askama, SQLx and PostgreSQL. The system covers customer onboarding, account opening, dashboard banking, deposits, transfers, PayNow-style payments, cards, fixed deposits, personal loans, home loans, transaction controls, GIRO arrangements, statements, audit logs and admin review workflows.

The project follows a layered MVC structure. Controllers handle requests and sessions, services hold banking business rules, repositories isolate SQL queries, models represent domain records, forms receive validated user input, and Askama templates render the UI.

---

## 1. Quick Start

### Requirements

Install the following before running the project:

- Rust toolchain with Cargo
- PostgreSQL with `psql`
- Git or a ZIP extraction tool
- A browser such as Chrome, Edge, Safari or Firefox
- Optional: Gmail App Password for SMTP email notifications

This project uses PostgreSQL through SQLx. The schema is in:

```text
migrations/001_init.sql
```

Default local server:

```text
http://127.0.0.1:3000
```

Default admin login after running the migration:

```text
Username: admin
Password: Admin@12345
Admin URL: http://127.0.0.1:3000/admin/login
```

---

## 2. Run on Windows

### 2.1 Install tools

Install:

- Rust from rustup
- PostgreSQL for Windows

During PostgreSQL installation, remember the password for the `postgres` database user.

If PowerShell cannot find `psql`, use the full PostgreSQL path. Adjust the version number if your installation folder is different:

```powershell
& "C:\Program Files\PostgreSQL\18\bin\psql.exe" --version
```

### 2.2 Create the database

From the project root, open PowerShell and run:

```powershell
psql -U postgres
```

Inside the PostgreSQL prompt:

```sql
CREATE DATABASE rusttogold;
\q
```

If `psql` is not in PATH:

```powershell
& "C:\Program Files\PostgreSQL\18\bin\psql.exe" -U postgres
```

### 2.3 Create the `.env` file

```powershell
Copy-Item .env.example .env
```

Open `.env` and set your own PostgreSQL password:

```env
DATABASE_URL=postgres://postgres:yourActualPassword@localhost:5432/rusttogold
SESSION_SECRET=0123456789012345678901234567891201234567890123456789012345678912
SERVER_HOST=127.0.0.1
SERVER_PORT=3000
SMTP_HOST=smtp.gmail.com
SMTP_PORT=587
SMTP_USERNAME=your_email@gmail.com
SMTP_PASSWORD=your_16_character_gmail_app_password
SMTP_FROM_EMAIL=your_email@gmail.com
APP_BASE_URL=http://127.0.0.1:3000
ONBOARDING_BASE_URL=http://127.0.0.1:3000
```

### 2.4 Run the database schema

```powershell
psql -U postgres -d rusttogold -f migrations/001_init.sql
```

If `psql` is not in PATH:

```powershell
& "C:\Program Files\PostgreSQL\18\bin\psql.exe" -U postgres -d rusttogold -f migrations/001_init.sql
```

### 2.5 Start the app

```powershell
cargo run
```

Open:

```text
http://127.0.0.1:3000
```

---

## 3. Run on macOS

### 3.1 Install tools

Using Homebrew:

```bash
brew install rustup-init
rustup-init
brew install postgresql@16
brew services start postgresql@16
```

Restart your terminal after installing Rust if `cargo` is not found.

### 3.2 Create the database

For a normal Homebrew PostgreSQL setup:

```bash
createdb rusttogold
```

If your PostgreSQL uses the `postgres` user instead:

```bash
psql -U postgres
```

Inside PostgreSQL:

```sql
CREATE DATABASE rusttogold;
```

### 3.3 Create the `.env` file

```bash
cp .env.example .env
```

For Homebrew PostgreSQL using your macOS username, `.env` can be:

```env
DATABASE_URL=postgres://YOUR_MAC_USERNAME@localhost:5432/rusttogold
SESSION_SECRET=0123456789012345678901234567891201234567890123456789012345678912
SERVER_HOST=127.0.0.1
SERVER_PORT=3000
SMTP_HOST=smtp.gmail.com
SMTP_PORT=587
SMTP_USERNAME=your_email@gmail.com
SMTP_PASSWORD=your_16_character_gmail_app_password
SMTP_FROM_EMAIL=your_email@gmail.com
APP_BASE_URL=http://127.0.0.1:3000
ONBOARDING_BASE_URL=http://127.0.0.1:3000
```

For PostgreSQL using the `postgres` user and password:

```env
DATABASE_URL=postgres://postgres:your_password@localhost:5432/rusttogold
SESSION_SECRET=0123456789012345678901234567891201234567890123456789012345678912
SERVER_HOST=127.0.0.1
SERVER_PORT=3000
SMTP_HOST=smtp.gmail.com
SMTP_PORT=587
SMTP_USERNAME=your_email@gmail.com
SMTP_PASSWORD=your_16_character_gmail_app_password
SMTP_FROM_EMAIL=your_email@gmail.com
APP_BASE_URL=http://127.0.0.1:3000
ONBOARDING_BASE_URL=http://127.0.0.1:3000
```

### 3.4 Run the database schema

```bash
psql -d rusttogold -f migrations/001_init.sql
```

If using the `postgres` user:

```bash
psql -U postgres -d rusttogold -f migrations/001_init.sql
```

### 3.5 Start the app

```powershell
cargo check
cargo run
```

Open the website at:

```text
http://127.0.0.1:3000
```

---

## 4. Reset the database

`001_init.sql` is a reset script. It drops and recreates the local tables.

Windows:

```powershell
psql -U postgres -d rusttogold -f migrations/001_init.sql
```

macOS:

```bash
psql -d rusttogold -f migrations/001_init.sql
```

Use this when you want a clean local database.


---

## 5. Email SMTP Setup

RustToGold sends onboarding and application emails through SMTP when the email variables are present in `.env`. If SMTP is not configured, the app still runs, but email sending is skipped and the server prints a setup message.

For Gmail, use an App Password instead of your normal Gmail password:

1. Open your Google Account security settings.
2. Turn on 2-Step Verification.
3. Open App Passwords.
4. Create an app password for this project, for example `RustToGold Local`.
5. Copy the generated 16-character password into `.env` as `SMTP_PASSWORD`.
6. Restart `cargo run` after changing `.env`.

Example `.env` email block:

```env
SMTP_HOST=smtp.gmail.com
SMTP_PORT=587
SMTP_USERNAME=your_email@gmail.com
SMTP_PASSWORD=your_16_character_gmail_app_password
SMTP_FROM_EMAIL=your_email@gmail.com
APP_BASE_URL=http://127.0.0.1:3000
ONBOARDING_BASE_URL=http://127.0.0.1:3000
```

Notes:

- Do not commit the real `.env` file.
- Keep `.env.example` as placeholders only.
- If an app password is exposed, delete it from Google Account settings and generate a new one.
- `APP_BASE_URL` and `ONBOARDING_BASE_URL` control the base URL used in account-creation email links.


---

## 6. Main Demo Accounts and URLs

Admin:

```text
http://127.0.0.1:3000/admin/login
username: admin
password: Admin@12345
```

Customer accounts are created through onboarding and admin approval:

1. Open `/onboarding/account`.
2. Submit customer onboarding.
3. Login as admin.
4. Approve the onboarding record.
5. Use the generated account-creation link.
6. Create the customer username and password.
7. Login through `/login`.

Optional host-based URLs are also supported by the router:

```text
admin.localhost:3000
onboarding.localhost:3000
```

The normal `/admin/...` and `/onboarding/...` paths work without host-file changes.

---

## 7. Architecture and Folder Structure

```text
RustToGold/
├── Cargo.toml
├── Cargo.lock
├── README.md
├── .env.example
├── migrations/
│   └── 001_init.sql
├── src/
│   ├── main.rs
│   ├── config.rs
│   ├── routes.rs
│   ├── controllers/
│   ├── forms/
│   ├── models/
│   ├── repositories/
│   ├── services/
│   └── views/
├── templates/
│   ├── admin/
│   ├── auth/
│   ├── customer/
│   ├── email/
│   ├── errors/
│   ├── layouts/
│   ├── onboarding/
│   └── partials/
└── static/
    ├── css/
    ├── images/
    └── js/
```

### MVC and layered mapping

| Layer | Location | Responsibility |
|---|---|---|
| Routes | `src/routes.rs` | Maps URLs to controller functions. |
| Controllers | `src/controllers/` | Handles HTTP requests, sessions, redirects and template rendering. |
| Services | `src/services/` | Contains banking rules, validation, workflow decisions and transaction logic. |
| Repositories | `src/repositories/` | Contains SQLx database queries and persistence functions. |
| Models | `src/models/` | Defines domain structs and display/helper methods. |
| Forms | `src/forms/` | Defines request payloads from HTML forms. |
| Views | `src/views/` and `templates/` | Connects Askama template structs to SSR HTML files. |
| Static assets | `static/` | Stores CSS, JavaScript and images. |

Request flow example:

```text
POST /customer/transfer
→ customer_controller::transfer
→ product_service::transfer
→ transaction_control_service::validate_outgoing_transaction
→ product_repository / transaction_repository
→ PostgreSQL transaction
→ Askama-rendered response
```

---

## 7. OOP Concepts Used in the Rust Code

Rust is not class-based, but the project applies OOP-style design through Rust features:

| OOP concept | Rust implementation in this project |
|---|---|
| Encapsulation | Domain data is grouped into structs such as `User`, `Customer`, `CustomerProduct`, `Transaction`, `PersonalLoan`, `FixedDeposit`, `Card`, `GiroArrangement` and `TransactionControls`. |
| Behaviour with data | `impl` blocks provide formatting and state helpers such as display labels, status checks and amount formatting. |
| Abstraction | Controllers depend on service functions instead of direct SQL. Services depend on repositories instead of embedding queries in page handlers. |
| Modular design | Each feature has its own form, model, repository, service and template where needed. |
| Controlled state | Status fields such as active, pending, frozen, rejected, cleared and blocked are enforced through validation and database checks. |
| Reuse | Shared helpers such as `Money`, session guards, renderer functions and template partials avoid repeating logic. |

---

## 8. Core Features

Customer features:

- Public banking pages and onboarding flow
- Login and logout
- Account dashboard
- Savings account applications
- Deposits
- Bank account transfers
- PayNow registration and transfers
- Card applications, freeze and reactivation
- Personal loan application and repayment
- Home loan application and repayment
- Fixed deposit placement and withdrawal
- Transaction history and activity logs
- Bank statement PDF export with date range
- Daily transaction limit management
- Money Lock
- GIRO recurring payment arrangements
- Profile update, PayNow update and password change

Admin features:

- Admin login and logout
- Customer onboarding approval and rejection
- Account product review, activation, freeze and closure
- Staff account management
- Personal loan review
- Home loan review
- Fixed deposit plan management
- High-value transaction monitoring
- Audit log review

Advanced logic:

- Role-based access control
- Password hashing with Argon2
- Cookie-based sessions
- SQL-backed workflows
- Money stored as cents using integers
- Daily transfer limits
- Money Lock validation
- High-value transaction monitoring
- Velocity-style fraud rules
- Own-account transfer handling
- PDF statement generation
- Audit logging for key admin actions

---

## 9. Routes

### Public routes

| Method | Route | Purpose |
|---|---|---|
| GET | `/` | Landing page |
| GET | `/banking` | Banking products page |
| GET | `/security` | Security page |
| GET | `/about` | About page |
| GET | `/faq` | FAQ page |
| GET | `/contact` | Contact page |
| GET | `/login` | Customer login page |
| POST | `/login` | Customer login submission |
| GET | `/logout` | Logout |
| GET | `/403` | Access denied page |

### Onboarding and account creation routes

| Method | Route | Purpose |
|---|---|---|
| GET | `/onboarding` | Redirects to onboarding entry |
| GET | `/onboarding/{path}` | Onboarding step page |
| POST | `/onboarding/{path}` | Onboarding step submission |
| POST | `/api/onboarding/actions/submit-step1` | Submit account step |
| POST | `/api/onboarding/actions/submit-step2` | Submit personal step |
| POST | `/api/onboarding/actions/submit-step3` | Submit contact step |
| POST | `/api/onboarding/actions/submit-step4` | Submit employment step |
| POST | `/api/onboarding/actions/submit` | Submit full onboarding |
| POST | `/api/onboarding/account` | Alternate account step API |
| POST | `/api/onboarding/personal` | Alternate personal step API |
| POST | `/api/onboarding/contact` | Alternate contact step API |
| POST | `/api/onboarding/employment` | Alternate employment step API |
| POST | `/api/onboarding/submit` | Alternate full submit API |
| GET | `/account-creation/init` | Starts account creation from approval link |
| GET | `/account-creation` | Account credential setup page |
| POST | `/account-creation` | Create customer login credentials |
| POST | `/api/account-creation/submit` | Account creation API submission |
| GET | `/signup` | Legacy redirect to onboarding |
| GET | `/signup/{path}` | Legacy signup step redirect |

### Customer routes

| Method | Route | Purpose |
|---|---|---|
| GET | `/customer/dashboard` | Customer dashboard |
| POST | `/customer/accounts/create` | Apply for another savings account product |
| GET | `/customer/deposit` | Deposit page |
| POST | `/customer/deposit` | Deposit money |
| GET | `/customer/transfer` | Bank transfer page |
| POST | `/customer/transfer` | Transfer to bank account |
| GET | `/customer/paynow` | PayNow page |
| POST | `/customer/paynow/register` | Register PayNow mobile number |
| POST | `/customer/paynow/transfer` | Transfer by PayNow number |
| GET | `/customer/giro` | GIRO recurring payment page |
| POST | `/customer/giro` | Create GIRO arrangement |
| POST | `/customer/giro/{id}/cancel` | Cancel GIRO arrangement |
| GET | `/customer/transaction-controls` | Daily limit and Money Lock page |
| POST | `/customer/transaction-controls/limit` | Update daily transaction limit |
| POST | `/customer/transaction-controls/money-lock` | Lock or unlock outgoing transfers |
| GET | `/customer/cards` | Card application and card list page |
| POST | `/customer/cards` | Apply for card |
| POST | `/customer/cards/{id}/freeze` | Freeze card |
| POST | `/customer/cards/{id}/activate` | Reactivate card |
| GET | `/customer/transactions` | Deposit, transfer and PayNow activity log |
| GET | `/customer/statements` | Statement download page |
| GET | `/customer/statements/download` | Download statement PDF |
| GET | `/customer/loan-activity` | Loan activity log |
| GET | `/customer/loan-log` | Alternate loan activity route |
| GET | `/customer/fixed-deposit-activity` | Fixed deposit activity log |
| GET | `/customer/fixed-deposit-log` | Alternate fixed deposit activity route |
| GET | `/customer/loans` | Personal loan dashboard |
| GET | `/customer/loans/apply` | Personal loan application page |
| POST | `/customer/loans/apply` | Submit personal loan application |
| POST | `/customer/loans/{id}/pay` | Repay personal loan |
| GET | `/customer/home-loans` | Home loan dashboard |
| GET | `/customer/home-loans/apply` | Home loan application page |
| POST | `/customer/home-loans/apply` | Submit home loan application |
| POST | `/customer/home-loans/{id}/pay` | Repay home loan |
| GET | `/customer/fixed-deposits` | Fixed deposit dashboard |
| GET | `/customer/fixed-deposits/new` | New fixed deposit page |
| POST | `/customer/fixed-deposits` | Create fixed deposit placement |
| POST | `/customer/fixed-deposits/{id}/withdraw` | Withdraw fixed deposit |
| GET | `/customer/profile` | Profile page |
| POST | `/customer/profile` | Update profile, PayNow or password details |

### Admin routes

| Method | Route | Purpose |
|---|---|---|
| GET | `/admin/login` | Admin login page |
| POST | `/admin/login` | Admin login submission |
| GET | `/admin/logout` | Admin logout |
| GET | `/admin` | Admin dashboard |
| GET | `/admin/dashboard` | Admin dashboard |
| GET | `/admin/signups` | Customer onboarding review |
| GET | `/admin/onboardings` | Customer onboarding review |
| POST | `/admin/onboardings/{id}/approve` | Approve onboarding |
| POST | `/admin/onboardings/{id}/reject` | Reject onboarding |
| POST | `/admin/signups/{id}/approve` | Approve signup route alias |
| POST | `/admin/signups/{id}/reject` | Reject signup route alias |
| GET | `/admin/staff` | Staff management page |
| POST | `/admin/staff` | Create staff/admin user |
| POST | `/admin/staff/{id}/update` | Update staff/admin user |
| POST | `/admin/staff/{id}/delete` | Delete staff/admin user |
| GET | `/admin/accounts` | Customer and product account review |
| POST | `/admin/accounts/users/{id}/suspend` | Suspend customer login |
| POST | `/admin/accounts/users/{id}/activate` | Activate customer login |
| POST | `/admin/accounts/products/{id}/activate` | Activate customer product |
| POST | `/admin/accounts/products/{id}/freeze` | Freeze customer product |
| POST | `/admin/accounts/products/{id}/close` | Close customer product |
| GET | `/admin/high-value-monitoring` | High-value transaction monitoring |
| POST | `/admin/high-value-monitoring/{id}/status` | Clear monitoring record with review notes |
| GET | `/admin/audit-log` | Audit log page |
| GET | `/admin/personal-loans` | Personal loan review page |
| POST | `/admin/personal-loans/{id}/approve` | Approve personal loan |
| POST | `/admin/personal-loans/{id}/reject` | Reject personal loan |
| GET | `/admin/home-loans` | Home loan review page |
| POST | `/admin/home-loans/{id}/approve` | Approve home loan |
| POST | `/admin/home-loans/{id}/reject` | Reject home loan |
| GET | `/admin/fixed-deposits` | Fixed deposit records |
| GET | `/admin/fixed-deposit-plans` | Fixed deposit plan setup page |
| POST | `/admin/fixed-deposit-plans` | Create fixed deposit plan |
| POST | `/admin/fixed-deposit-plans/{id}` | Update fixed deposit plan |

---

## 10. Database Tables

| Table | Purpose |
|---|---|
| `customers` | Stores KYC/customer profile data collected during onboarding. |
| `users` | Stores login credentials, role, status and optional link to a customer. |
| `customer_products` | Stores customer-held products such as everyday savings and high-yield savings accounts, including account number, status and balance. |
| `transactions` | Stores deposits, transfers, PayNow movements, GIRO transactions, loan payments and fixed deposit movements. |
| `personal_loans` | Stores personal loan applications, approvals, repayment amounts and outstanding balance. |
| `home_loan_applications` | Stores home loan applications, property values, down payment checks, approvals and repayments. |
| `fixed_deposit_plans` | Stores admin-created fixed deposit plan definitions such as tenure, interest rate and minimum amount. |
| `fixed_deposits` | Stores customer fixed deposit placements, maturity dates, principal and interest. |
| `cards` | Stores card products linked to active customer accounts and their freeze/active status. |
| `account_creation_links` | Stores one-time account creation links issued after admin approval. |
| `audit_logs` | Stores important admin and system actions for traceability. |
| `registered_paynow` | Stores customer PayNow mobile number registration and linked receiving account. |
| `transaction_controls` | Stores daily transaction limit, limit change cooldown and Money Lock status. |
| `fraud_alerts` | Stores high-value monitoring records, blocked transfer records and review notes. |
| `giro_arrangements` | Stores recurring payment arrangements, frequency, amount, recipient and status. |

Main database design choices:

- UUID primary keys for core entities.
- Money values stored as cents using `BIGINT`.
- Status fields constrained with `CHECK` rules.
- Foreign keys connect customers, users, products, transactions and applications.
- Indexes support account lookup, transaction history, monitoring queues and admin dashboards.

---

## 11. Main Business Workflows

### Customer onboarding

```text
Customer submits onboarding
→ Admin reviews application
→ Admin approves or rejects
→ Approved customer receives account creation link
→ Customer creates login credentials
→ Customer accesses dashboard
```

### Deposit

```text
Customer selects account
→ Enters amount and description
→ Service validates amount/account state
→ Balance increases
→ Transaction record is inserted
```

### Bank transfer

```text
Customer selects source account
→ Enters recipient account and amount
→ Service checks active account, balance, Money Lock, daily limit and high-value rules
→ Database transaction debits sender and credits receiver
→ Transaction records are inserted for both sides
```

### PayNow transfer

```text
Customer registers mobile number
→ Customer enters recipient mobile number
→ Service resolves linked account
→ Transfer validation runs
→ Sender and receiver account balances are updated
```

### Loan review

```text
Customer submits loan application
→ Admin reviews pending loan
→ Admin approves or rejects
→ Approved loan can be repaid by customer
→ Loan transactions are logged
```

### Fixed deposit

```text
Admin maintains plans
→ Customer selects plan and funding account
→ Service validates minimum amount and balance
→ Principal is deducted
→ Fixed deposit record and transaction record are created
```

### High-value monitoring

```text
Customer attempts high-value transfer
→ Service applies high-value and velocity rules
→ High-value records are flagged or blocked
→ Admin reviews monitoring queue
→ Admin clears record with review notes
```

---

## 13. Dependencies

Main Rust crates:

| Crate | Use |
|---|---|
| `actix-web` | HTTP server, routing and request handling. |
| `actix-files` | Static file serving. |
| `actix-session` | Cookie-based sessions. |
| `askama` | Server-side HTML rendering. |
| `sqlx` | PostgreSQL database integration. |
| `uuid` | UUID identifiers. |
| `chrono` | Dates and timestamps. |
| `serde` | Form and request deserialization. |
| `argon2` | Password hashing. |
| `dotenvy` | Environment variable loading. |
| `lettre` | Email sending support. |
| `tokio` | Async runtime. |
| `rand` | Secure token and number generation support. |

---

## 14. Useful Test Flow

Use this flow for a complete walkthrough:

1. Open `/` and review public pages.
2. Submit onboarding through `/onboarding/account`.
3. Login to `/admin/login`.
4. Approve the onboarding application.
5. Open the account creation link and create customer login credentials.
6. Login as customer.
7. Apply for savings account product if needed.
8. Make a deposit.
9. Register PayNow.
10. Perform bank transfer and PayNow transfer.
11. Create card and test freeze/reactivate.
12. Create fixed deposit.
13. Submit personal loan and home loan applications.
14. Approve applications from admin side.
15. Download a bank statement PDF.
16. Change daily transaction limit.
17. Enable and disable Money Lock.
18. Create and cancel a GIRO arrangement.
19. Trigger high-value monitoring and clear the record with notes.
20. Review audit logs.

---

## 15. Submission Notes

Do not include these in the final source ZIP:

```text
target/
.env
*.log
.DS_Store
src.zip
templates.zip
```

Recommended source archive contents:

```text
Cargo.toml
Cargo.lock
README.md
.env.example
migrations/001_init.sql
src/
templates/
static/
```

Required project materials depend on the course submission instructions, but the normal package includes source code, README, database schema, slides, report and recording.
