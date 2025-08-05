use anyhow::{Result, anyhow};
use argon2::{Argon2, PasswordHash, PasswordHasher, PasswordVerifier, password_hash::{rand_core::OsRng, SaltString}};
use database::{Database, models::User};
use russh_keys::key;
use std::collections::HashMap;
use log::{error, warn, info, debug};
use std::time::{Duration, Instant};

const MAX_AUTH_ATTEMPTS: u32 = 3;
const LOCKOUT_DURATION: Duration = Duration::from_secs(300); // 5 minutes

#[derive(Debug)]
struct AuthAttempt {
    count: u32,
    last_attempt: Instant,
    locked_until: Option<Instant>,
}

pub struct SecureAuthService {
    database: Database,
    user_cache: HashMap<String, User>,
    auth_attempts: HashMap<String, AuthAttempt>,
    argon2: Argon2<'static>,
}

impl SecureAuthService {
    pub fn new(database: Database) -> Self {
        Self {
            database,
            user_cache: HashMap::new(),
            auth_attempts: HashMap::new(),
            argon2: Argon2::default(),
        }
    }

    pub async fn authenticate_password(&mut self, username: &str, password: &str) -> Result<bool> {
        debug!("Password authentication attempt for user: {}", username);

        // Check rate limiting
        if self.is_locked_out(username) {
            warn!("Authentication blocked for user {} due to rate limiting", username);
            return Ok(false);
        }

        // Try cache first
        if let Some(user) = self.user_cache.get(username) {
            let is_valid = self.verify_password(password, &user.password_hash)?;
            self.handle_auth_result(username, is_valid);
            return Ok(is_valid);
        }

        // Fetch from database
        match self.database.get_user_by_username(username).await {
            Ok(Some(user)) => {
                let is_valid = self.verify_password(password, &user.password_hash)?;
                if is_valid {
                    info!("Successful password authentication for user: {}", username);
                    self.user_cache.insert(username.to_string(), user);
                    self.reset_auth_attempts(username);
                } else {
                    warn!("Failed password authentication for user: {}", username);
                    self.record_failed_attempt(username);
                }
                Ok(is_valid)
            }
            Ok(None) => {
                warn!("Authentication attempt for non-existent user: {}", username);
                self.record_failed_attempt(username);
                Ok(false)
            }
            Err(e) => {
                error!("Database error during authentication for user {}: {}", username, e);
                Ok(false)
            }
        }
    }

    pub async fn authenticate_publickey(&mut self, username: &str, public_key: &key::PublicKey) -> Result<bool> {
        debug!("Public key authentication attempt for user: {}", username);

        // Check rate limiting
        if self.is_locked_out(username) {
            warn!("Public key authentication blocked for user {} due to rate limiting", username);
            return Ok(false);
        }

        // Try cache first
        if let Some(user) = self.user_cache.get(username) {
            let is_valid = self.verify_public_key(public_key, &user.public_key)?;
            self.handle_auth_result(username, is_valid);
            return Ok(is_valid);
        }

        // Fetch from database
        match self.database.get_user_by_username(username).await {
            Ok(Some(user)) => {
                let is_valid = self.verify_public_key(public_key, &user.public_key)?;
                if is_valid {
                    info!("Successful public key authentication for user: {}", username);
                    self.user_cache.insert(username.to_string(), user);
                    self.reset_auth_attempts(username);
                } else {
                    warn!("Failed public key authentication for user: {}", username);
                    self.record_failed_attempt(username);
                }
                Ok(is_valid)
            }
            Ok(None) => {
                warn!("Public key authentication attempt for non-existent user: {}", username);
                self.record_failed_attempt(username);
                Ok(false)
            }
            Err(e) => {
                error!("Database error during public key authentication for user {}: {}", username, e);
                Ok(false)
            }
        }
    }

    fn verify_password(&self, provided_password: &str, stored_hash: &str) -> Result<bool> {
        match PasswordHash::new(stored_hash) {
            Ok(parsed_hash) => {
                Ok(self.argon2.verify_password(provided_password.as_bytes(), &parsed_hash).is_ok())
            }
            Err(e) => {
                error!("Failed to parse stored password hash: {}", e);
                Ok(false)
            }
        }
    }

    fn verify_public_key(&self, provided_key: &key::PublicKey, stored_key: &Option<String>) -> Result<bool> {
        if let Some(stored_key_str) = stored_key {
            match key::parse_public_key(stored_key_str.as_bytes()) {
                Ok(stored_key_parsed) => {
                    // Compare key types and data
                    let provided_bytes = provided_key.public_key_bytes();
                    let stored_bytes = stored_key_parsed.public_key_bytes();
                    Ok(provided_bytes == stored_bytes)
                }
                Err(e) => {
                    error!("Failed to parse stored public key: {}", e);
                    Ok(false)
                }
            }
        } else {
            debug!("No public key stored for user");
            Ok(false)
        }
    }

    pub async fn create_user(&mut self, username: &str, password: &str, email: &str) -> Result<uuid::Uuid> {
        info!("Creating new user: {}", username);
        let password_hash = self.hash_password(password)?;
        
        let user = database::models::NewUser {
            username: username.to_string(),
            email: email.to_string(),
            password_hash,
            public_key: None,
        };

        match self.database.create_user(user).await {
            Ok(user_id) => {
                info!("Successfully created user: {} with ID: {}", username, user_id);
                Ok(user_id)
            }
            Err(e) => {
                error!("Failed to create user {}: {}", username, e);
                Err(e)
            }
        }
    }

