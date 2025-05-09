//! Constants used throughout the application
//! This module contains constants that are used in multiple places in the application
//! to avoid hardcoding values and improve maintainability.

/// Authentication constants
pub mod auth {
    /// Default token expiration time in days
    pub const DEFAULT_TOKEN_EXPIRATION_DAYS: i64 = 30;
    
    /// Minimum password length
    pub const MIN_PASSWORD_LENGTH: usize = 8;
    
    /// Maximum failed login attempts before account lockout
    pub const MAX_FAILED_LOGIN_ATTEMPTS: u32 = 5;
    
    /// Account lockout duration in minutes
    pub const ACCOUNT_LOCKOUT_MINUTES: i64 = 30;
}

/// Database constants
pub mod database {
    /// Default database pool size
    pub const DEFAULT_POOL_SIZE: u32 = 5;
    
    /// Maximum database connection attempts
    pub const MAX_CONNECTION_ATTEMPTS: u32 = 3;
    
    /// Database connection retry delay in milliseconds
    pub const CONNECTION_RETRY_DELAY_MS: u64 = 1000;
}

/// Pagination constants
pub mod pagination {
    /// Default page size
    pub const DEFAULT_PAGE_SIZE: u32 = 20;
    
    /// Maximum page size
    pub const MAX_PAGE_SIZE: u32 = 100;
}

/// Security constants
pub mod security {
    /// Default Argon2 memory cost
    pub const DEFAULT_ARGON2_MEMORY_COST: u32 = 65536;
    
    /// Default Argon2 time cost
    pub const DEFAULT_ARGON2_TIME_COST: u32 = 3;
    
    /// Default Argon2 parallelism
    pub const DEFAULT_ARGON2_PARALLELISM: u32 = 1;
}

/// HTTP constants
pub mod http {
    /// Default HTTP timeout in seconds
    pub const DEFAULT_TIMEOUT_SECONDS: u64 = 30;
    
    /// Maximum request body size in bytes (10MB)
    pub const MAX_REQUEST_BODY_SIZE: u64 = 10 * 1024 * 1024;
    
    /// Request ID header name
    pub const REQUEST_ID_HEADER: &str = "x-request-id";
}

/// Tor constants
pub mod tor {
    /// Default Tor SOCKS port
    pub const DEFAULT_SOCKS_PORT: u16 = 9050;
    
    /// Default Tor control port
    pub const DEFAULT_CONTROL_PORT: u16 = 9051;
}

/// Cryptocurrency constants
pub mod crypto {
    /// Bitcoin confirmation threshold
    pub const BITCOIN_CONFIRMATION_THRESHOLD: u32 = 3;
    
    /// Monero confirmation threshold
    pub const MONERO_CONFIRMATION_THRESHOLD: u32 = 10;
}

/// PGP constants
pub mod pgp {
    /// PGP key server URL
    pub const KEY_SERVER_URL: &str = "hkps://keys.openpgp.org";
}

/// Rate limiting constants
pub mod rate_limit {
    /// Default rate limit window in seconds
    pub const DEFAULT_WINDOW_SECONDS: u64 = 60;
    
    /// Default rate limit max requests
    pub const DEFAULT_MAX_REQUESTS: u32 = 100;
    
    /// Login rate limit window in seconds
    pub const LOGIN_WINDOW_SECONDS: u64 = 300; // 5 minutes
    
    /// Login rate limit max requests
    pub const LOGIN_MAX_REQUESTS: u32 = 10;
}
