pub mod account_service;
pub mod auth_service;
pub mod profile_service;
pub mod support;
pub mod onboard_service;

pub use self::account_service::{deposit, list_transactions, load_customer_dashboard};
pub use self::auth_service::{authenticate_user, register_customer};
pub use self::profile_service::update_customer_profile;
pub use self::onboard_service::{get_product_details, get_path_template};