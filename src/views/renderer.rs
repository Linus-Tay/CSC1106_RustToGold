
use actix_web::{error::ErrorInternalServerError, HttpResponse, Result};
use askama::Template;

// Render Askama template into HTTP response
pub fn render<T: Template>(template: T) -> Result<HttpResponse> {
    let html = template.render().map_err(ErrorInternalServerError)?;
    Ok(HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(html))
}

// Handle render html
pub fn render_html(html: String) -> Result<HttpResponse> {
    Ok(HttpResponse::Ok().content_type("text/html").body(html))
}
