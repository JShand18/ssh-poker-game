//! Main application structure inspired by Bubble Tea architecture
//! 
//! Provides a clean, event-driven TUI application framework

use std::time::Duration;
use anyhow::Result;
use ratatui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout, Rect},
    Terminal, Frame,
};
use tokio::time::Instant;

use crate::{
    events::{AppEvent, EventHandler, InputEvent, InputListener},
    themes::CasinoStyles,
    views::{AuthView, GameView, LobbyView, View},
};

/// Main poker application following Bubble Tea patterns
pub struct PokerApp {
    /// Current application state
    state: AppState,
    /// Event handling system
    event_handler: EventHandler,
    /// UI styling
    styles: CasinoStyles,
    /// Current view
    current_view: Box<dyn View>,
    /// Application should quit
    should_quit: bool,
    /// Last frame time for FPS calculation
    last_frame: Instant,
}

/// Application states
#[derive(Debug, Clone, PartialEq)]
pub enum AppState {
    /// Authentication/welcome screen
    Auth,
    /// In the lobby waiting for players
    Lobby,
    /// Playing a poker game
    InGame,
    /// Paused/settings
    Paused,
    /// Error state
    Error(String),
}

impl PokerApp {
    /// Create a new poker application
    pub fn new() -> Result<Self> {
        let event_handler = EventHandler::new();
        let styles = CasinoStyles::new();
        let current_view = Box::new(AuthView::new());
        
        Ok(Self {
            state: AppState::Auth,
            event_handler,
            styles,
            current_view,
            should_quit: false,
            last_frame: Instant::now(),
        })
    }
    
    /// Run the application event loop
    pub async fn run<B: Backend>(&mut self, terminal: &mut Terminal<B>) -> Result<()> {
        // Start input listener in background
        let mut input_listener = InputListener::new(self.event_handler.sender());
        let _input_handle = tokio::spawn(async move {
            if let Err(e) = input_listener.listen().await {
                log::error!("Input listener error: {}", e);
            }
        });
        
        loop {
            // Draw the current frame
            terminal.draw(|frame| {
                let _ = self.draw(frame, frame.area());
            })?;
            
            // Handle events
            if let Some(event) = self.event_handler.next().await {
                self.handle_event(event).await?;
                
                if self.should_quit {
                    break;
                }
            }
            
            // Small delay to prevent busy waiting
            tokio::time::sleep(Duration::from_millis(16)).await; // ~60 FPS
        }
        
        Ok(())
    }

    /// Run the application using events supplied externally via the internal EventHandler.
    /// This does not spawn the internal crossterm InputListener.
    pub async fn run_with_external_events<B: Backend>(&mut self, terminal: &mut Terminal<B>) -> Result<()> {
        loop {
            terminal.draw(|frame| {
                let _ = self.draw(frame, frame.area());
            })?;

            if let Some(event) = self.event_handler.next().await {
                self.handle_event(event).await?;
                if self.should_quit { break; }
            }

            tokio::time::sleep(Duration::from_millis(16)).await;
        }
        Ok(())
    }

    /// Get a clone of the event sender so external sources can feed events
    pub fn event_sender(&self) -> tokio::sync::mpsc::UnboundedSender<AppEvent> {
        self.event_handler.sender()
    }
    
    /// Try to receive an event without blocking
    pub async fn try_recv_event(&mut self) -> Option<AppEvent> {
        // Use a very short timeout to make it non-blocking
        tokio::time::timeout(tokio::time::Duration::from_millis(1), self.event_handler.next()).await.ok().flatten()
    }
    
    /// Handle application events (Bubble Tea-style update function)
    pub async fn handle_event(&mut self, event: AppEvent) -> Result<()> {
        match event {
            AppEvent::Quit => {
                self.should_quit = true;
            }
            
            AppEvent::Input(input) => {
                // Let the current view handle input first
                if let Some(new_state) = self.current_view.handle_input(&input, &self.state) {
                    self.transition_to_state(new_state);
                }
                
                // Global input handling
                match input {
                    InputEvent::Key(key) => {
                        match key.code {
                            crossterm::event::KeyCode::Char('q') => {
                                self.should_quit = true;
                            }
                            crossterm::event::KeyCode::F(1) => {
                                // Toggle between lobby and game for demo
                                match self.state {
                                    AppState::Lobby => self.transition_to_state(AppState::InGame),
                                    AppState::InGame => self.transition_to_state(AppState::Lobby),
                                    _ => {}
                                }
                            }
                            _ => {}
                        }
                    }
                    _ => {}
                }
            }
            
            AppEvent::GameUpdate(game_event) => {
                // Forward to current view
                self.current_view.handle_game_event(&game_event, &self.state);
            }
            
            AppEvent::Network(net_event) => {
                // Handle network events
                self.current_view.handle_network_event(&net_event, &self.state);
            }
            
            AppEvent::Tick => {
                // Update views on tick
                self.current_view.update(&self.state);
            }
            
            AppEvent::Resize(width, height) => {
                // Handle terminal resize - ratatui handles this automatically
                log::info!("Terminal resized to {}x{}", width, height);
            }
        }
        
        Ok(())
    }
    
