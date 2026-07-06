
pub mod renderer;
pub mod templates;

pub use self::renderer::render;
pub use self::templates::{
    AboutTemplate, AdminAuditLogTemplate, AdminCustomerAccountsTemplate,
    AdminCustomerApplicationsTemplate, AdminDashboardTemplate, AdminFixedDepositPlansTemplate,
    AdminFixedDepositsTemplate, AdminHighValueMonitoringTemplate, AdminHomeLoansTemplate,
    AdminLoginTemplate, AdminPersonalLoansTemplate, AdminStaffTemplate, BankingTemplate,
    CardDashboardTemplate, ContactTemplate, CustomerActivityLogTemplate, DashboardTemplate,
    ErrorTemplate, FaqTemplate, FixedDepositCreateTemplate, FixedDepositDashboardTemplate,
    ForbiddenTemplate, GiroTemplate, HomeLoanApplyTemplate, HomeLoanDashboardTemplate,
    HomeTemplate, LoanApplyTemplate, LoanDashboardTemplate, LoginTemplate, NotFoundTemplate,
    OnboardingResultTemplate, PayNowTemplate, ProfileTemplate, SecurityTemplate,
    StatementTemplate, TransactionControlsTemplate, TransactionsTemplate, TransferTemplate,
};
