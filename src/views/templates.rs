use crate::models::{
    AdminCustomerApplication, AdminDashboardSummary, AdminHomeLoanRecord, AdminPersonalLoanRecord,
    FixedDeposit, FixedDepositAdminRecord, FixedDepositPlan, FixedDepositSummary, HomeLoanApplication,
    HomeLoanSummary, PersonalLoan, Product, Transaction,
};
use askama::Template;

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
#[template(path= "onboarding/forms/onboarding_form1.html")]
pub struct OnboardingFormTemplate {
    pub test: bool,
}

#[derive(Template)]
#[template(path= "onboarding/forms/onboarding_form2.html")]
pub struct OnboardingFormTemplate1 {
    pub test: bool,
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
#[template(path = "auth/signup.html")]
pub struct SignupTemplate {
    pub error: String,
    pub has_error: bool,
}

#[derive(Template)]
#[template(path = "auth/signup_account.html")]
pub struct SignupAccountTemplate {
    pub error: String,
    pub has_error: bool,
    pub selected_account_type: String,
    pub preferred_account_name: String,
    pub account_purpose: String,
}

#[derive(Template)]
#[template(path = "auth/signup_personal.html")]
pub struct SignupPersonalTemplate {
    pub error: String,
    pub has_error: bool,
    pub full_name: String,
    pub nric_fin: String,
    pub date_of_birth: String,
    pub nationality: String,
    pub residential_status: String,
    pub residential_address: String,
}

#[derive(Template)]
#[template(path = "auth/signup_contact.html")]
pub struct SignupContactTemplate {
    pub error: String,
    pub has_error: bool,
    pub email: String,
    pub phone_number: String,
    pub mailing_address: String,
}

#[derive(Template)]
#[template(path = "auth/signup_employment.html")]
pub struct SignupEmploymentTemplate {
    pub error: String,
    pub has_error: bool,
    pub employment_status: String,
    pub occupation: String,
    pub employer_name: String,
    pub monthly_income_range: String,
    pub source_initial_deposit: String,
}

#[derive(Template)]
#[template(path = "auth/signup_security.html")]
pub struct SignupSecurityTemplate {
    pub error: String,
    pub has_error: bool,
}

#[derive(Template)]
#[template(path = "auth/signup_review.html")]
pub struct SignupReviewTemplate {
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
}

#[derive(Template)]
#[template(path = "customer/dashboard.html")]
pub struct DashboardTemplate {
    pub full_name: String,
    pub balance: String,
}

#[derive(Template)]
#[template(path = "customer/deposit.html")]
pub struct DepositTemplate {
    pub account_number: String,
    pub balance: String,
    pub error: String,
    pub has_error: bool,
    pub success: String,
    pub has_success: bool,
}

#[derive(Template)]
#[template(path = "customer/transfer.html")]
pub struct TransferTemplate {
    pub account_number: String,
    pub balance: String,
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
#[template(path = "customer/profile.html")]
pub struct ProfileTemplate {
    pub full_name: String,
    pub email: String,
    pub phone: String,
    pub date_of_birth: String,
    pub account_number: String,
    pub balance: String,
    pub account_type: String,
    pub status: String,
    pub created_at: String,
    pub last_login: String,
}

#[derive(Template)]
#[template(path = "customer/placeholder.html")]
pub struct CustomerPageTemplate {
    pub title: &'static str,
    pub active_nav: &'static str,
    pub heading: &'static str,
    pub description: &'static str,
    pub message: &'static str,
    pub primary_label: &'static str,
    pub primary_href: &'static str,
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
#[template(path = "email/account_creation.html")]
pub struct AccountCreationTemplate {
    pub account_creation_link: String,
}


#[derive(Template)]
#[template(path = "customer/loans.html")]
pub struct LoanDashboardTemplate {
    pub account: Product,
    pub loans: Vec<PersonalLoan>,
    pub has_loans: bool,
    pub error: String,
    pub has_error: bool,
}

#[derive(Template)]
#[template(path = "customer/loan_apply.html")]
pub struct LoanApplyTemplate {
    pub error: String,
    pub has_error: bool,
}

#[derive(Template)]
#[template(path = "customer/home_loans.html")]
pub struct HomeLoanDashboardTemplate {
    pub account: Product,
    pub summary: HomeLoanSummary,
    pub applications: Vec<HomeLoanApplication>,
    pub has_applications: bool,
    pub error: String,
    pub has_error: bool,
}

#[derive(Template)]
#[template(path = "customer/home_loan_apply.html")]
pub struct HomeLoanApplyTemplate {
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
