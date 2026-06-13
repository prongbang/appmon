use std::path::Path;

use crate::{AppError, AppResult};

pub fn non_empty(value: &str, field: &str) -> AppResult<()> {
    if value.trim().is_empty() {
        return Err(AppError::InvalidInput(format!("{field} cannot be empty")));
    }
    Ok(())
}

pub fn package_or_bundle_id(value: &str) -> AppResult<()> {
    non_empty(value, "package_or_bundle_id")?;
    if value
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '.' || c == '_' || c == '-')
    {
        Ok(())
    } else {
        Err(AppError::InvalidInput(
            "package_or_bundle_id contains unsupported characters".to_string(),
        ))
    }
}

pub fn readable_file(path: &Path) -> AppResult<()> {
    if path.is_file() {
        Ok(())
    } else {
        Err(AppError::InvalidInput(format!(
            "file does not exist: {}",
            path.display()
        )))
    }
}

pub fn text_input(value: &str) -> AppResult<()> {
    if value.len() > 4096 {
        return Err(AppError::InvalidInput("text is too long".to_string()));
    }
    if value.chars().any(|c| matches!(c, '\n' | '\r' | '\0')) {
        return Err(AppError::InvalidInput(
            "text contains unsupported control characters".to_string(),
        ));
    }
    Ok(())
}

pub fn key_input(value: &str) -> AppResult<()> {
    non_empty(value, "key")?;
    if value
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || matches!(c, '_' | '-' | '.'))
    {
        Ok(())
    } else {
        Err(AppError::InvalidInput(
            "key contains unsupported characters".to_string(),
        ))
    }
}

pub fn coordinate(value: u32, field: &str) -> AppResult<()> {
    if value <= 100_000 {
        Ok(())
    } else {
        Err(AppError::InvalidInput(format!("{field} is out of range")))
    }
}
