// View layer: Askama template structs and rendering helpers.

use crate::models::{
    AdminAuditLogRecord, AdminCustomerAccountRecord, AdminCustomerApplication, AdminDashboardSummary, AdminHomeLoanRecord, AdminPersonalLoanRecord, AdminStaffUser, Card, FixedDeposit, FixedDepositAdminRecord, FixedDepositPlan, FixedDepositSummary, FraudAlert, GiroArrangement, HighValueAlertRecord, HomeLoanApplication, HomeLoanSummary, PayNowRegistration, PersonalLoan, Product, StatementTransaction, Transaction, TransactionControl,
};
use askama::Template;
use uuid::Uuid;

#[derive(Template)]
#[template(path = "index.html")]
// Data carrier for the HomeTemplate workflow.
pub struct HomeTemplate;

#[derive(Template)]
#[template(path = "about.html")]
// Data carrier for the AboutTemplate workflow.
pub struct AboutTemplate;

#[derive(Template)]
#[template(path = "banking.html")]
// Data carrier for the BankingTemplate workflow.
pub struct BankingTemplate;

#[derive(Template)]
#[template(path = "contact.html")]
// Data carrier for the ContactTemplate workflow.
pub struct ContactTemplate;

#[derive(Template)]
#[template(path = "faq.html")]
// Data carrier for the FaqTemplate workflow.
pub struct FaqTemplate;

#[derive(Template)]
#[template(path = "security.html")]
// Data carrier for the SecurityTemplate workflow.
pub struct SecurityTemplate;

#[derive(Template)]
#[template(path = "onboarding/onboarding.html")]
// Data carrier for the OnboardingTemplate workflow.
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
// Data carrier for the OnboardingResultTemplate workflow.
pub struct OnboardingResultTemplate {
    pub reference_no: String,
    pub created_at: String,
}

#[derive(Template)]
#[template(path = "onboarding/forms/onboarding_account.html")]
// Data carrier for the OnboardingAccountTemplate workflow.
pub struct OnboardingAccountTemplate {
    pub error: String,
    pub has_error: bool,
    pub selected_account_type: String,
    pub preferred_account_name: String,
    pub account_purpose: String,
}

#[derive(Template)]
#[template(path = "onboarding/forms/onboarding_personal.html")]
// Data carrier for the OnboardingPersonalTemplate workflow.
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
// Data carrier for the OnboardingContactTemplate workflow.
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
// Data carrier for the OnboardingEmploymentTemplate workflow.
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
// Data carrier for the OnboardingReviewTemplate workflow.
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
// Data carrier for the AccountCreationSetupTemplate workflow.
pub struct AccountCreationSetupTemplate {
    pub email: String,
    pub error: String,
    pub has_error: bool,
}

#[derive(Template)]
#[template(path = "onboarding/account_creation_success.html")]
// Data carrier for the AccountCreationSuccessTemplate workflow.
pub struct AccountCreationSuccessTemplate {
    pub username: String,
    pub email: String,
}

#[derive(Template)]
#[template(path = "email/account_creation_email.html")]
// Data carrier for the AccountCreationEmailTemplate workflow.
pub struct AccountCreationEmailTemplate {
    pub account_creation_link: String,
}

#[derive(Template)]
#[template(path = "email/account_2fa_email.html")]
// Data carrier for the AccountCreationEmailTemplate workflow.
pub struct Account2FAEmailTemplate {
    pub verification_code: String,
}


#[derive(Template)]
#[template(path = "email/application_received_email.html")]
// Data carrier for the ApplicationReceivedEmailTemplate workflow.
pub struct ApplicationReceivedEmailTemplate;


#[derive(Template)]
#[template(path = "admin/login.html")]
// Data carrier for the AdminLoginTemplate workflow.
pub struct AdminLoginTemplate {
    pub error: String,
    pub has_error: bool,
}

#[derive(Template)]
#[template(path = "auth/login.html")]
// Data carrier for the LoginTemplate workflow.
pub struct LoginTemplate {
    pub error: String,
    pub has_error: bool,
}

#[derive(Template)]
#[template(path = "auth/2fa.html")]
// Data carrier for the AdminFixedDepositPlansTemplate workflow.
pub struct TwoFactorAuthTemplate {
    pub error: String,
    pub has_error: bool,
}

#[derive(Template)]
#[template(path = "customer/dashboard.html")]
// Data carrier for the DashboardTemplate workflow.
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
// Data carrier for the DepositTemplate workflow.
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
// Data carrier for the TransferTemplate workflow.
pub struct TransferTemplate {
    pub accounts: Vec<Product>,
    pub selected_account_number: String,
    pub balance: String,
    pub error: String,
    pub has_error: bool,
}


#[derive(Template)]
#[template(path = "customer/statements.html")]
// Data carrier for the StatementTemplate workflow.
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
// Data carrier for the TransactionsTemplate workflow.
pub struct TransactionsTemplate {
    pub transactions: Vec<Transaction>,
    pub has_transactions: bool,
}

#[derive(Template)]
#[template(path = "customer/activity_log.html")]
// Data carrier for the CustomerActivityLogTemplate workflow.
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
// Data carrier for the CardDashboardTemplate workflow.
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
// Data carrier for the TransactionControlsTemplate workflow.
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
// Data carrier for the GiroTemplate workflow.
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
// Data carrier for the PayNowTemplate workflow.
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
// Data carrier for the PayNowTemplate workflow.
pub struct PayNowRegisterTemplate {
    pub accounts: Vec<Product>,
    pub nric: String,
    pub phone: String,
    pub has_success: bool,
    pub has_error: bool,
    pub success: String,
    pub error: String
}

