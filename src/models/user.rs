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

impl User {
    pub fn new<A, B>(username: A, password_hash: B, pgp_public_key: Option<String>, role: &str) -> NewUser
    where
        A: Into<String>,
        B: Into<String>,
    {
        NewUser {
            username: username.into(),
            password_hash: password_hash.into(),
            pgp_public_key,
            role: role.to_string(),
        }
    }

    pub fn is_password_match(&self, password: &str) -> bool {
        verify_password(password, &self.password_hash, &Argon2::default()).unwrap_or(false)
    }

    pub fn is_admin(&self) -> bool {
        self.role == "admin"
    }

    pub fn is_moderator(&self) -> bool {
        self.role == "moderator" || self.role == "admin"
    }

    pub fn is_vendor(&self) -> bool {
        self.role == "vendor"
    }

    pub fn is_buyer(&self) -> bool {
        self.role == "buyer"
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
        Self {
            memory_cost: 65536,
            time_cost: 3,
            parallelism: 1,
        }
    }
}

pub async fn hash_password<P>(password: P, config: &Argon2Config) -> Result<String, Error>
where
    P: AsRef<str> + Send + 'static,
{
    let config = config.clone();
    task::spawn_blocking(move || {
        let salt = SaltString::generate(&mut OsRng);
        let argon2 = Argon2::new(
            argon2::Algorithm::Argon2id,
            argon2::Version::V0x13,
            argon2::Params::new(config.memory_cost, config.time_cost, config.parallelism, None)
                .map_err(|e| Error::HashPassword(e.to_string()))?,
        );
        argon2
            .hash_password(password.as_ref().as_bytes(), &salt)
            .map(|hash| hash.to_string())
            .map_err(|e| Error::HashPassword(e.to_string()))
    })
    .await
    .map_err(|e| Error::RunSyncTask(e))?
}

fn verify_password(password: &str, hash: &str, argon2: &Argon2) -> Result<bool, Error> {
    let parsed_hash = PasswordHash::new(hash).map_err(|e| Error::HashPassword(e.to_string()))?;
    Ok(argon2
        .verify_password(password.as_bytes(), &parsed_hash)
        .is_ok())
}
