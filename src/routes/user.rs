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

async fn create_user(Json(body): Json<CreateBody>) -> Result<CustomResponse<PublicUser>, Error> {
    // Validate password length
    if body.password.len() < 8 {
        return Err(Error::bad_request());
    }

    // Hash the password
    let password_hash = user::hash_password(body.password).await?;

    // Create the new user
    let new_user = User::new(body.username, password_hash, body.pgp_public_key, "buyer");

    // Insert into database
    let mut conn = get_connection()?;
    let user_result = diesel::insert_into(users)
        .values(&new_user)
        .get_result::<User>(&mut conn)
        .map_err(|_| Error::DatabaseError("Failed to create user".to_string()))?;

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

async fn authenticate_user(
    Json(body): Json<AuthorizeBody>,
) -> Result<Json<AuthenticateResponse>, Error> {
    let username_val = &body.username;
    let password_val = &body.password;

    if username_val.is_empty() {
        debug!("Missing username, returning 400 status code");
        return Err(Error::bad_request());
    }

    if password_val.is_empty() {
        debug!("Missing password, returning 400 status code");
        return Err(Error::bad_request());
    }

    // Find user by username
    let mut conn = get_connection()?;
    let user_result = users
        .filter(username.eq(username_val))
        .first::<User>(&mut conn)
        .map_err(|_| Error::Authenticate(AuthenticateError::WrongCredentials))?;

    // Verify password
    if !user_result.is_password_match(password_val) {
        debug!("User password is incorrect, returning 401 status code");
        return Err(Error::Authenticate(AuthenticateError::WrongCredentials));
    }

    // Check if user is locked
    if user_result.is_locked.unwrap_or(false) {
        debug!("User is locked, returning 401");
        return Err(Error::Authenticate(AuthenticateError::Locked));
    }

    // Create JWT token
    let secret = SETTINGS.auth.secret.as_str();
    let token = token::create(user_result.clone(), secret)
        .map_err(|_| Error::Authenticate(AuthenticateError::TokenCreation))?;

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
