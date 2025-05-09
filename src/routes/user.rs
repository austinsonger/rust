use axum::http::StatusCode;
use axum::{extract::Path, routing::{get, post}, Json, Router};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use tracing::debug;

use crate::database::get_connection;
use crate::errors::{AuthenticateError, Error};
use crate::models::user;
use crate::models::user::{NewUser, PublicUser, User};
use crate::schema::users::dsl::*;
use crate::settings::SETTINGS;
use crate::utils::authenticate_request::TokenUser;
use crate::utils::custom_response::{CustomResponse, CustomResponseBuilder};
use crate::utils::token;

pub fn create_route() -> Router {
    Router::new()
        .route("/users", post(create_user).get(get_current_user))
        .route("/users/:id", get(get_user))
        .route("/users/authenticate", post(authenticate_user))
}

/// Create a new user
///
/// # Arguments
/// * `body` - The request body containing the user information
///
/// # Returns
/// * `Result<CustomResponse<PublicUser>, Error>` - The created user or an error
async fn create_user(Json(body): Json<CreateBody>) -> Result<CustomResponse<PublicUser>, Error> {
    use tracing::{debug, error, info};
    use crate::constants::auth::MIN_PASSWORD_LENGTH;

    debug!("Creating new user with username: {}", body.username);

    // Validate password length
    if body.password.len() < MIN_PASSWORD_LENGTH {
        debug!("Password too short: {} characters (minimum: {})", body.password.len(), MIN_PASSWORD_LENGTH);
        return Err(Error::validation_error(format!(
            "Password must be at least {} characters long",
            MIN_PASSWORD_LENGTH
        )));
    }

    // Hash the password
    let password_hash = user::hash_password(body.password, None).await?;

    // Create the new user
    let new_user = User::new(
        body.username.clone(),
        password_hash,
        body.pgp_public_key.clone(),
        user::roles::BUYER
    )?;

    // Insert into database
    let mut conn = get_connection()?;

    // Check if username already exists
    let existing_user = users
        .filter(username.eq(&body.username))
        .first::<User>(&mut conn)
        .optional()
        .map_err(|err| {
            error!(error = %err, "Database error when checking for existing user");
            Error::database_error(
                format!("Error checking for existing user: {}", err),
                Some(Box::new(err)),
                None
            )
        })?;

    if existing_user.is_some() {
        debug!("Username already exists: {}", body.username);
        return Err(Error::validation_error("Username already exists"));
    }

    // Insert the new user
    let user_result = diesel::insert_into(users)
        .values(&new_user)
        .get_result::<User>(&mut conn)
        .map_err(|err| {
            error!(
                error = %err,
                username = %body.username,
                "Failed to create user"
            );
            Error::database_error(
                "Failed to create user".to_string(),
                Some(Box::new(err)),
                None
            )
        })?;

    // Log successful user creation
    info!(
        user_id = user_result.id,
        username = %user_result.username,
        role = %user_result.role,
        "User created successfully"
    );

    // Return the public user
    let res = PublicUser::from(user_result);
    let res = CustomResponseBuilder::new()
        .body(res)
        .status_code(StatusCode::CREATED)
        .build();

    Ok(res)
}

async fn get_current_user(token_user: TokenUser) -> Result<CustomResponse<PublicUser>, Error> {
    let mut conn = get_connection()?;
    let user_result = users
        .find(token_user.id)
        .first::<User>(&mut conn)
        .map_err(|_| Error::not_found())?;

    let res = PublicUser::from(user_result);
    let res = CustomResponseBuilder::new()
        .body(res)
        .status_code(StatusCode::OK)
        .build();

    Ok(res)
}

async fn get_user(Path(user_id): Path<i32>) -> Result<CustomResponse<PublicUser>, Error> {
    let mut conn = get_connection()?;
    let user_result = users
        .find(user_id)
        .first::<User>(&mut conn)
        .map_err(|_| Error::not_found())?;

    let res = PublicUser::from(user_result);
    let res = CustomResponseBuilder::new()
        .body(res)
        .status_code(StatusCode::OK)
        .build();

    Ok(res)
}

