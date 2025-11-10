//! Event handling for the TUI application
//! 
//! Provides a clean event system inspired by Bubble Tea's message passing

use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers, MouseEvent};
use std::time::Duration;
use tokio::sync::mpsc;

/// Application events that can occur
#[derive(Debug, Clone)]
pub enum AppEvent {
    /// Terminal/keyboard events
    Input(InputEvent),
    /// Game state updates
    GameUpdate(GameEvent),
    /// Network events  
    Network(NetworkEvent),
    /// Timer/periodic events
    Tick,
    /// Application should quit
    Quit,
    /// Resize terminal
    Resize(u16, u16),
}

/// Input events from the user
#[derive(Debug, Clone)]
pub enum InputEvent {
    /// Key press
    Key(KeyEvent),
    /// Mouse event
    Mouse(MouseEvent),
    /// Paste event
    Paste(String),
}

/// Game-specific events
#[derive(Debug, Clone)]
pub enum GameEvent {
    /// Player joined the game
    PlayerJoined { player_id: String, name: String },
    /// Player left the game
    PlayerLeft { player_id: String },
    /// Game state changed
    StateChanged { new_state: String },
    /// Player action (bet, fold, etc.)
    PlayerAction { player_id: String, action: String },
    /// Round completed
    RoundComplete { winner: String, pot: u64 },
}

/// Network events
#[derive(Debug, Clone)]
pub enum NetworkEvent {
    /// Connected to server
    Connected,
    /// Disconnected from server
    Disconnected,
    /// Connection error
    Error(String),
}

/// Event handler for the application
pub struct EventHandler {
    sender: mpsc::UnboundedSender<AppEvent>,
    receiver: mpsc::UnboundedReceiver<AppEvent>,
}

impl EventHandler {
    pub fn new() -> Self {
        let (sender, receiver) = mpsc::unbounded_channel();
        Self { sender, receiver }
    }
    
    /// Get the sender for other components to send events
    pub fn sender(&self) -> mpsc::UnboundedSender<AppEvent> {
        self.sender.clone()
    }
    
    /// Receive the next event (async)
    pub async fn next(&mut self) -> Option<AppEvent> {
        self.receiver.recv().await
    }
    
    /// Send an event
    pub fn send(&self, event: AppEvent) -> Result<(), mpsc::error::SendError<AppEvent>> {
        self.sender.send(event)
    }
}

impl Default for EventHandler {
    fn default() -> Self {
        Self::new()
    }
}

/// Input event listener that converts crossterm events to AppEvents
pub struct InputListener {
    event_sender: mpsc::UnboundedSender<AppEvent>,
}

impl InputListener {
    pub fn new(event_sender: mpsc::UnboundedSender<AppEvent>) -> Self {
        Self { event_sender }
    }
    
    /// Start listening for input events (runs in background)
    pub async fn listen(&mut self) -> anyhow::Result<()> {
        loop {
            // Check for input with timeout
            if crossterm::event::poll(Duration::from_millis(100))? {
                match crossterm::event::read()? {
                    Event::Key(key) => {
                        // Handle special key combinations
                        match key {
                            KeyEvent { 
                                code: KeyCode::Char('c'), 
                                modifiers: KeyModifiers::CONTROL, 
                                .. 
                            } => {
                                let _ = self.event_sender.send(AppEvent::Quit);
                            }
                            _ => {
                                let _ = self.event_sender.send(AppEvent::Input(InputEvent::Key(key)));
                            }
                        }
                    }
                    Event::Mouse(mouse) => {
                        let _ = self.event_sender.send(AppEvent::Input(InputEvent::Mouse(mouse)));
                    }
                    Event::Resize(width, height) => {
                        let _ = self.event_sender.send(AppEvent::Resize(width, height));
                    }
                    Event::Paste(text) => {
                        let _ = self.event_sender.send(AppEvent::Input(InputEvent::Paste(text)));
                    }
                    _ => {} // Ignore other events
                }
            }
            
            // Send periodic tick
            let _ = self.event_sender.send(AppEvent::Tick);
            
            // Small delay to prevent busy waiting
            tokio::time::sleep(Duration::from_millis(50)).await;
        }
    }
}

/// Helper function to check if a key event matches expected key
pub fn is_key(event: &InputEvent, expected: KeyCode) -> bool {
    match event {
        InputEvent::Key(KeyEvent { code, .. }) => *code == expected,
        _ => false,
    }
}

/// Helper function to check if a key event is a character
pub fn is_char(event: &InputEvent, expected: char) -> bool {
    match event {
        InputEvent::Key(KeyEvent { code: KeyCode::Char(c), .. }) => *c == expected,
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_event_handler_creation() {
        let handler = EventHandler::new();
        let sender = handler.sender();
        
        // Test sending an event
        sender.send(AppEvent::Tick).unwrap();
    }
    
    #[test]
    fn test_key_matching() {
        let key_event = InputEvent::Key(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE));
        assert!(is_key(&key_event, KeyCode::Enter));
        assert!(!is_key(&key_event, KeyCode::Esc));
    }
    
    #[test]  
    fn test_char_matching() {
        let char_event = InputEvent::Key(KeyEvent::new(KeyCode::Char('a'), KeyModifiers::NONE));
        assert!(is_char(&char_event, 'a'));
        assert!(!is_char(&char_event, 'b'));
    }
}