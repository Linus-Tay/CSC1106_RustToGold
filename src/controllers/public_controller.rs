// Controller layer: handles HTTP/session flow and delegates business rules to services.

use crate::views::{
    render, AboutTemplate, BankingTemplate, ContactTemplate, FaqTemplate, HomeTemplate,
    SecurityTemplate,
};
use actix_web::{HttpResponse, Result};

// Renders the home screen with data prepared by the service layer.
pub async fn home() -> Result<HttpResponse> {
    render(HomeTemplate)
}

// Renders the banking page screen with data prepared by the service layer.
pub async fn banking_page() -> Result<HttpResponse> {
    render(BankingTemplate)
}

// Renders the security page screen with data prepared by the service layer.
pub async fn security_page() -> Result<HttpResponse> {
    render(SecurityTemplate)
}

// Renders the about page screen with data prepared by the service layer.
pub async fn about_page() -> Result<HttpResponse> {
    render(AboutTemplate)
}

// Renders the faq page screen with data prepared by the service layer.
pub async fn faq_page() -> Result<HttpResponse> {
    render(FaqTemplate)
}

// Renders the contact page screen with data prepared by the service layer.
pub async fn contact_page() -> Result<HttpResponse> {
    render(ContactTemplate)
}
