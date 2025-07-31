use crate::{TuiError, ui};
use poker_engine::{GameState, Player, Action};
use tokio::sync::mpsc;
use futures::stream::StreamExt;
use crossterm::event::{Event, KeyCode};

/// The main application state
#[derive(Clone)]
pub struct App {
    pub state: AppState,
    pub input: String,
    pub messages: Vec<String>,
    pub game: Option<GameState>,
    pub focused_component: FocusedComponent,
    pub selected_action: usize,
    pub should_quit: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppState {
    MainMenu,
    InGame,
    GameOver,
    Help,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FocusedComponent {
    ActionList,
    ChatInput,
    MainArea,
}

impl App {
    pub fn new() -> Self {
        Self {
            state: AppState::MainMenu,
            input: String::new(),
            messages: vec![
                "Welcome to SSH Poker!".to_string(),
                "Press 'n' to start a new game".to_string(),
                "Press 'h' for help".to_string(),
                "Press 'q' to quit".to_string(),
            ],
            game: None,
            focused_component: FocusedComponent::MainArea,
            selected_action: 0,
            should_quit: false,
        }
    }

    pub fn state(&self) -> AppState {
        self.state
    }

    pub fn show_help(&mut self) {
        self.state = AppState::Help;
        self.messages.clear();
        self.messages.push("=== SSH Poker Help ===".to_string());
        self.messages.push("".to_string());
        self.messages.push("Navigation:".to_string());
        self.messages.push("  Tab       - Switch between components".to_string());
        self.messages.push("  ↑/↓       - Navigate action list".to_string());
        self.messages.push("  Enter     - Select action/send message".to_string());
        self.messages.push("".to_string());
        self.messages.push("Commands:".to_string());
        self.messages.push("  n         - New game".to_string());
        self.messages.push("  h         - Show this help".to_string());
        self.messages.push("  q         - Quit".to_string());
        self.messages.push("".to_string());
        self.messages.push("Press any key to return...".to_string());
    }

    pub fn process_input(&mut self) {
        match self.focused_component {
            FocusedComponent::ChatInput => {
                if !self.input.is_empty() {
                    self.messages.push(format!("> {}", self.input));
                    self.process_command(&self.input.clone());
                    self.input.clear();
                }
            }
            FocusedComponent::ActionList => {
                if self.state == AppState::InGame {
                    // Process the selected action
                    self.messages.push(format!("Selected action: {}", self.selected_action));
                }
            }
            _ => {}
        }
    }

    fn process_command(&mut self, command: &str) {
        match command.trim() {
            "n" | "new" => self.start_new_game(),
            "h" | "help" => self.show_help(),
            _ => {
                self.messages.push(format!("Unknown command: {}", command));
            }
        }
    }

    fn start_new_game(&mut self) {
        self.state = AppState::InGame;
        self.messages.clear();
        self.messages.push("Starting new game...".to_string());
        
        // Create a simple 2-player game
        let players = vec![
            Player::new(0, "You".to_string(), 1000),
            Player::new(1, "Opponent".to_string(), 1000),
        ];
        
        let mut game = GameState::new(players, 10, 20, 0);
        game.start_new_hand();
        
        self.game = Some(game);
        self.messages.push("Game started! Good luck!".to_string());
    }

    pub fn on_char(&mut self, c: char) {
        if self.state == AppState::Help {
            self.state = AppState::MainMenu;
            self.messages.clear();
            self.messages.push("Returned to main menu".to_string());
            return;
        }

        match self.focused_component {
            FocusedComponent::ChatInput => {
                self.input.push(c);
            }
            FocusedComponent::MainArea => {
                match c {
                    'n' => self.start_new_game(),
                    'h' => self.show_help(),
                    _ => {}
                }
            }
            _ => {}
        }
    }

    pub fn on_backspace(&mut self) {
        if self.focused_component == FocusedComponent::ChatInput {
            self.input.pop();
        }
    }

    pub fn on_up(&mut self) {
        if self.focused_component == FocusedComponent::ActionList && self.selected_action > 0 {
            self.selected_action -= 1;
        }
    }

    pub fn on_down(&mut self) {
        if self.focused_component == FocusedComponent::ActionList {
            // TODO: Check against actual action count
            self.selected_action += 1;
        }
    }

    pub fn next_focus(&mut self) {
        self.focused_component = match self.focused_component {
            FocusedComponent::MainArea => FocusedComponent::ActionList,
            FocusedComponent::ActionList => FocusedComponent::ChatInput,
            FocusedComponent::ChatInput => FocusedComponent::MainArea,
        };
    }

    pub fn get_available_actions(&self) -> Vec<String> {
        if let Some(game) = &self.game {
            game.get_valid_actions()
                .iter()
                .map(|action| format!("{:?}", action))
                .collect()
        } else {
            vec![]
        }
    }

    pub fn init_terminal(&mut self) -> Result<ratatui::Terminal<ratatui::backend::CrosstermBackend<std::io::Stdout>>, TuiError> {
        let mut stdout = std::io::stdout();
        crossterm::execute!(stdout, crossterm::terminal::EnterAlternateScreen)?;
        crossterm::terminal::enable_raw_mode()?;
        let backend = ratatui::backend::CrosstermBackend::new(stdout);
        let terminal = ratatui::Terminal::new(backend)?;
        Ok(terminal)
    }

    pub fn handle_input(&mut self) -> mpsc::Receiver<crossterm::event::Event> {
        let (tx, rx) = mpsc::channel(1);
        tokio::spawn(async move {
            let mut reader = crossterm::event::EventStream::new();
            while let Some(Ok(event)) = reader.next().await {
                if tx.send(event).await.is_err() {
                    break;
                }
            }
        });
        rx
    }

    pub async fn run<B: ratatui::backend::Backend>(
        &mut self,
        terminal: &mut ratatui::Terminal<B>,
    ) -> Result<(), TuiError> {
        loop {
            terminal.draw(|f| ui::draw(f, self))?;
            // The input handling will be done in the SSH server
        }
    }

    pub fn on_key(&mut self, event: Event) {
        if let Event::Key(key) = event {
            match key.code {
                KeyCode::Char('q') => self.should_quit = true,
                _ => {}
            }
        }
    }
} 