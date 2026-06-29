pub mod account_service;
pub mod auth_service;
pub mod profile_service;
pub mod support;
pub mod product_service;
pub mod email_service;
pub mod customer_service;

pub use self::account_service::{list_transactions, load_customer_dashboard, register_user};
pub use self::auth_service::{authenticate_user};
pub use self::profile_service::update_customer_profile;
pub use self::product_service::{create_product, deposit, transfer, generate_account_number};
pub use self::email_service::{send_template_email};
pub use self::customer_service::{create_customer_with_product, validate_account_creation_link, get_customer_by_account_creation_link, invalidate_account_creation_link};