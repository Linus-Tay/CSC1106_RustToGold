pub mod renderer;
pub mod templates;

pub use self::renderer::render;
pub use self::templates::{
    AboutTemplate, BankingTemplate, ContactTemplate, CustomerPageTemplate, DashboardTemplate,
    DepositTemplate, ErrorTemplate, FaqTemplate, ForbiddenTemplate, HomeTemplate, LoginTemplate,
    NotFoundTemplate, OnboardingFormTemplate, OnboardingFormTemplate1, OnboardingResultTemplate,
    OnboardingTemplate, ProfileTemplate, SecurityTemplate, SignupAccountTemplate,
    SignupContactTemplate, SignupEmploymentTemplate, SignupPersonalTemplate, SignupReviewTemplate,
    SignupSecurityTemplate, SignupTemplate, TransactionsTemplate,
};
