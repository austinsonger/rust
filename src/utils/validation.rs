use tracing::warn;

/// Validation error type
#[derive(Debug)]
pub struct ValidationError {
    pub field: String,
    pub message: String,
}

/// Validate a non-empty string
///
/// # Arguments
/// * `value` - The string to validate
/// * `field_name` - The name of the field for error messages
///
/// # Returns
/// * `Result<(), ValidationError>` - Success or a validation error
pub fn validate_non_empty_string(value: &str, field_name: &str) -> Result<(), ValidationError> {
    if value.trim().is_empty() {
        warn!("{} is empty or contains only whitespace", field_name);
        return Err(ValidationError {
            field: field_name.to_string(),
            message: format!("{} cannot be empty", field_name),
        });
    }

    Ok(())
}

/// Validate a password
///
/// # Arguments
/// * `password` - The password to validate
///
/// # Returns
/// * `Result<(), ValidationError>` - Success or a validation error
pub fn validate_password(password: &str) -> Result<(), ValidationError> {
    const MIN_PASSWORD_LENGTH: usize = 8;

    if password.len() < MIN_PASSWORD_LENGTH {
        warn!(
            "Password too short: {} characters (minimum: {})",
            password.len(),
            MIN_PASSWORD_LENGTH
        );
        return Err(ValidationError {
            field: "password".to_string(),
            message: format!("Password must be at least {} characters long", MIN_PASSWORD_LENGTH),
        });
    }

    // Additional password validation rules could be added here
    // For example, requiring uppercase, lowercase, numbers, special characters, etc.

    Ok(())
}

/// Validate an email address
///
/// # Arguments
/// * `email` - The email to validate
///
/// # Returns
/// * `Result<(), ValidationError>` - Success or a validation error
pub fn validate_email(email: &str) -> Result<(), ValidationError> {
    // Basic email validation
    if !email.contains('@') || !email.contains('.') {
        warn!("Invalid email format: {}", email);
        return Err(ValidationError {
            field: "email".to_string(),
            message: "Invalid email format".to_string(),
        });
    }

    // More sophisticated email validation could be added here

    Ok(())
}

/// Validate a numeric value is within a range
///
/// # Type Parameters
/// * `T` - The numeric type
///
/// # Arguments
/// * `value` - The value to validate
/// * `min` - The minimum allowed value
/// * `max` - The maximum allowed value
/// * `field_name` - The name of the field for error messages
///
/// # Returns
/// * `Result<(), ValidationError>` - Success or a validation error
pub fn validate_range<T>(value: T, min: T, max: T, field_name: &str) -> Result<(), ValidationError>
where
    T: PartialOrd + std::fmt::Debug,
{
    if value < min || value > max {
        warn!(
            "{} out of range: {:?} (min: {:?}, max: {:?})",
            field_name, value, min, max
        );
        return Err(ValidationError {
            field: field_name.to_string(),
            message: format!("{} must be between {:?} and {:?}", field_name, min, max),
        });
    }

    Ok(())
}
