
use crate::controllers::session_guard::{admin_session_user_id, customer_session_user_id, redirect};
use crate::views::{
    render, AboutTemplate, BankingTemplate, ContactTemplate, FaqTemplate, HomeTemplate,
    SecurityTemplate,
};
use actix_session::Session;
use actix_web::{HttpResponse, Result};

// Render home
pub async fn home(session: Session) -> Result<HttpResponse> {
    if customer_session_user_id(&session).is_some() {
        return Ok(redirect("/customer/dashboard"));
    }

    if admin_session_user_id(&session).is_some() {
        return Ok(redirect("/admin/dashboard"));
    }

    render(HomeTemplate)
}

// Render banking page
pub async fn banking_page() -> Result<HttpResponse> {
    render(BankingTemplate)
}

// Render security page
pub async fn security_page() -> Result<HttpResponse> {
    render(SecurityTemplate)
}

// Render about page
pub async fn about_page() -> Result<HttpResponse> {
    render(AboutTemplate)
}

// Render faq page
pub async fn faq_page() -> Result<HttpResponse> {
    render(FaqTemplate)
}

// Render contact page
pub async fn contact_page() -> Result<HttpResponse> {
    render(ContactTemplate)
}
