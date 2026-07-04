// View layer: Askama template structs and rendering helpers.

use crate::models::{
    AdminAuditLogRecord, AdminCustomerAccountRecord, AdminCustomerApplication, AdminDashboardSummary, AdminHomeLoanRecord, AdminPersonalLoanRecord, AdminStaffUser, Card, FixedDeposit, FixedDepositAdminRecord, FixedDepositPlan, FixedDepositSummary, FraudAlert, GiroArrangement, HighValueAlertRecord, HomeLoanApplication, HomeLoanSummary, PayNowRegistration, PersonalLoan, Product, StatementTransaction, Transaction, TransactionControl,
};
use askama::Template;
use uuid::Uuid;

#[derive(Template)]
#[template(path = "index.html")]
pub struct HomeTemplate;

#[derive(Template)]
#[template(path = "about.html")]
pub struct AboutTemplate;

#[derive(Template)]
#[template(path = "banking.html")]
pub struct BankingTemplate;

#[derive(Template)]
#[template(path = "contact.html")]
pub struct ContactTemplate;

#[derive(Template)]
#[template(path = "faq.html")]
pub struct FaqTemplate;

#[derive(Template)]
#[template(path = "security.html")]
pub struct SecurityTemplate;

#[derive(Template)]
#[template(path = "onboarding/onboarding.html")]
pub struct OnboardingTemplate {
    pub product_available: bool,
    pub product_id: String,
    pub product_type: String,
    pub product_name: String,
    pub product_summary: String,
    pub product_rate: String,
    pub product_minimum: String,
    pub product_features: Vec<String>,
}

#[derive(Template)]
#[template(path = "onboarding/onboarding_result.html")]
pub struct OnboardingResultTemplate {
    pub reference_no: String,
    pub created_at: String,
}

#[derive(Template)]
#[template(path = "onboarding/forms/onboarding_account.html")]
pub struct OnboardingAccountTemplate {
    pub error: String,
    pub has_error: bool,
    pub selected_account_type: String,
    pub preferred_account_name: String,
    pub account_purpose: String,
}

#[derive(Template)]
#[template(path = "onboarding/forms/onboarding_personal.html")]
pub struct OnboardingPersonalTemplate {
    pub error: String,
    pub has_error: bool,
    pub full_name: String,
    pub nric: String,
    pub gender: String,
    pub race: String,
    pub dob: String,
    pub nationality: String,
    pub residential_status: String,
    pub residential_address: String,
    pub step1_completed: bool,
    pub identity_confirmed: bool,
}

#[derive(Template)]
#[template(path = "onboarding/forms/onboarding_contact.html")]
pub struct OnboardingContactTemplate {
    pub error: String,
    pub has_error: bool,
    pub email: String,
    pub phone_number: String,
    pub mailing_address: String,
    pub step1_completed: bool,
    pub step2_completed: bool,
}

#[derive(Template)]
#[template(path = "onboarding/forms/onboarding_employment.html")]
pub struct OnboardingEmploymentTemplate {
    pub error: String,
    pub has_error: bool,
    pub employment_status: String,
    pub occupation: String,
    pub employer_name: String,
    pub monthly_income_range: String,
    pub source_initial_deposit: String,
    pub step1_completed: bool,
    pub step2_completed: bool,
    pub step3_completed: bool,
}

#[derive(Template)]
#[template(path = "onboarding/forms/onboarding_review.html")]
pub struct OnboardingReviewTemplate {
    pub error: String,
    pub has_error: bool,
    pub selected_account_type: String,
    pub preferred_account_name: String,
    pub account_purpose: String,
    pub full_name: String,
    pub nric_fin: String,
    pub date_of_birth: String,
    pub nationality: String,
    pub residential_status: String,
    pub residential_address: String,
    pub email: String,
    pub phone_number: String,
    pub mailing_address: String,
    pub employment_status: String,
    pub occupation: String,
    pub employer_name: String,
    pub monthly_income_range: String,
    pub source_initial_deposit: String,
    pub step1_completed: bool,
    pub step2_completed: bool,
    pub step3_completed: bool,
    pub step4_completed: bool,
}

