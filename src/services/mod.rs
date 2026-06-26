pub mod account_service;
pub mod auth_service;
pub mod profile_service;
pub mod support;
pub mod onboard_service;
pub mod product_service;
pub mod email_service;
pub mod customer_service;

pub use self::account_service::{list_transactions, load_customer_dashboard};
pub use self::auth_service::{authenticate_user, register_customer};
pub use self::profile_service::update_customer_profile;
pub use self::onboard_service::{get_product_details, get_path_template};
pub use self::product_service::{create_product, deposit, transfer, approve_product};
pub use self::email_service::{send_template_email};
pub use self::customer_service::{create_customer};