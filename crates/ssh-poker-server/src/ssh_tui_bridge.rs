//! Bridge between SSH server and Poker TUI
//! 
//! This module handles the conversion between SSH terminal events and 
//! the poker-tui event system, enabling the casino-style poker interface
//! to work over SSH connections.

use anyhow::Result;
use poker_tui::{AppEvent, InputEvent, PokerApp};
use poker_tui::crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use poker_tui::ratatui::{
    backend::Backend,
    buffer::Cell,
    layout::Size,
    style::{Color, Modifier},
    Terminal,
};

use tokio::sync::mpsc;

/// SSH TUI Bridge - connects SSH input/output to the poker TUI
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
        log::info!("TUI bridge run starting with terminal size: {}x{}", self.terminal_size.0, self.terminal_size.1);
        
        // Create SSH backend and terminal
        let backend = SshBackend::new(self.output_sender.clone(), self.terminal_size.0, self.terminal_size.1);
        let mut terminal = Terminal::new(backend)?;
        
        // Clear and prepare terminal
        log::debug!("Clearing terminal");
        terminal.clear()?;
        
        let mut frame_count = 0;
        loop {
            frame_count += 1;
            if frame_count % 60 == 1 { // Log every 60 frames (~1 second)
                log::debug!("Drawing frame {}", frame_count);
            }
            
            // Process any pending events from SSH input
            if let Some(event) = self.app.try_recv_event().await {
                log::debug!("Processing event: {:?}", event);
                if let Err(e) = self.app.handle_event(event).await {
                    log::error!("Failed to handle event: {}", e);
                }
            }
            
            // Draw the app
            terminal.draw(|frame| {
                if frame_count % 60 == 1 {
                    log::debug!("Terminal draw callback called, frame area: {:?}", frame.area());
                }
                if let Err(e) = self.app.draw(frame, frame.area()) {
                    log::error!("Failed to draw app: {}", e);
                }
            })?;
            
            // Check if app should quit
            if self.app.should_quit() {
                log::info!("App requested quit, exiting TUI loop");
                break;
            }
            
            // Small delay to prevent busy waiting
            tokio::time::sleep(tokio::time::Duration::from_millis(16)).await;
        }
        
        log::info!("TUI bridge run completed");
        Ok(())
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

/// Internal buffer to track terminal state
pub struct SshBuffer {
    _cells: Vec<Vec<Cell>>,
    _width: u16,
    _height: u16,
}

impl SshBuffer {
    fn _new(width: u16, height: u16) -> Self {
        let cells = (0..height)
            .map(|_| (0..width).map(|_| Cell::default()).collect())
            .collect();
        Self { _cells: cells, _width: width, _height: height }
    }
    
    fn _get_mut(&mut self, x: u16, y: u16) -> Option<&mut Cell> {
        self._cells
            .get_mut(y as usize)
            .and_then(|row| row.get_mut(x as usize))
    }
    
    fn _diff<'a>(&self, other: &'a SshBuffer) -> Vec<(u16, u16, &'a Cell)> {
        let mut changes = Vec::new();
        for y in 0..self._height {
            for x in 0..self._width {
                let idx_y = y as usize;
                let idx_x = x as usize;
                if let (Some(old_row), Some(new_row)) = (self._cells.get(idx_y), other._cells.get(idx_y)) {
                    if let (Some(old_cell), Some(new_cell)) = (old_row.get(idx_x), new_row.get(idx_x)) {
                        if old_cell != new_cell {
                            changes.push((x, y, new_cell));
                        }
                    }
                }
            }
        }
        changes
    }
}

