use crate::models::Transaction;
use askama::Template;

#[derive(Template)]
#[template(path = "index.html")]
pub struct HomeTemplate;

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
#[template(path= "onboarding/forms/onboarding_account.html")]
pub struct OnboardingAccountTemplate {
    pub error: String,
    pub has_error: bool,
    pub selected_account_type: String,
    pub preferred_account_name: String,
    pub account_purpose: String,
}

#[derive(Template)]
#[template(path= "onboarding/forms/onboarding_personal.html")]
pub struct OnboardingPersonalTemplate {
    pub error: String,
    pub has_error: bool,
    pub full_name: String,
    pub nric: String,
    pub dob: String,
    pub nationality: String,
    pub residential_status: String,
    pub residential_address: String,
    pub step1_completed: bool,
    pub gender: String,
    pub race: String,
    pub identity_confirmed: bool
}

#[derive(Template)]
#[template(path= "onboarding/forms/onboarding_contact.html")]
pub struct OnboardingContactTemplate {
    pub error: String,
    pub has_error: bool,
    pub email: String,
    pub phone_number: String,
    pub mailing_address: String,
    pub step1_completed: bool,
    pub step2_completed: bool
}

#[derive(Template)]
#[template(path= "onboarding/forms/onboarding_employment.html")]
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
    pub step3_completed: bool
}

#[derive(Template)]
#[template(path= "onboarding/forms/onboarding_review.html")]
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
    pub step4_completed: bool
}

#[derive(Template)]
#[template(path= "onboarding/onboarding_result.html")]
pub struct OnboardingResultTemplate {
    pub reference_no: String,
    pub created_at: String,
}

#[derive(Template)]
#[template(path ="onboarding/account_creation.html")]
pub struct AccountCreationTemplate {
    pub email: String
}

#[derive(Template)]
#[template(path = "onboarding/account_creation_success.html")]
pub struct AccountCreationSuccessTemplate {
    pub username: String,
    pub email: String,
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
    pub password_created: bool,
}

#[derive(Template)]
#[template(path = "customer/dashboard.html")]
pub struct DashboardTemplate {
    pub full_name: String,
    pub account_number: String,
    pub balance: String,
    pub recent_transactions: Vec<Transaction>,
    pub has_transactions: bool,
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
#[template(path = "customer/transactions.html")]
pub struct TransactionsTemplate {
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
#[template(path = "email/account_creation_email.html")]
pub struct AccountCreationEmailTemplate {
    pub account_creation_link: String,
}

#[derive(Template)]
#[template(path = "email/application_received_email.html")]
pub struct ApplicationReceivedEmailTemplate {}

