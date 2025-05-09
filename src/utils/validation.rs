use std::fmt::Debug;
use tracing::{debug, warn};
use validator::{Validate, ValidationErrors};

use crate::{
    constants::auth::MIN_PASSWORD_LENGTH,
    errors::Error,
};

/// Validate a struct that implements the Validate trait
/// 
/// # Type Parameters
/// * `T` - The type to validate
/// 
/// # Arguments
/// * `value` - The value to validate
/// * `resource_name` - The name of the resource for error messages
/// 
/// # Returns
/// * `Result<(), Error>` - Success or a validation error
pub fn validate_struct<T>(value: &T, resource_name: &str) -> Result<(), Error>
where
    T: Validate + Debug,
{
    debug!("Validating {}: {:?}", resource_name, value);
    
    value.validate().map_err(|err: ValidationErrors| {
        let error_message = format_validation_errors(&err);
        warn!(
            validation_errors = %error_message,
            "Validation failed for {}",
            resource_name
        );
        Error::validation_error(error_message)
    })
}

/// Format validation errors into a human-readable string
/// 
/// # Arguments
/// * `errors` - The validation errors
/// 
/// # Returns
/// * `String` - A formatted error message
fn format_validation_errors(errors: &ValidationErrors) -> String {
    let mut messages = Vec::new();
    
    for (field, field_errors) in errors.field_errors() {
        for error in field_errors {
            if let Some(message) = &error.message {
                messages.push(format!("{}: {}", field, message));
            } else {
                messages.push(format!("{}: invalid value", field));
            }
        }
    }
    
    if messages.is_empty() {
        "Validation failed".to_string()
    } else {
        messages.join(", ")
    }
}

/// Validate a non-empty string
/// 
/// # Arguments
/// * `value` - The string to validate
/// * `field_name` - The name of the field for error messages
/// 
/// # Returns
/// * `Result<(), Error>` - Success or a validation error
pub fn validate_non_empty_string(value: &str, field_name: &str) -> Result<(), Error> {
    if value.trim().is_empty() {
        warn!("{} is empty or contains only whitespace", field_name);
        return Err(Error::validation_error(format!("{} cannot be empty", field_name)));
    }
    
    Ok(())
}

/// Validate a password
/// 
/// # Arguments
/// * `password` - The password to validate
/// 
/// # Returns
/// * `Result<(), Error>` - Success or a validation error
pub fn validate_password(password: &str) -> Result<(), Error> {
    if password.len() < MIN_PASSWORD_LENGTH {
        warn!(
            "Password too short: {} characters (minimum: {})",
            password.len(),
            MIN_PASSWORD_LENGTH
        );
        return Err(Error::validation_error(format!(
            "Password must be at least {} characters long",
            MIN_PASSWORD_LENGTH
        )));
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
/// * `Result<(), Error>` - Success or a validation error
pub fn validate_email(email: &str) -> Result<(), Error> {
    // Basic email validation
    if !email.contains('@') || !email.contains('.') {
        warn!("Invalid email format: {}", email);
        return Err(Error::validation_error("Invalid email format"));
    }
    
    // More sophisticated email validation could be added here
    
    Ok(())
}

/// Validate a PGP public key
/// 
/// # Arguments
/// * `pgp_key` - The PGP key to validate
/// 
/// # Returns
/// * `Result<(), Error>` - Success or a validation error
pub fn validate_pgp_key(pgp_key: &str) -> Result<(), Error> {
    if !pgp_key.contains("-----BEGIN PGP PUBLIC KEY BLOCK-----") {
        warn!("Invalid PGP public key format");
        return Err(Error::validation_error("Invalid PGP public key format"));
    }
    
    // More sophisticated PGP key validation could be added here
    
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
/// * `Result<(), Error>` - Success or a validation error
pub fn validate_range<T>(value: T, min: T, max: T, field_name: &str) -> Result<(), Error>
where
    T: PartialOrd + Debug,
{
    if value < min || value > max {
        warn!(
            "{} out of range: {:?} (min: {:?}, max: {:?})",
            field_name, value, min, max
        );
        return Err(Error::validation_error(format!(
            "{} must be between {:?} and {:?}",
            field_name, min, max
        )));
    }
    
    Ok(())
}