impl Backend for SshBackend {
    fn draw<'a, I>(&mut self, content: I) -> std::io::Result<()>
    where
        I: Iterator<Item = (u16, u16, &'a Cell)>,
    {
        let mut output = String::new();
        let mut last_y = 0;
        let mut last_x = 0;
        let mut last_fg = Color::Reset;
        let mut last_bg = Color::Reset;
        let mut last_modifier = Modifier::empty();
        
        for (x, y, cell) in content {
            // Move cursor if needed
            if y != last_y || x != last_x + 1 {
                output.push_str(&format!("\x1b[{};{}H", y + 1, x + 1));
            }
            last_y = y;
            last_x = x;
            
            // Apply style changes
            if cell.fg != last_fg || cell.bg != last_bg || cell.modifier != last_modifier {
                // Reset style
                output.push_str("\x1b[0m");
                
                // Apply foreground color
                if cell.fg != Color::Reset {
                    output.push_str(&self.color_to_ansi(cell.fg, true));
                }
                last_fg = cell.fg;
                
                // Apply background color
                if cell.bg != Color::Reset {
                    output.push_str(&self.color_to_ansi(cell.bg, false));
                }
                last_bg = cell.bg;
                
                // Apply modifiers
                if cell.modifier.contains(Modifier::BOLD) {
                    output.push_str("\x1b[1m");
                }
                if cell.modifier.contains(Modifier::ITALIC) {
                    output.push_str("\x1b[3m");
                }
                if cell.modifier.contains(Modifier::UNDERLINED) {
                    output.push_str("\x1b[4m");
                }
                last_modifier = cell.modifier;
            }
            
            // Write the character
            output.push_str(cell.symbol());
        }
        
        // Send the output
        let output_bytes = output.into_bytes();
        log::debug!("Sending {} bytes to SSH client", output_bytes.len());
        self.output
            .send(output_bytes)
            .map_err(|_| std::io::Error::new(std::io::ErrorKind::BrokenPipe, "SSH output channel closed"))?;
        
        Ok(())
    }
    
    fn hide_cursor(&mut self) -> std::io::Result<()> {
        self.output
            .send(b"\x1b[?25l".to_vec())
            .map_err(|_| std::io::Error::new(std::io::ErrorKind::BrokenPipe, "SSH output channel closed"))?;
        Ok(())
    }
    
    fn show_cursor(&mut self) -> std::io::Result<()> {
        self.output
            .send(b"\x1b[?25h".to_vec())
            .map_err(|_| std::io::Error::new(std::io::ErrorKind::BrokenPipe, "SSH output channel closed"))?;
        Ok(())
    }
    
    fn get_cursor(&mut self) -> std::io::Result<(u16, u16)> {
        // SSH doesn't easily support getting cursor position
        // Return a default position
        Ok((0, 0))
    }
    

    
    fn clear(&mut self) -> std::io::Result<()> {
        log::debug!("Clearing SSH terminal");
        self.output
            .send(b"\x1b[2J\x1b[H".to_vec())
            .map_err(|_| std::io::Error::new(std::io::ErrorKind::BrokenPipe, "SSH output channel closed"))?;
        Ok(())
    }
    
    fn size(&self) -> std::io::Result<Size> {
        Ok(Size::new(self.width, self.height))
    }
    
    fn flush(&mut self) -> std::io::Result<()> {
        // For SSH, we send data immediately, so flush is a no-op
        Ok(())
    }
    
    fn get_cursor_position(&mut self) -> std::io::Result<poker_tui::ratatui::layout::Position> {
        // SSH doesn't easily support getting cursor position
        // Return a default position
        Ok(poker_tui::ratatui::layout::Position::new(0, 0))
    }
    
    fn set_cursor_position<P>(&mut self, position: P) -> std::io::Result<()>
    where
        P: Into<poker_tui::ratatui::layout::Position>,
    {
        let pos = position.into();
        let cmd = format!("\x1b[{};{}H", pos.y + 1, pos.x + 1);
        self.output
            .send(cmd.into_bytes())
            .map_err(|_| std::io::Error::new(std::io::ErrorKind::BrokenPipe, "SSH output channel closed"))
    }
    
    fn window_size(&mut self) -> std::io::Result<poker_tui::ratatui::backend::WindowSize> {
        Ok(poker_tui::ratatui::backend::WindowSize {
            columns_rows: self.size()?,
            pixels: Size::new(0, 0),
        })
    }
}

impl SshBackend {
    /// Convert ratatui Color to ANSI escape sequence
    fn color_to_ansi(&self, color: Color, is_foreground: bool) -> String {
        let base = if is_foreground { 30 } else { 40 };
        match color {
            Color::Reset => "\x1b[39m".to_string(),
            Color::Black => format!("\x1b[{}m", base),
            Color::Red => format!("\x1b[{}m", base + 1),
            Color::Green => format!("\x1b[{}m", base + 2),
            Color::Yellow => format!("\x1b[{}m", base + 3),
            Color::Blue => format!("\x1b[{}m", base + 4),
            Color::Magenta => format!("\x1b[{}m", base + 5),
            Color::Cyan => format!("\x1b[{}m", base + 6),
            Color::Gray => format!("\x1b[{}m", base + 7),
            Color::DarkGray => format!("\x1b[{}m", base + 60),
            Color::LightRed => format!("\x1b[{}m", base + 61),
            Color::LightGreen => format!("\x1b[{}m", base + 62),
            Color::LightYellow => format!("\x1b[{}m", base + 63),
            Color::LightBlue => format!("\x1b[{}m", base + 64),
            Color::LightMagenta => format!("\x1b[{}m", base + 65),
            Color::LightCyan => format!("\x1b[{}m", base + 66),
            Color::White => format!("\x1b[{}m", base + 67),
            Color::Rgb(r, g, b) => {
                if is_foreground {
                    format!("\x1b[38;2;{};{};{}m", r, g, b)
                } else {
                    format!("\x1b[48;2;{};{};{}m", r, g, b)
                }
            }
            Color::Indexed(idx) => {
                if is_foreground {
                    format!("\x1b[38;5;{}m", idx)
                } else {
                    format!("\x1b[48;5;{}m", idx)
                }
            }
        }
    }
}