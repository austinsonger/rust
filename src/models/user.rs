use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use chrono::{DateTime, Utc};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use tokio::task;
use validator::Validate;

use crate::errors::Error;
use crate::schema::users;

#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Identifiable, Validate)]
#[diesel(table_name = users)]
pub struct User {
    pub id: i32,
    #[validate(length(min = 1))]
    pub username: String,
    pub password_hash: String,
    pub pgp_public_key: Option<String>,
    pub role: String,
    pub reputation: Option<f64>,
    pub is_locked: Option<bool>,
    pub locked_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Insertable)]
#[diesel(table_name = users)]
pub struct NewUser {
    pub username: String,
    pub password_hash: String,
    pub pgp_public_key: Option<String>,
    pub role: String,
}

#[derive(Debug, AsChangeset)]
#[diesel(table_name = users)]
pub struct UpdateUser {
    pub username: Option<String>,
    pub password_hash: Option<String>,
    pub pgp_public_key: Option<String>,
    pub role: Option<String>,
    pub reputation: Option<f64>,
    pub is_locked: Option<bool>,
    pub locked_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
}

/// Valid user roles
pub mod roles {
    pub const ADMIN: &str = "admin";
    pub const MODERATOR: &str = "moderator";
    pub const VENDOR: &str = "vendor";
    pub const BUYER: &str = "buyer";

    /// Check if a role is valid
    pub fn is_valid(role: &str) -> bool {
        matches!(role, ADMIN | MODERATOR | VENDOR | BUYER)
    }
}

impl User {
    /// Create a new user
    ///
    /// # Arguments
    /// * `username` - The username for the new user
    /// * `password_hash` - The hashed password for the new user
    /// * `pgp_public_key` - Optional PGP public key for the user
    /// * `role` - The role for the new user
    ///
    /// # Returns
    /// * `Result<NewUser, Error>` - The new user or an error
    pub fn new<A, B>(
        username: A,
        password_hash: B,
        pgp_public_key: Option<String>,
        role: &str
    ) -> Result<NewUser, Error>
    where
        A: Into<String>,
        B: Into<String>,
    {
        use tracing::{debug, warn};

        let username = username.into();
        let password_hash = password_hash.into();

        // Validate username
        if username.is_empty() {
            return Err(Error::validation_error("Username cannot be empty"));
        }

        // Validate password hash
        if password_hash.is_empty() {
            return Err(Error::validation_error("Password hash cannot be empty"));
        }

        // Validate role
        if !roles::is_valid(role) {
            warn!(role = %role, "Invalid role specified");
            return Err(Error::validation_error(format!("Invalid role: {}", role)));
        }

        // Validate PGP key if provided
        if let Some(ref key) = pgp_public_key {
            if key.is_empty() {
                debug!("Empty PGP key provided, setting to None");
                return Ok(NewUser {
                    username,
                    password_hash,
                    pgp_public_key: None,
                    role: role.to_string(),
                });
            }

            // Basic PGP key validation - should start with "-----BEGIN PGP PUBLIC KEY BLOCK-----"
            if !key.contains("-----BEGIN PGP PUBLIC KEY BLOCK-----") {
                return Err(Error::validation_error("Invalid PGP public key format"));
            }
        }

        Ok(NewUser {
            username,
            password_hash,
            pgp_public_key,
            role: role.to_string(),
        })
    }

    /// Check if a password matches the user's password hash
    ///
    /// # Arguments
    /// * `password` - The password to check
    ///
    /// # Returns
    /// * `bool` - Whether the password matches
    pub fn is_password_match(&self, password: &str) -> bool {
        use tracing::{debug, error};

        // Validate password
        if password.is_empty() {
            debug!("Empty password provided for password match check");
            return false;
        }

        match verify_password(password, &self.password_hash, &Argon2::default()) {
            Ok(result) => result,
            Err(err) => {
                error!(
                    error = %err,
                    user_id = self.id,
                    "Error verifying password"
                );
                false
            }
        }
    }

    /// Check if the user is an admin
    pub fn is_admin(&self) -> bool {
        self.role == roles::ADMIN
    }

    /// Check if the user is a moderator or admin
    pub fn is_moderator(&self) -> bool {
        self.role == roles::MODERATOR || self.role == roles::ADMIN
    }