#[derive(Template)]
#[template(path = "onboarding/account_creation.html")]
pub struct AccountCreationSetupTemplate {
    pub email: String,
    pub error: String,
    pub has_error: bool,
}

#[derive(Template)]
#[template(path = "onboarding/account_creation_success.html")]
pub struct AccountCreationSuccessTemplate {
    pub username: String,
    pub email: String,
}

#[derive(Template)]
#[template(path = "email/account_creation_email.html")]
pub struct AccountCreationEmailTemplate {
    pub account_creation_link: String,
}

#[derive(Template)]
#[template(path = "email/account_2fa_email.html")]
pub struct Account2FAEmailTemplate {
    pub verification_code: String,
}


#[derive(Template)]
#[template(path = "email/application_received_email.html")]
pub struct ApplicationReceivedEmailTemplate;


#[derive(Template)]
#[template(path = "admin/login.html")]
pub struct AdminLoginTemplate {
    pub error: String,
    pub has_error: bool,
}

#[derive(Template)]
#[template(path = "auth/login.html")]
pub struct LoginTemplate {
    pub error: String,
    pub has_error: bool,
}

#[derive(Template)]
#[template(path = "auth/2fa.html")]
pub struct TwoFactorAuthTemplate {
    pub error: String,
    pub has_error: bool,
}

#[derive(Template)]
#[template(path = "customer/dashboard.html")]
pub struct DashboardTemplate {
    pub full_name: String,
    pub accounts: Vec<Product>,
    pub has_accounts: bool,
    pub can_apply_everyday_savings: bool,
    pub can_apply_high_yield_savings: bool,
    pub account_application_notice: String,
    pub has_account_application_notice: bool,
    pub daily_limit_display: String,
    pub outgoing_today_display: String,
    pub remaining_today_display: String,
    pub create_account_error: String,
    pub has_create_account_error: bool,
}

#[derive(Template)]
#[template(path = "customer/deposit.html")]
pub struct DepositTemplate {
    pub accounts: Vec<Product>,
    pub selected_account_number: String,
    pub balance: String,
    pub error: String,
    pub has_error: bool,
    pub success: String,
    pub has_success: bool,
}

#[derive(Template)]
#[template(path = "customer/transfer.html")]
pub struct TransferTemplate {
    pub accounts: Vec<Product>,
    pub selected_account_number: String,
    pub balance: String,
    pub error: String,
    pub has_error: bool,
}


#[derive(Template)]
#[template(path = "customer/statements.html")]
pub struct StatementTemplate {
    pub accounts: Vec<Product>,
    pub has_accounts: bool,
    pub selected_account_id: String,
    pub start_date: String,
    pub end_date: String,
    pub transactions: Vec<StatementTransaction>,
    pub has_transactions: bool,
    pub error: String,
    pub has_error: bool,
}

#[derive(Template)]
#[template(path = "customer/transactions.html")]
pub struct TransactionsTemplate {
    pub transactions: Vec<Transaction>,
    pub has_transactions: bool,
}

#[derive(Template)]
#[template(path = "customer/activity_log.html")]
pub struct CustomerActivityLogTemplate {
    pub eyebrow: &'static str,
    pub title: &'static str,
    pub description: &'static str,
    pub icon: &'static str,
    pub empty_title: &'static str,
    pub empty_message: &'static str,
    pub transactions: Vec<Transaction>,
    pub has_transactions: bool,
}


#[derive(Template)]
#[template(path = "customer/cards.html")]
pub struct CardDashboardTemplate {
    pub cards: Vec<Card>,
    pub has_cards: bool,
    pub accounts: Vec<Product>,
    pub has_accounts: bool,
    pub error: String,
    pub has_error: bool,
    pub success: String,
    pub has_success: bool,
}