    /// Transition to a new application state
    fn transition_to_state(&mut self, new_state: AppState) {
        if self.state != new_state {
            log::info!("State transition: {:?} -> {:?}", self.state, new_state);
            
            // Create appropriate view for new state
            self.current_view = match new_state {
                AppState::Auth => Box::new(AuthView::new()),
                AppState::Lobby => Box::new(LobbyView::new()),
                AppState::InGame => Box::new(GameView::new()),
                AppState::Paused => Box::new(LobbyView::new()), // TODO: Create PausedView
                AppState::Error(ref msg) => {
                    log::error!("Entering error state: {}", msg);
                    Box::new(LobbyView::new()) // TODO: Create ErrorView
                }
            };
            
            self.state = new_state;
        }
    }
    
    /// Draw the application (Bubble Tea-style view function)
    pub fn draw(&mut self, frame: &mut Frame, _area: Rect) -> Result<()> {
        let size = frame.area();
        log::debug!("Drawing app in state {:?}, area: {}x{}", self.state, size.width, size.height);
        
        // Calculate FPS for debug info
        let now = Instant::now();
        let _fps = 1.0 / now.duration_since(self.last_frame).as_secs_f64();
        self.last_frame = now;
        
        // Main layout
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(1),    // Status bar
                Constraint::Min(0),       // Main content
                Constraint::Length(2),    // Help/controls
            ])
            .split(size);
        
        // Status bar
        self.draw_status_bar(frame, chunks[0]);
        
        // Main content (delegate to current view)
        self.current_view.render(chunks[1], frame, &self.styles);
        
        // Help bar
        self.draw_help_bar(frame, chunks[2]);
        
        // Debug info in corner (only in debug builds)
        #[cfg(debug_assertions)]
        {
            let debug_text = format!("FPS: {:.1} | State: {:?}", _fps, self.state);
            let debug_area = Rect {
                x: size.width.saturating_sub(debug_text.len() as u16 + 2),
                y: 0,
                width: debug_text.len() as u16 + 2,
                height: 1,
            };
            
            let debug_paragraph = ratatui::widgets::Paragraph::new(debug_text)
                .style(self.styles.subtitle());
            frame.render_widget(debug_paragraph, debug_area);
        }
        
        Ok(())
    }
    
    /// Draw the status bar
    fn draw_status_bar(&self, frame: &mut Frame, area: Rect) {
        let status_text = match self.state {
            AppState::Auth => "🔐 Welcome - Choose how to play",
            AppState::Lobby => "🏠 Lobby - Waiting for players...",
            AppState::InGame => "🎮 In Game - Your turn!",
            AppState::Paused => "⏸️ Paused",
            AppState::Error(ref _msg) => return, // Skip status for errors
        };
        
        let status = ratatui::widgets::Paragraph::new(status_text)
            .style(self.styles.subtitle());
        
        frame.render_widget(status, area);
    }
    
    /// Draw the help bar
    fn draw_help_bar(&self, frame: &mut Frame, area: Rect) {
        let help_text = "Press 'q' to quit | F1 to toggle demo | Arrow keys to navigate";
        
        let help = ratatui::widgets::Paragraph::new(help_text)
            .style(self.styles.subtitle())
            .alignment(ratatui::layout::Alignment::Center);
        
        frame.render_widget(help, area);
    }
    
    /// Get current application state
    pub fn state(&self) -> &AppState {
        &self.state
    }
    
    /// Check if application should quit
    pub fn should_quit(&self) -> bool {
        self.should_quit
    }
}

impl Default for PokerApp {
    fn default() -> Self {
        Self::new().expect("Failed to create PokerApp")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_poker_app_creation() {
        let app = PokerApp::new().unwrap();
        assert_eq!(app.state(), &AppState::Lobby);
        assert!(!app.should_quit());
    }
    
    #[test]
    fn test_state_transitions() {
        let mut app = PokerApp::new().unwrap();
        
        // Test lobby to game transition
        app.transition_to_state(AppState::InGame);
        assert_eq!(app.state(), &AppState::InGame);
        
        // Test game to lobby transition
        app.transition_to_state(AppState::Lobby);
        assert_eq!(app.state(), &AppState::Lobby);
    }
}