    fn hash_password(&self, password: &str) -> Result<String> {
        let salt = SaltString::generate(&mut OsRng);
        
        match self.argon2.hash_password(password.as_bytes(), &salt) {
            Ok(password_hash) => Ok(password_hash.to_string()),
            Err(e) => {
                error!("Failed to hash password: {}", e);
                Err(anyhow!("Password hashing failed: {}", e))
            }
        }
    }

    fn is_locked_out(&self, username: &str) -> bool {
        if let Some(attempt) = self.auth_attempts.get(username) {
            if let Some(locked_until) = attempt.locked_until {
                if Instant::now() < locked_until {
                    return true;
                }
            }
        }
        false
    }

    fn record_failed_attempt(&mut self, username: &str) {
        let now = Instant::now();
        let attempt = self.auth_attempts.entry(username.to_string()).or_insert(AuthAttempt {
            count: 0,
            last_attempt: now,
            locked_until: None,
        });

        attempt.count += 1;
        attempt.last_attempt = now;

        if attempt.count >= MAX_AUTH_ATTEMPTS {
            attempt.locked_until = Some(now + LOCKOUT_DURATION);
            warn!("User {} locked out after {} failed attempts", username, attempt.count);
        }
    }

    fn reset_auth_attempts(&mut self, username: &str) {
        self.auth_attempts.remove(username);
    }

    fn handle_auth_result(&mut self, username: &str, success: bool) {
        if success {
            self.reset_auth_attempts(username);
        } else {
            self.record_failed_attempt(username);
        }
    }

    pub async fn get_user(&mut self, username: &str) -> Result<Option<User>> {
        // Try cache first
        if let Some(user) = self.user_cache.get(username) {
            return Ok(Some(user.clone()));
        }

        // Fetch from database
        match self.database.get_user_by_username(username).await {
            Ok(Some(user)) => {
                self.user_cache.insert(username.to_string(), user.clone());
                Ok(Some(user))
            }
            Ok(None) => Ok(None),
            Err(e) => {
                error!("Database error fetching user {}: {}", username, e);
                Err(e)
            }
        }
    }

    pub fn clear_cache(&mut self) {
        self.user_cache.clear();
        info!("Authentication cache cleared");
    }

    pub fn cleanup_expired_lockouts(&mut self) {
        let now = Instant::now();
        self.auth_attempts.retain(|username, attempt| {
            if let Some(locked_until) = attempt.locked_until {
                if now >= locked_until {
                    debug!("Removing expired lockout for user: {}", username);
                    false
                } else {
                    true
                }
            } else {
                true
            }
        });
    }

    pub async fn change_password(&mut self, username: &str, old_password: &str, new_password: &str) -> Result<bool> {
        // First verify the old password
        if !self.authenticate_password(username, old_password).await? {
            warn!("Password change failed for user {}: invalid old password", username);
            return Ok(false);
        }

        // Hash the new password
        let new_hash = self.hash_password(new_password)?;

        // Update in database
        match self.database.update_user_password(username, &new_hash).await {
            Ok(_) => {
                info!("Password successfully changed for user: {}", username);
                // Remove from cache to force reload
                self.user_cache.remove(username);
                Ok(true)
            }
            Err(e) => {
                error!("Failed to update password for user {}: {}", username, e);
                Err(e)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_password_hashing() {
        let db = Database::new_in_memory().await.expect("Failed to create test database");
        let mut auth = SecureAuthService::new(db);

        let password = "test_password_123";
        let hash = auth.hash_password(password).unwrap();
        
        assert!(auth.verify_password(password, &hash).unwrap());
        assert!(!auth.verify_password("wrong_password", &hash).unwrap());
    }

    #[tokio::test]
    async fn test_rate_limiting() {
        let db = Database::new_in_memory().await.expect("Failed to create test database");
        let mut auth = SecureAuthService::new(db);

        let username = "test_user";
        
        // Create a user first
        auth.create_user(username, "correct_password", "test@example.com").await.unwrap();

        // Make multiple failed attempts
        for _ in 0..MAX_AUTH_ATTEMPTS {
            let result = auth.authenticate_password(username, "wrong_password").await.unwrap();
            assert!(!result);
        }

        // Should be locked out now
        assert!(auth.is_locked_out(username));

        // Even correct password should fail when locked out
        let result = auth.authenticate_password(username, "correct_password").await.unwrap();
        assert!(!result);
    }

    #[tokio::test]
    async fn test_user_creation_and_auth() {
        let db = Database::new_in_memory().await.expect("Failed to create test database");
        let mut auth = SecureAuthService::new(db);

        let username = "test_user";
        let password = "secure_password_123";
        let email = "test@example.com";

        // Create user
        let user_id = auth.create_user(username, password, email).await.unwrap();
        assert!(!user_id.is_nil());

        // Test successful authentication
        let result = auth.authenticate_password(username, password).await.unwrap();
        assert!(result);

        // Test failed authentication
        let result = auth.authenticate_password(username, "wrong_password").await.unwrap();
        assert!(!result);
    }
}