#[derive(Template)]
#[template(path = "customer/transaction_controls.html")]
pub struct TransactionControlsTemplate {
    pub controls: TransactionControl,
    pub alerts: Vec<FraudAlert>,
    pub has_alerts: bool,
    pub error: String,
    pub has_error: bool,
    pub success: String,
    pub has_success: bool,
}

#[derive(Template)]
#[template(path = "customer/giro.html")]
pub struct GiroTemplate {
    pub accounts: Vec<Product>,
    pub has_accounts: bool,
    pub arrangements: Vec<GiroArrangement>,
    pub has_arrangements: bool,
    pub error: String,
    pub has_error: bool,
    pub success: String,
    pub has_success: bool,
}

#[derive(Template)]
#[template(path = "customer/paynow.html")]
pub struct PayNowTemplate {
    pub accounts: Vec<Product>,
    pub has_accounts: bool,
    pub registrations: Vec<PayNowRegistration>,
    pub has_registrations: bool,
    pub error: String,
    pub has_error: bool,
    pub success: String,
    pub has_success: bool
}

#[derive(Template)]
#[template(path = "customer/paynow_register.html")]
pub struct PayNowRegisterTemplate {
    pub accounts: Vec<Product>,
    pub nric: String,
    pub phone: String,
    pub has_success: bool,
    pub has_error: bool,
    pub success: String,
    pub error: String
}

#[derive(Template)]
#[template(path = "customer/profile.html")]
pub struct ProfileTemplate {
    pub full_name: String,
    pub email: String,
    pub phone: String,
    pub date_of_birth: String,
    pub last_login: String,
    pub accounts: Vec<Product>,
    pub has_accounts: bool,
    pub paynow_registrations: Vec<PayNowRegistration>,
}

#[derive(Template)]
#[template(path = "errors/403.html")]
pub struct ForbiddenTemplate;

#[derive(Template)]
#[template(path = "errors/404.html")]
pub struct NotFoundTemplate;

#[derive(Template)]
#[template(path = "errors/error.html")]
pub struct ErrorTemplate;

#[derive(Template)]
#[template(path = "customer/loans.html")]
pub struct LoanDashboardTemplate {
    pub account: Product,
    pub accounts: Vec<Product>,
    pub loans: Vec<PersonalLoan>,
    pub has_loans: bool,
    pub error: String,
    pub has_error: bool,
}

#[derive(Template)]
#[template(path = "customer/loan_apply.html")]
pub struct LoanApplyTemplate {
    pub accounts: Vec<Product>,
    pub has_accounts: bool,
    pub error: String,
    pub has_error: bool,
}

#[derive(Template)]
#[template(path = "customer/home_loans.html")]
pub struct HomeLoanDashboardTemplate {
    pub account: Product,
    pub accounts: Vec<Product>,
    pub summary: HomeLoanSummary,
    pub applications: Vec<HomeLoanApplication>,
    pub has_applications: bool,
    pub error: String,
    pub has_error: bool,
}

#[derive(Template)]
#[template(path = "customer/home_loan_apply.html")]
pub struct HomeLoanApplyTemplate {
    pub accounts: Vec<Product>,
    pub has_accounts: bool,
    pub error: String,
    pub has_error: bool,
}

#[derive(Template)]
#[template(path = "admin/dashboard.html")]
pub struct AdminDashboardTemplate {
    pub summary: AdminDashboardSummary,
}

#[derive(Template)]
#[template(path = "admin/signups.html")]
pub struct AdminCustomerApplicationsTemplate {
    pub applications: Vec<AdminCustomerApplication>,
    pub has_applications: bool,
    pub error: String,
    pub has_error: bool,
}



#[derive(Template)]
#[template(path = "admin/high_value_monitoring.html")]
pub struct AdminHighValueMonitoringTemplate {
    pub alerts: Vec<HighValueAlertRecord>,
    pub has_alerts: bool,
    pub blocked_count: i64,
    pub flagged_count: i64,
    pub cleared_count: i64,
    pub error: String,
    pub has_error: bool,
}

