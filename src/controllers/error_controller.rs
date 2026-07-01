// Controller layer: handles HTTP/session flow and delegates business rules to services.

use crate::views::{render, ErrorTemplate, ForbiddenTemplate, NotFoundTemplate};
use actix_web::{HttpResponse, Result};

// Handles the forbidden request.
pub async fn forbidden() -> Result<HttpResponse> {
    render(ForbiddenTemplate)
}

// Handles the not found request.
pub async fn not_found() -> Result<HttpResponse> {
    render(NotFoundTemplate)
}

// Handles the render error request.
pub fn render_error(_heading: &'static str, _message: String) -> Result<HttpResponse> {
    render(ErrorTemplate)
}
