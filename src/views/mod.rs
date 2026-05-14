pub mod renderer;
pub mod templates;

pub use self::renderer::render;
pub use self::templates::{
    CustomerPageTemplate, DashboardTemplate, DepositTemplate, ErrorTemplate, ForbiddenTemplate,
    HomeTemplate, LoginTemplate, NotFoundTemplate, ProfileTemplate, SignupTemplate,
    TransactionsTemplate,
};
