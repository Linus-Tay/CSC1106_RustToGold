pub mod auth_controller;
pub mod customer_controller;
pub mod error_controller;
pub mod public_controller;
pub mod onboard_controller;
pub mod session_guard;

pub use self::onboard_controller::{display_product, onboarding, redirect_to_product_information, step1_post, submit};
pub use self::auth_controller::{
    login, login_page, logout, post_signup_account, post_signup_contact, post_signup_employment,
    post_signup_personal, post_signup_security, post_signup_submit, show_signup_account,
    show_signup_contact, show_signup_employment, show_signup_personal, show_signup_review,
    show_signup_security, signup, signup_page,
};
pub use self::customer_controller::{
    dashboard, deposit, deposit_page, fixed_deposit_new_page, fixed_deposits_page,
    loan_apply_page, loans_page, profile_page, transactions, transfer, transfer_page, update_profile, approve_product
};
pub use self::error_controller::{forbidden, not_found};
pub use self::public_controller::{
    about_page, banking_page, contact_page, faq_page, home, security_page,
};
