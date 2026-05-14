use crate::views::{render, ErrorTemplate, ForbiddenTemplate, NotFoundTemplate};
use actix_web::{HttpResponse, Result};

pub async fn forbidden() -> Result<HttpResponse> {
    render(ForbiddenTemplate)
}

pub async fn not_found() -> Result<HttpResponse> {
    render(NotFoundTemplate)
}

pub fn render_error(_heading: &'static str, _message: String) -> Result<HttpResponse> {
    render(ErrorTemplate)
}