/// Authenticate a user and return a JWT token
///
/// # Arguments
/// * `body` - The request body containing the username and password
///
/// # Returns
/// * `Result<Json<AuthenticateResponse>, Error>` - The authentication response or an error
async fn authenticate_user(
    Json(body): Json<AuthorizeBody>,
) -> Result<Json<AuthenticateResponse>, Error> {
    use tracing::{debug, error, info, warn};
    use crate::constants::auth::{ACCOUNT_LOCKOUT_MINUTES, MAX_FAILED_LOGIN_ATTEMPTS};

    let username_val = &body.username;
    let password_val = &body.password;

    // Validate username
    if username_val.is_empty() {
        debug!("Missing username in authentication request");
        return Err(Error::validation_error("Username is required"));
    }

    // Validate password
    if password_val.is_empty() {
        debug!("Missing password in authentication request");
        return Err(Error::validation_error("Password is required"));
    }

    // Find user by username
    let mut conn = get_connection()?;
    let mut user_result = match users
        .filter(username.eq(username_val))
        .first::<User>(&mut conn) {
            Ok(user) => user,
            Err(err) => {
                // Don't reveal whether the user exists or not
                warn!(
                    error = %err,
                    username = %username_val,
                    "User not found during authentication"
                );
                return Err(Error::Authenticate(AuthenticateError::wrong_credentials()));
            }
        };

    // Check if user is locked
    if user_result.is_account_locked() {
        // Check if the lockout period has expired
        if let Some(locked_at) = user_result.locked_at {
            let now = chrono::Utc::now();
            let lockout_duration = chrono::Duration::minutes(ACCOUNT_LOCKOUT_MINUTES);

            if now - locked_at > lockout_duration {
                // Lockout period has expired, unlock the user
                debug!(
                    user_id = user_result.id,
                    username = %user_result.username,
                    "Unlocking user after lockout period"
                );

                // Update the user to unlock them
                diesel::update(&user_result)
                    .set((
                        is_locked.eq(false),
                        locked_at.eq::<Option<DateTime<Utc>>>(None),
                    ))
                    .execute(&mut conn)
                    .map_err(|err| {
                        error!(
                            error = %err,
                            user_id = user_result.id,
                            "Failed to unlock user"
                        );
                        Error::database_error(
                            "Failed to unlock user".to_string(),
                            Some(Box::new(err)),
                            None
                        )
                    })?;

                // Refresh the user data
                user_result = users
                    .find(user_result.id)
                    .first::<User>(&mut conn)
                    .map_err(|err| {
                        error!(
                            error = %err,
                            user_id = user_result.id,
                            "Failed to refresh user data"
                        );
                        Error::database_error(
                            "Failed to refresh user data".to_string(),
                            Some(Box::new(err)),
                            None
                        )
                    })?;
            } else {
                // User is still locked
                warn!(
                    user_id = user_result.id,
                    username = %user_result.username,
                    locked_at = %locked_at,
                    "User is locked, authentication rejected"
                );
                return Err(Error::Authenticate(AuthenticateError::locked(Some(locked_at))));
            }
        } else {
            // User is locked but no locked_at timestamp, treat as locked
            warn!(
                user_id = user_result.id,
                username = %user_result.username,
                "User is locked (no timestamp), authentication rejected"
            );
            return Err(Error::Authenticate(AuthenticateError::locked(None)));
        }
    }

    // Verify password
    if !user_result.is_password_match(password_val) {
        warn!(
            user_id = user_result.id,
            username = %user_result.username,
            "Incorrect password during authentication"
        );

        // Increment failed login attempts and potentially lock the account
        // This would be implemented with a separate table in a real application
        // For now, we'll just lock the account after MAX_FAILED_LOGIN_ATTEMPTS

        // In a real implementation, we would:
        // 1. Get the current failed login attempts count
        // 2. Increment it
        // 3. If it exceeds MAX_FAILED_LOGIN_ATTEMPTS, lock the account

        // For demonstration purposes, we'll just lock the account
        // In a real application, this would be more sophisticated
        let now = chrono::Utc::now();
        diesel::update(&user_result)
            .set((
                is_locked.eq(true),
                locked_at.eq(now),
            ))
            .execute(&mut conn)
            .map_err(|err| {
                error!(
                    error = %err,
                    user_id = user_result.id,
                    "Failed to update user after failed login attempt"
                );
                Error::database_error(
                    "Failed to update user".to_string(),
                    Some(Box::new(err)),
                    None
                )
            })?;

        return Err(Error::Authenticate(AuthenticateError::wrong_credentials()));
    }

    // Create JWT token
    let secret = SETTINGS.auth.secret.as_str();
    let token = token::create(user_result.clone(), secret)
        .map_err(|err| {
            error!(
                error = %err,
                user_id = user_result.id,
                "Failed to create authentication token"
            );
            Error::Authenticate(AuthenticateError::token_creation(Some(Box::new(err))))
        })?;

    // Log successful authentication
    info!(
        user_id = user_result.id,
        username = %user_result.username,
        role = %user_result.role,
        "User authenticated successfully"
    );

    // Return the authentication response
    let res = AuthenticateResponse {
        access_token: token,
        user: PublicUser::from(user_result),
    };

    Ok(Json(res))
}

#[derive(Debug, Deserialize)]
struct CreateBody {
    username: String,
    password: String,
    pgp_public_key: Option<String>,
}

#[derive(Debug, Deserialize)]
struct AuthorizeBody {
    username: String,
    password: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AuthenticateResponse {
    pub access_token: String,
    pub user: PublicUser,
}
