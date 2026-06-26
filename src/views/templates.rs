use crate::models::Transaction;
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
    pub has_selected_product: bool,
    pub selected_product_id: String,
    pub selected_product_name: String,
    pub selected_product_summary: String,
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
