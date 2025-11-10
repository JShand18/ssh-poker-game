use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// SSH Server configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    /// Address to bind to
    pub address: String,
    
    /// Port to listen on
    pub port: u16,
    
    /// Path to server private key (optional)
    pub server_key_path: Option<String>,
    
    /// Maximum concurrent connections
    pub max_connections: usize,
    
    /// Connection timeout in seconds
    pub connection_timeout: u64,
    
    /// Authentication configuration
    pub auth: AuthConfig,
}

/// Authentication configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthConfig {
    /// Allow password authentication
    pub password_auth: bool,
    
    /// Allow public key authentication
    pub pubkey_auth: bool,
    
    /// Path to authorized_keys file (for pubkey auth)
    pub authorized_keys_path: Option<PathBuf>,
    
    /// Username/password pairs for simple password auth
    pub users: Vec<UserConfig>,
    
    /// Allow anonymous access (no authentication)
    pub allow_anonymous: bool,
}

/// User configuration for password authentication
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserConfig {
    pub username: String,
    pub password: String,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            address: "0.0.0.0".to_string(),
            port: 2222,
            server_key_path: None,
            max_connections: 100,
            connection_timeout: 600,
            auth: AuthConfig::default(),
        }
    }
}

impl Default for AuthConfig {
    fn default() -> Self {
        Self {
            password_auth: true,
            pubkey_auth: true,
            authorized_keys_path: None,
            users: vec![
                UserConfig {
                    username: "player".to_string(),
                    password: "poker123".to_string(),
                },
            ],
            allow_anonymous: true, // For development/testing
        }
    }
}

impl ServerConfig {
    /// Create a new server configuration with default values
    pub fn new() -> Self {
        Self::default()
    }
    
    /// Create a configuration for development/testing
    pub fn development() -> Self {
        Self {
            address: "127.0.0.1".to_string(),
            port: 2222,
            server_key_path: None,
            max_connections: 10,
            connection_timeout: 300,
            auth: AuthConfig::default(),
        }
    }
    
    /// Set the bind address
    pub fn with_address(mut self, address: impl Into<String>) -> Self {
        self.address = address.into();
        self
    }
    
    /// Set the port
    pub fn with_port(mut self, port: u16) -> Self {
        self.port = port;
        self
    }
    
    /// Set the server key path
    pub fn with_key_path(mut self, path: impl Into<String>) -> Self {
        self.server_key_path = Some(path.into());
        self
    }
} 