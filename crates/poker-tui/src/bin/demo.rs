//! Demo application to test the Charm.sh-style poker TUI
//! 
//! Run with: cargo run --bin demo -p charm-tui

use anyhow::Result;
use crossterm::{
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};
use std::io;

use poker_tui::PokerApp;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    env_logger::init();
    
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    
    // Create and run the poker app
    let mut app = PokerApp::new()?;
    let result = app.run(&mut terminal).await;
    
    // Restore terminal
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;
    
    // Handle any errors
    if let Err(e) = result {
        eprintln!("Application error: {}", e);
    }
    
    println!("Thanks for playing SSH Poker! 🎭");
    Ok(())
}