#[derive(Template)]
#[template(path = "admin/staff.html")]
pub struct AdminStaffTemplate {
    pub staff_users: Vec<AdminStaffUser>,
    pub current_admin_id: Uuid,
    pub error: String,
    pub has_error: bool,
    pub success: String,
    pub has_success: bool,
}

#[derive(Template)]
#[template(path = "admin/accounts.html")]
pub struct AdminCustomerAccountsTemplate {
    pub accounts: Vec<AdminCustomerAccountRecord>,
    pub has_accounts: bool,
    pub error: String,
    pub has_error: bool,
}

#[derive(Template)]
#[template(path = "admin/audit_log.html")]
pub struct AdminAuditLogTemplate {
    pub logs: Vec<AdminAuditLogRecord>,
    pub has_logs: bool,
    pub error: String,
    pub has_error: bool,
}

#[derive(Template)]
#[template(path = "admin/personal_loans.html")]
pub struct AdminPersonalLoansTemplate {
    pub loans: Vec<AdminPersonalLoanRecord>,
    pub has_loans: bool,
    pub error: String,
    pub has_error: bool,
}

#[derive(Template)]
#[template(path = "admin/home_loans.html")]
pub struct AdminHomeLoansTemplate {
    pub records: Vec<AdminHomeLoanRecord>,
    pub has_records: bool,
    pub error: String,
    pub has_error: bool,
}

#[derive(Template)]
#[template(path = "customer/fixed_deposits.html")]
pub struct FixedDepositDashboardTemplate {
    pub account_number: String,
    pub balance: String,
    pub accounts: Vec<Product>,
    pub summary: FixedDepositSummary,
    pub fixed_deposits: Vec<FixedDeposit>,
    pub has_fixed_deposits: bool,
    pub success: String,
    pub has_success: bool,
    pub error: String,
    pub has_error: bool,
}

#[derive(Template)]
#[template(path = "customer/fixed_deposit_new.html")]
pub struct FixedDepositCreateTemplate {
    pub account_number: String,
    pub balance: String,
    pub accounts: Vec<Product>,
    pub plans: Vec<FixedDepositPlan>,
    pub has_plans: bool,
    pub error: String,
    pub has_error: bool,
}

#[derive(Template)]
#[template(path = "admin/fixed_deposits.html")]
pub struct AdminFixedDepositsTemplate {
    pub records: Vec<FixedDepositAdminRecord>,
    pub has_records: bool,
}

#[derive(Template)]
#[template(path = "admin/fixed_deposit_plans.html")]
pub struct AdminFixedDepositPlansTemplate {
    pub plans: Vec<FixedDepositPlan>,
    pub error: String,
    pub has_error: bool,
    pub success: String,
    pub has_success: bool,
}


#[derive(Template)]
#[template(path = "atm/index.html")]
pub struct ATMPageTemplate {
    pub has_error: bool,
    pub error: String
}

#[derive(Template)]
#[template(path = "atm/pin.html")]
pub struct ATMPinTemplate {
    pub has_error: bool,
    pub error: String,
    pub card_number_last_4: String
}


#[derive(Template)]
#[template(path = "atm/menu.html")]
pub struct ATMMenuTemplate {
    pub account_balance: String,
    pub card_number_last_4: String
}

#[derive(Template)]
#[template(path = "atm/deposit.html")]
pub struct ATMDepositTemplate {
    pub card_number_last_4: String,
    pub error: String,
    pub has_error: bool
}

#[derive(Template)]
#[template(path = "atm/deposit-success.html")]
pub struct ATMDepositSuccessTemplate {
    pub card_number_last_4: String,
    pub amount: String,
    pub account_balance: String
}

#[derive(Template)]
#[template(path = "atm/withdraw.html")]
pub struct ATMWithdrawalTemplate {
    pub card_number_last_4: String,
    pub account_balance: String,
    pub error: String,
    pub has_error: bool
}

#[derive(Template)]
#[template(path = "atm/withdraw-success.html")]
pub struct ATMWithdrawalSuccessTemplate {
    pub card_number_last_4: String,
    pub amount: String,
    pub account_balance: String
}

