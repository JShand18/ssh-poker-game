//! Beautiful terminal UI components using Rust's best TUI libraries
//! 
//! Provides a casino-themed poker experience using ratatui, crossterm, and owo-colors.
//! This crate consolidates all TUI functionality with production-ready, beautifully styled components.

pub mod app;
pub mod components;  
pub mod events;
pub mod poker_table;
pub mod themes;
pub mod views;

// Core exports
pub use app::PokerApp;
pub use components::*;
pub use events::*;
pub use poker_table::PokerTableRenderer;
pub use themes::*;
pub use views::*;

// Re-exports for convenience
pub use crossterm;
pub use owo_colors;
pub use ratatui;