//! Beautiful terminal UI components using Rust's best TUI libraries
//! 
//! Provides a Charm.sh-inspired poker experience using ratatui, crossterm, and owo-colors.
//! This crate replaces the existing terminal-ui with production-ready, beautifully styled components.

pub mod app;
pub mod components;  
pub mod events;
pub mod themes;
pub mod views;

// Core exports
pub use app::PokerApp;
pub use components::*;
pub use events::*;
pub use themes::*;
pub use views::*;

// Re-exports for convenience
pub use crossterm;
pub use owo_colors;
pub use ratatui;