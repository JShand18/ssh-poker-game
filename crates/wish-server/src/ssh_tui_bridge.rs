//! Bridge between SSH server and Charm TUI
//! 
//! This module handles the conversion between SSH terminal events and 
//! the charm-tui event system, enabling the casino-style poker interface
//! to work over SSH connections.

use anyhow::Result;
use charm_tui::{AppEvent, InputEvent, PokerApp};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use log::{debug, info, warn};
use ratatui::{
    backend::CrosstermBackend,
    Terminal,
};
use std::io::{self, Write};
use tokio::sync::mpsc;

/// SSH TUI Bridge - connects SSH input/output to charm-tui
pub struct SshTuiBridge {
    /// The poker application with casino styling
    app: PokerApp,
    /// Channel to send rendered frames to SSH client
    output_sender: mpsc::UnboundedSender<Vec<u8>>,
    /// Terminal size
    terminal_size: (u16, u16),
}

impl SshTuiBridge {
    /// Create a new bridge between SSH and TUI
    pub fn new(output_sender: mpsc::UnboundedSender<Vec<u8>>) -> Result<Self> {
        let app = PokerApp::new()?;
        
        Ok(Self {
            app,
            output_sender,
            terminal_size: (80, 24), // Default terminal size
        })
    }
    
    /// Get the event sender to feed events from SSH
    pub fn event_sender(&self) -> mpsc::UnboundedSender<AppEvent> {
        self.app.event_sender()
    }
    
    /// Handle SSH input and convert to TUI events
    pub async fn handle_ssh_input(&mut self, data: &[u8]) -> Result<()> {
        // Parse SSH input data
        if let Ok(input_str) = String::from_utf8(data.to_vec()) {
            for ch in input_str.chars() {
                let event = self.char_to_event(ch);
                let sender = self.app.event_sender();
                sender.send(event)?;
            }
        }
        
        Ok(())
    }
    
    /// Convert a character to an AppEvent
    fn char_to_event(&self, ch: char) -> AppEvent {
        let key_event = match ch {
            '\x1b' => KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE),
            '\n' | '\r' => KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE),
            '\t' => KeyEvent::new(KeyCode::Tab, KeyModifiers::NONE),
            '\x7f' | '\x08' => KeyEvent::new(KeyCode::Backspace, KeyModifiers::NONE),
            
            // Arrow keys (ANSI escape sequences would need more complex parsing)
            'w' | 'W' => KeyEvent::new(KeyCode::Up, KeyModifiers::NONE),
            's' | 'S' => KeyEvent::new(KeyCode::Down, KeyModifiers::NONE),
            'a' | 'A' => KeyEvent::new(KeyCode::Left, KeyModifiers::NONE),
            'd' | 'D' => KeyEvent::new(KeyCode::Right, KeyModifiers::NONE),
            
            // Function keys shortcuts
            '1' => KeyEvent::new(KeyCode::F(1), KeyModifiers::NONE),
            '2' => KeyEvent::new(KeyCode::F(2), KeyModifiers::NONE),
            
            // Regular characters
            c => KeyEvent::new(KeyCode::Char(c.to_ascii_lowercase()), KeyModifiers::NONE),
        };
        
        AppEvent::Input(InputEvent::Key(key_event))
    }
    
    /// Update terminal size
    pub fn set_terminal_size(&mut self, width: u16, height: u16) {
        self.terminal_size = (width, height);
        let sender = self.app.event_sender();
        let _ = sender.send(AppEvent::Resize(width, height));
    }
    
    /// Run the TUI application with SSH backend
    pub async fn run(&mut self) -> Result<()> {
        // Create a virtual terminal backend
        let mut buffer = TerminalBuffer::new(self.terminal_size.0, self.terminal_size.1);
        
        loop {
            // Render the TUI to our buffer
            let frame_output = buffer.render_frame(&mut self.app)?;
            
            // Send the rendered frame to SSH client
            if !frame_output.is_empty() {
                self.output_sender.send(frame_output.into_bytes())?;
            }
            
            // Check if app should quit
            if self.app.should_quit() {
                break;
            }
            
            // Small delay for smooth rendering
            tokio::time::sleep(tokio::time::Duration::from_millis(16)).await;
        }
        
        Ok(())
    }
}

/// Virtual terminal buffer for rendering TUI without a real terminal
struct TerminalBuffer {
    width: u16,
    height: u16,
    buffer: Vec<Vec<char>>,
    styles: Vec<Vec<String>>,
}

impl TerminalBuffer {
    fn new(width: u16, height: u16) -> Self {
        let buffer = vec![vec![' '; width as usize]; height as usize];
        let styles = vec![vec![String::new(); width as usize]; height as usize];
        
        Self {
            width,
            height,
            buffer,
            styles,
        }
    }
    
    /// Render a frame from the poker app
    fn render_frame(&mut self, app: &mut PokerApp) -> Result<String> {
        // Create a string buffer to simulate terminal
        let mut output = String::new();
        
        // Clear screen
        output.push_str("\x1b[2J\x1b[H");
        
        // TODO: This is a simplified rendering. In production, we'd need to:
        // 1. Create a proper Backend implementation for SSH
        // 2. Use ratatui's rendering pipeline
        // 3. Convert the rendered widgets to ANSI escape sequences
        
        // For now, render a placeholder that shows the app is connected
        output.push_str("🎰 SSH Poker - Casino Edition 🎰\n");
        output.push_str("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n\n");
        output.push_str("Casino TUI is loading...\n\n");
        output.push_str("Controls:\n");
        output.push_str("  • F1 (or '1'): Toggle views\n");
        output.push_str("  • Arrow keys (or WASD): Navigate\n");
        output.push_str("  • Enter: Select\n");
        output.push_str("  • ESC: Back\n");
        output.push_str("  • q: Quit\n");
        
        Ok(output)
    }
}

/// SSH Backend for ratatui (to be implemented)
/// This would provide a proper Backend trait implementation for ratatui
/// to render directly to SSH terminals
pub struct SshBackend {
    output: mpsc::UnboundedSender<Vec<u8>>,
    width: u16,
    height: u16,
}

impl SshBackend {
    pub fn new(output: mpsc::UnboundedSender<Vec<u8>>, width: u16, height: u16) -> Self {
        Self { output, width, height }
    }
}

// TODO: Implement ratatui::backend::Backend for SshBackend
// This requires implementing all the Backend trait methods to render to SSH