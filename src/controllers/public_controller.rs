use actix_web::{HttpResponse, Result};

use crate::views::{
    render, AboutTemplate, BankingTemplate, ContactTemplate, FaqTemplate, HomeTemplate,
    SecurityTemplate,
};

pub async fn home() -> Result<HttpResponse> {
    render(HomeTemplate)
}

pub async fn banking_page() -> Result<HttpResponse> {
    render(BankingTemplate)
}

pub async fn security_page() -> Result<HttpResponse> {
    render(SecurityTemplate)
}

pub async fn about_page() -> Result<HttpResponse> {
    render(AboutTemplate)
}

pub async fn faq_page() -> Result<HttpResponse> {
    render(FaqTemplate)
}

pub async fn contact_page() -> Result<HttpResponse> {
    render(ContactTemplate)
}