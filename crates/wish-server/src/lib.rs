pub mod ssh_tui_bridge;

use anyhow::Result;
use data_store::Database;

// Temporary facade to reuse the working russh server until full Charm/Wish
// session plumbing is completed. This preserves the crate API expected by the
// launcher while ensuring the game is stable and playable now.
// TODO: Replace with integrated charm-tui version
pub async fn run_server(database: Database, bind_address: &str, port: u16) -> Result<()> {
    ssh_server::run_server(database, bind_address, port).await
}

// Re-export authentication service for convenience
pub use ssh_server::SecureAuthService;

