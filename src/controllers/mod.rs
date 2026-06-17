pub mod account_controller;
pub mod auth_controller;
pub mod customer_controller;
pub mod error_controller;
pub mod public_controller;
pub mod session_guard;

pub use self::account_controller::{display_product, redirect_to_product_information, onboarding, step1_post};
pub use self::auth_controller::{login, login_page, logout, signup, signup_page};
pub use self::customer_controller::{
    dashboard, deposit, deposit_page, fixed_deposit_new_page, fixed_deposits_page,
    loan_apply_page, loans_page, profile_page, transactions, transfer_page, update_profile,
};
pub use self::error_controller::{forbidden, not_found};
pub use self::public_controller::home;
