use std::path::Path;
use std::fs;
use log::{info, warn, error};
use crate::config::{AuthConfig, UserConfig};

/// Authentication result
#[derive(Debug, PartialEq)]
pub enum AuthResult {
    /// Authentication successful
    Success,
    /// Authentication failed
    Failed,
    /// Continue with next authentication method
    Continue,
}

/// Authentication handler
#[derive(Clone)]
pub struct AuthHandler {
    config: AuthConfig,
}

impl AuthHandler {
    pub fn new(config: AuthConfig) -> Self {
        Self { config }
    }

    /// Authenticate a user with username and password
    pub fn authenticate_password(&self, username: &str, password: &str) -> AuthResult {
        if !self.config.password_auth {
            info!("Password authentication disabled");
            return AuthResult::Continue;
        }

        info!("Attempting password authentication for user: {}", username);

        // Check configured users
        for user in &self.config.users {
            if user.username == username && user.password == password {
                info!("Password authentication successful for user: {}", username);
                return AuthResult::Success;
            }
        }

        warn!("Password authentication failed for user: {}", username);
        AuthResult::Failed
    }

    /// Authenticate a user with public key
    pub fn authenticate_pubkey(&self, username: &str, public_key: &[u8]) -> AuthResult {
        if !self.config.pubkey_auth {
            info!("Public key authentication disabled");
            return AuthResult::Continue;
        }

        info!("Attempting public key authentication for user: {}", username);

        // Check authorized keys file if configured
        if let Some(ref authorized_keys_path) = self.config.authorized_keys_path {
            if let Ok(authorized_keys) = self.load_authorized_keys(authorized_keys_path) {
                if self.check_public_key_in_authorized_keys(public_key, &authorized_keys) {
                    info!("Public key authentication successful for user: {}", username);
                    return AuthResult::Success;
                }
            }
        }

        warn!("Public key authentication failed for user: {}", username);
        AuthResult::Failed
    }

    /// Check if anonymous access is allowed
    pub fn allow_anonymous(&self) -> bool {
        self.config.allow_anonymous
    }

    /// Load authorized keys from file
    fn load_authorized_keys(&self, path: &Path) -> Result<String, std::io::Error> {
        fs::read_to_string(path)
    }

    /// Check if a public key exists in the authorized keys content
    fn check_public_key_in_authorized_keys(&self, _public_key: &[u8], _authorized_keys: &str) -> bool {
        // TODO: Implement proper public key matching
        // For now, just return false - this would need proper SSH key parsing
        warn!("Public key matching not yet implemented");
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::AuthConfig;

    #[test]
    fn test_password_authentication() {
        let config = AuthConfig {
            password_auth: true,
            pubkey_auth: false,
            authorized_keys_path: None,
            users: vec![
                UserConfig {
                    username: "test".to_string(),
                    password: "password123".to_string(),
                },
            ],
            allow_anonymous: false,
        };

        let auth = AuthHandler::new(config);

        // Test successful authentication
        assert_eq!(
            auth.authenticate_password("test", "password123"),
            AuthResult::Success
        );

        // Test failed authentication
        assert_eq!(
            auth.authenticate_password("test", "wrongpassword"),
            AuthResult::Failed
        );

        // Test non-existent user
        assert_eq!(
            auth.authenticate_password("nonexistent", "password123"),
            AuthResult::Failed
        );
    }

    #[test]
    fn test_disabled_password_auth() {
        let config = AuthConfig {
            password_auth: false,
            pubkey_auth: false,
            authorized_keys_path: None,
            users: vec![],
            allow_anonymous: false,
        };

        let auth = AuthHandler::new(config);
        assert_eq!(
            auth.authenticate_password("test", "password"),
            AuthResult::Continue
        );
    }

    #[test]
    fn test_anonymous_access() {
        let config = AuthConfig {
            password_auth: false,
            pubkey_auth: false,
            authorized_keys_path: None,
            users: vec![],
            allow_anonymous: true,
        };

        let auth = AuthHandler::new(config);
        assert!(auth.allow_anonymous());
    }
} 