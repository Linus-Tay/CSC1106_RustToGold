pub mod renderer;
pub mod templates;

pub use self::renderer::render;

pub use self::templates::{
    AboutTemplate, BankingTemplate, ContactTemplate, CustomerPageTemplate, DashboardTemplate,
    DepositTemplate, ErrorTemplate, FaqTemplate, ForbiddenTemplate, HomeTemplate, LoginTemplate,
    NotFoundTemplate, ProfileTemplate, SecurityTemplate, SignupTemplate,
    TransactionsTemplate,
};