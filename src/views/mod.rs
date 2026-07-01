pub mod renderer;
pub mod templates;

pub use self::renderer::render;
pub use self::templates::{
    AboutTemplate, AccountCreationEmailTemplate, AccountCreationSetupTemplate, AccountCreationSuccessTemplate, ApplicationReceivedEmailTemplate, OnboardingAccountTemplate, OnboardingContactTemplate, OnboardingEmploymentTemplate, OnboardingPersonalTemplate, OnboardingReviewTemplate, AdminAuditLogTemplate, AdminCustomerAccountsTemplate, AdminCustomerApplicationsTemplate, AdminDashboardTemplate, AdminFixedDepositPlansTemplate, AdminFixedDepositsTemplate, AdminHighValueMonitoringTemplate, AdminHomeLoansTemplate, AdminLoginTemplate, AdminPersonalLoansTemplate, AdminStaffTemplate,
    BankingTemplate, ContactTemplate, CardDashboardTemplate, CustomerActivityLogTemplate, CustomerPageTemplate, DashboardTemplate, DepositTemplate,
    ErrorTemplate, FaqTemplate, FixedDepositCreateTemplate, FixedDepositDashboardTemplate, GiroTemplate,
    ForbiddenTemplate, HomeLoanApplyTemplate, HomeLoanDashboardTemplate, HomeTemplate, LoginTemplate,
    LoanApplyTemplate, LoanDashboardTemplate, NotFoundTemplate, OnboardingResultTemplate, OnboardingTemplate, PayNowTemplate, ProfileTemplate,
    SecurityTemplate, StatementTemplate, TransactionControlsTemplate, TransactionsTemplate, TransferTemplate,
};
