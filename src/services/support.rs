// Service layer: keeps banking validation and workflow rules away from templates and SQL.

pub fn clean_optional_text(value: &str) -> Option<String> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.chars().take(160).collect())
    }
}