// #[derive(Template)]
// #[template(path = "customer/profile.html")]
// // Data carrier for the ProfileTemplate workflow.
// pub struct ProfileTemplate {
//     pub full_name: String,
//     pub email: String,
//     pub phone: String,
//     pub date_of_birth: String,
//     pub last_login: String,
//     pub accounts: Vec<Product>,
//     pub has_accounts: bool,
//     pub paynow_id: String,
//     pub paynow_linked_product_id: String,
//     pub has_paynow: bool,
// }

#[derive(Template)]
#[template(path = "customer/profile.html")]
// Data carrier for the ProfileTemplate workflow.
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

// #[derive(Template)]
// #[template(path = "customer/home_loans.html")]
// pub struct HomeLoanDashboardTemplate {
//     pub account: Product,
//     pub summary: HomeLoanSummary,
//     pub applications: Vec<HomeLoanApplication>,
//     pub has_applications: bool,
//     pub error: String,  
//     pub has_error: bool,
// }

// #[derive(Template)]
// #[template(path = "customer/home_loan_apply.html")]
// pub struct HomeLoanApplyTemplate {
//     pub accounts: Vec<Product>,
//     pub has_accounts: bool,
//     pub error: String,
//     pub has_error: bool,
// }

// #[derive(Template)]
// #[template(path = "admin/home_loans.html")]
// pub struct AdminHomeLoansTemplate {
//     pub records: Vec<AdminHomeLoanRecord>,
//     pub has_records: bool,
//     pub error: String,
//     pub has_error: bool,
// }

#[derive(Template)]
#[template(path = "errors/403.html")]
// Data carrier for the ForbiddenTemplate workflow.
pub struct ForbiddenTemplate;

#[derive(Template)]
#[template(path = "errors/404.html")]
// Data carrier for the NotFoundTemplate workflow.
pub struct NotFoundTemplate;

#[derive(Template)]
#[template(path = "errors/error.html")]
// Data carrier for the ErrorTemplate workflow.
pub struct ErrorTemplate;

#[derive(Template)]
#[template(path = "customer/loans.html")]
// Data carrier for the LoanDashboardTemplate workflow.
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
// Data carrier for the LoanApplyTemplate workflow.
pub struct LoanApplyTemplate {
    pub accounts: Vec<Product>,
    pub has_accounts: bool,
    pub error: String,
    pub has_error: bool,
}

#[derive(Template)]
#[template(path = "customer/home_loans.html")]
// Data carrier for the HomeLoanDashboardTemplate workflow.
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
// Data carrier for the HomeLoanApplyTemplate workflow.
pub struct HomeLoanApplyTemplate {
    pub accounts: Vec<Product>,
    pub has_accounts: bool,
    pub error: String,
    pub has_error: bool,
}

#[derive(Template)]
#[template(path = "admin/dashboard.html")]
// Data carrier for the AdminDashboardTemplate workflow.
pub struct AdminDashboardTemplate {
    pub summary: AdminDashboardSummary,
}

#[derive(Template)]
#[template(path = "admin/signups.html")]
// Data carrier for the AdminCustomerApplicationsTemplate workflow.
pub struct AdminCustomerApplicationsTemplate {
    pub applications: Vec<AdminCustomerApplication>,
    pub has_applications: bool,
    pub error: String,
    pub has_error: bool,
}



#[derive(Template)]
#[template(path = "admin/high_value_monitoring.html")]
// Data carrier for the AdminHighValueMonitoringTemplate workflow.
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
// Data carrier for the AdminStaffTemplate workflow.
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
// Data carrier for the AdminCustomerAccountsTemplate workflow.
pub struct AdminCustomerAccountsTemplate {
    pub accounts: Vec<AdminCustomerAccountRecord>,
    pub has_accounts: bool,
    pub error: String,
    pub has_error: bool,
}

#[derive(Template)]
#[template(path = "admin/audit_log.html")]
// Data carrier for the AdminAuditLogTemplate workflow.
pub struct AdminAuditLogTemplate {
    pub logs: Vec<AdminAuditLogRecord>,
    pub has_logs: bool,
    pub error: String,
    pub has_error: bool,
}

#[derive(Template)]
#[template(path = "admin/personal_loans.html")]
// Data carrier for the AdminPersonalLoansTemplate workflow.
pub struct AdminPersonalLoansTemplate {
    pub loans: Vec<AdminPersonalLoanRecord>,
    pub has_loans: bool,
    pub error: String,
    pub has_error: bool,
}

#[derive(Template)]
#[template(path = "admin/home_loans.html")]
// Data carrier for the AdminHomeLoansTemplate workflow.
pub struct AdminHomeLoansTemplate {
    pub records: Vec<AdminHomeLoanRecord>,
    pub has_records: bool,
    pub error: String,
    pub has_error: bool,
}

#[derive(Template)]
#[template(path = "customer/fixed_deposits.html")]
// Data carrier for the FixedDepositDashboardTemplate workflow.
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
// Data carrier for the FixedDepositCreateTemplate workflow.
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
// Data carrier for the AdminFixedDepositsTemplate workflow.
pub struct AdminFixedDepositsTemplate {
    pub records: Vec<FixedDepositAdminRecord>,
    pub has_records: bool,
}

#[derive(Template)]
#[template(path = "admin/fixed_deposit_plans.html")]
// Data carrier for the AdminFixedDepositPlansTemplate workflow.
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
    pub error: String
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

