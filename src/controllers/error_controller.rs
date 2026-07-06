
use crate::views::{render, ErrorTemplate, ForbiddenTemplate, NotFoundTemplate};
use actix_web::{HttpResponse, Result};

// Render forbidden
pub async fn forbidden() -> Result<HttpResponse> {
    render(ForbiddenTemplate)
}

// Render not found
pub async fn not_found() -> Result<HttpResponse> {
    render(NotFoundTemplate)
}

// Render error
pub fn render_error(_heading: &'static str, _message: String) -> Result<HttpResponse> {
    render(ErrorTemplate)
}
