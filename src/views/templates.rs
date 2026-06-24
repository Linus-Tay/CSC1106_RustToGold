use crate::models::{FixedDeposit, FixedDepositPlan, FixedDepositSummary, StaffUser, Transaction, AuditLogEntry, BankAccount};
use askama::Template;

#[derive(Template)]
#[template(path = "index.html")]
pub struct HomeTemplate;

#[derive(Template)]
#[template(path = "onboarding.html")]
pub struct OnboardingTemplate {
    pub product_available: bool,
    pub product_id: String,
    pub channel: String,
    pub product_name: String,
    pub product_summary: String,
    pub product_rate: String,
    pub product_minimum: String,
    pub product_features: Vec<String>,
    pub action_url: String,
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
    pub error: String,
    pub has_error: bool,
}

#[derive(Template)]
#[template(path = "admin/dashboard.html")]
pub struct AdminDashboardTemplate {
    pub total_deposits: i64,
    pub active_plans: i64,
    pub staff_count: i64,
    pub transaction_count: i64,
    pub recent_audit: Vec<AuditLogEntry>,
    pub has_recent_audit: bool,
}

#[derive(Template)]
#[template(path = "admin/fixed_deposits.html")]
pub struct AdminFixedDepositsTemplate {
    pub fixed_deposits: Vec<FixedDeposit>,
    pub has_fixed_deposits: bool,
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
#[template(path = "admin/staff.html")]
pub struct AdminStaffDashboardTemplate {
    pub staff_users: Vec<StaffUser>,
    pub has_staff: bool,
    pub success: String,
    pub has_success: bool,
    pub error: String,
    pub has_error: bool,
}

#[derive(Template)]
#[template(path = "admin/staff_edit.html")]
pub struct AdminStaffEditTemplate {
    pub staff: Option<StaffUser>,
    pub error: String,
    pub has_error: bool,
}

#[derive(Template)]
#[template(path = "admin/audit_log.html")]
pub struct AdminAuditLogTemplate {
    pub entries: Vec<AuditLogEntry>,
    pub has_entries: bool,
    pub total_count: i64,
    pub page: i64,
    pub total_pages: i64,
    pub filter_action: String,
    pub filter_status: String,
    pub filter_entity_type: String,
    pub filter_user_id: String,
}

#[derive(Template)]
#[template(path = "admin/transactions.html")]
pub struct AdminTransactionsTemplate {
    pub transactions: Vec<Transaction>,
    pub has_transactions: bool,
    pub total_count: i64,
    pub page: i64,
    pub total_pages: i64,
    pub filter_transaction_type: String,
    pub filter_user_id: String,
    pub filter_account_id: String,
}

#[derive(Template)]
#[template(path = "admin/accounts.html")]
pub struct AdminAccountsTemplate {
    pub accounts: Vec<BankAccount>,
    pub has_accounts: bool,
    pub total_count: i64,
    pub page: i64,
    pub total_pages: i64,
    pub filter_status: String,
    pub success: String,
    pub has_success: bool,
    pub error: String,
    pub has_error: bool,
}