    /// Check if the user is a vendor
    pub fn is_vendor(&self) -> bool {
        self.role == roles::VENDOR
    }

    /// Check if the user is a buyer
    pub fn is_buyer(&self) -> bool {
        self.role == roles::BUYER
    }

    /// Check if the user account is locked
    pub fn is_account_locked(&self) -> bool {
        self.is_locked.unwrap_or(false)
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PublicUser {
    pub id: i32,
    pub username: String,
    pub pgp_public_key: Option<String>,
    pub role: String,
    pub reputation: Option<f64>,
    pub created_at: DateTime<Utc>,
}

impl From<User> for PublicUser {
    fn from(user: User) -> Self {
        Self {
            id: user.id,
            username: user.username,
            pgp_public_key: user.pgp_public_key,
            role: user.role,
            reputation: user.reputation,
            created_at: user.created_at,
        }
    }
}

pub struct Argon2Config {
    pub memory_cost: u32,
    pub time_cost: u32,
    pub parallelism: u32,
}

impl Default for Argon2Config {
    fn default() -> Self {
        use crate::constants::security::{
            DEFAULT_ARGON2_MEMORY_COST,
            DEFAULT_ARGON2_TIME_COST,
            DEFAULT_ARGON2_PARALLELISM,
        };

        Self {
            memory_cost: DEFAULT_ARGON2_MEMORY_COST,
            time_cost: DEFAULT_ARGON2_TIME_COST,
            parallelism: DEFAULT_ARGON2_PARALLELISM,
        }
    }
}

/// Hash a password using Argon2id
///
/// # Arguments
/// * `password` - The password to hash
/// * `config` - The Argon2 configuration to use
///
/// # Returns
/// * `Result<String, Error>` - The hashed password or an error
pub async fn hash_password<P>(password: P, config: Option<&Argon2Config>) -> Result<String, Error>
where
    P: AsRef<str> + Send + 'static,
{
    use tracing::{debug, error};

    // Use the provided config or the default
    let config = config.cloned().unwrap_or_default();

    // Validate password length
    if password.as_ref().len() < crate::constants::auth::MIN_PASSWORD_LENGTH {
        return Err(Error::validation_error(format!(
            "Password must be at least {} characters long",
            crate::constants::auth::MIN_PASSWORD_LENGTH
        )));
    }

    debug!("Hashing password with Argon2id");

    // Spawn a blocking task to hash the password
    task::spawn_blocking(move || {
        let salt = SaltString::generate(&mut OsRng);

        // Create the Argon2 instance with the specified parameters
        let argon2 = Argon2::new(
            argon2::Algorithm::Argon2id,
            argon2::Version::V0x13,
            argon2::Params::new(config.memory_cost, config.time_cost, config.parallelism, None)
                .map_err(|e| {
                    error!(error = %e, "Failed to create Argon2 parameters");
                    Error::HashPassword(e.to_string())
                })?,
        );

        // Hash the password
        argon2
            .hash_password(password.as_ref().as_bytes(), &salt)
            .map(|hash| hash.to_string())
            .map_err(|e| {
                error!(error = %e, "Failed to hash password");
                Error::HashPassword(e.to_string())
            })
    })
    .await
    .map_err(|e| {
        error!(error = %e, "Failed to spawn blocking task for password hashing");
        Error::RunSyncTask(e)
    })?
}

/// Verify a password against a hash
///
/// # Arguments
/// * `password` - The password to verify
/// * `hash` - The hash to verify against
/// * `argon2` - The Argon2 instance to use
///
/// # Returns
/// * `Result<bool, Error>` - Whether the password matches the hash or an error
fn verify_password(password: &str, hash: &str, argon2: &Argon2) -> Result<bool, Error> {
    use tracing::{debug, error};

    debug!("Verifying password");

    // Parse the hash
    let parsed_hash = PasswordHash::new(hash).map_err(|e| {
        error!(error = %e, "Failed to parse password hash");
        Error::HashPassword(e.to_string())
    })?;

    // Verify the password
    Ok(argon2
        .verify_password(password.as_bytes(), &parsed_hash)
        .is_ok())
}
