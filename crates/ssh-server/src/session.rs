use anyhow::Result;
use log::{debug, info, warn, error};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::{Mutex, mpsc};
use uuid::Uuid;
use database::models::User;
use poker_engine::{GameState, Action, Player};
use terminal_ui::App;

#[derive(Debug, Clone)]
pub struct PlayerSession {
    pub id: Uuid,
    pub user_id: Uuid,
    pub username: String,
    pub connected_at: Instant,
    pub last_activity: Instant,
    pub is_authenticated: bool,
    pub current_table: Option<Uuid>,
    pub channel_id: Option<u32>,
}

impl PlayerSession {
    pub fn new(user_id: Uuid, username: String) -> Self {
        let now = Instant::now();
        Self {
            id: Uuid::new_v4(),
            user_id,
            username,
            connected_at: now,
            last_activity: now,
            is_authenticated: true,
            current_table: None,
            channel_id: None,
        }
    }

    pub fn update_activity(&mut self) {
        self.last_activity = Instant::now();
    }

    pub fn is_expired(&self, timeout: Duration) -> bool {
        self.last_activity.elapsed() > timeout
    }

    pub fn join_table(&mut self, table_id: Uuid) {
        self.current_table = Some(table_id);
        info!("Player {} joined table {}", self.username, table_id);
    }

    pub fn leave_table(&mut self) {
        if let Some(table_id) = self.current_table.take() {
            info!("Player {} left table {}", self.username, table_id);
        }
    }
}

#[derive(Debug)]
pub struct GameTable {
    pub id: Uuid,
    pub name: String,
    pub game_state: GameState,
    pub players: HashMap<Uuid, Player>,
    pub max_players: usize,
    pub small_blind: u64,
    pub big_blind: u64,
    pub created_at: Instant,
}

impl GameTable {
    pub fn new(name: String, max_players: usize, small_blind: u64, big_blind: u64) -> Self {
        let players = Vec::new();
        let game_state = GameState::new(players, small_blind, big_blind, 0);
        
        Self {
            id: Uuid::new_v4(),
            name,
            game_state,
            players: HashMap::new(),
            max_players,
            small_blind,
            big_blind,
            created_at: Instant::now(),
        }
    }

    pub fn add_player(&mut self, session_id: Uuid, chips: u64) -> Result<()> {
        if self.players.len() >= self.max_players {
            return Err(anyhow::anyhow!("Table is full"));
        }

        let player_id = self.players.len();
        let player = Player::new(player_id, session_id.to_string(), chips);
        self.players.insert(session_id, player);
        
        // Update game state with new players
        let players: Vec<Player> = self.players.values().cloned().collect();
        self.game_state = GameState::new(players, self.small_blind, self.big_blind, 0);
        
        info!("Player {} added to table {}", session_id, self.id);
        Ok(())
    }

    pub fn remove_player(&mut self, session_id: &Uuid) -> Result<()> {
        if self.players.remove(session_id).is_some() {
            // Update game state
            let players: Vec<Player> = self.players.values().cloned().collect();
            if !players.is_empty() {
                self.game_state = GameState::new(players, self.small_blind, self.big_blind, 0);
            }
            info!("Player {} removed from table {}", session_id, self.id);
            Ok(())
        } else {
            Err(anyhow::anyhow!("Player not found in table"))
        }
    }

    pub fn is_empty(&self) -> bool {
        self.players.is_empty()
    }

    pub fn player_count(&self) -> usize {
        self.players.len()
    }

    pub fn process_action(&mut self, session_id: &Uuid, action: Action) -> Result<()> {
        if !self.players.contains_key(session_id) {
            return Err(anyhow::anyhow!("Player not in this table"));
        }

        // Process the action through the game engine
        match self.game_state.process_action(action) {
            Ok(_) => {
                debug!("Action {:?} processed for player {} in table {}", action, session_id, self.id);
                Ok(())
            }
            Err(e) => {
                warn!("Invalid action {:?} from player {} in table {}: {}", action, session_id, self.id, e);
                Err(anyhow::anyhow!("Invalid action: {}", e))
            }
        }
    }
}

pub struct SessionManager {
    sessions: Arc<Mutex<HashMap<Uuid, PlayerSession>>>,
    tables: Arc<Mutex<HashMap<Uuid, GameTable>>>,
    session_timeout: Duration,
    cleanup_interval: Duration,
}

impl SessionManager {
    pub fn new() -> Self {
        Self {
            sessions: Arc::new(Mutex::new(HashMap::new())),
            tables: Arc::new(Mutex::new(HashMap::new())),
            session_timeout: Duration::from_secs(300), // 5 minutes
            cleanup_interval: Duration::from_secs(60), // 1 minute
        }
    }

    pub async fn create_session(&self, user: User) -> Uuid {
        let user_id = Uuid::parse_str(&user.id).unwrap_or_else(|_| Uuid::new_v4());
        let session = PlayerSession::new(user_id, user.username.clone());
        let session_id = session.id;
        
        {
            let mut sessions = self.sessions.lock().await;
            sessions.insert(session_id, session);
        }
        
        info!("Created session {} for user {}", session_id, user.username);
        session_id
    }

    pub async fn get_session(&self, session_id: &Uuid) -> Option<PlayerSession> {
        let sessions = self.sessions.lock().await;
        sessions.get(session_id).cloned()
    }

    pub async fn update_session_activity(&self, session_id: &Uuid) {
        let mut sessions = self.sessions.lock().await;
        if let Some(session) = sessions.get_mut(session_id) {
            session.update_activity();
        }
    }

    pub async fn remove_session(&self, session_id: &Uuid) {
        let mut sessions = self.sessions.lock().await;
        if let Some(session) = sessions.remove(session_id) {
            // Remove player from any table they're in
            if let Some(table_id) = session.current_table {
                drop(sessions); // Release the lock before acquiring another
                let _ = self.leave_table(session_id, &table_id).await;
            }
            info!("Removed session {} for user {}", session_id, session.username);
        }
    }

    pub async fn create_table(&self, name: String, max_players: usize, small_blind: u64, big_blind: u64) -> Uuid {
        let table = GameTable::new(name.clone(), max_players, small_blind, big_blind);
        let table_id = table.id;
        
        {
            let mut tables = self.tables.lock().await;
            tables.insert(table_id, table);
        }
        
        info!("Created table {} with name '{}'", table_id, name);
        table_id
    }

    pub async fn join_table(&self, session_id: &Uuid, table_id: &Uuid, chips: u64) -> Result<()> {
        // Update session
        {
            let mut sessions = self.sessions.lock().await;
            if let Some(session) = sessions.get_mut(session_id) {
                // Leave current table if in one
                if let Some(current_table) = session.current_table {
                    drop(sessions); // Release lock before recursive call
                    let _ = self.leave_table(session_id, &current_table).await;
                    let mut sessions = self.sessions.lock().await; // Re-acquire lock
                    if let Some(session) = sessions.get_mut(session_id) {
                        session.join_table(*table_id);
                    }
                } else {
                    session.join_table(*table_id);
                }
            } else {
                return Err(anyhow::anyhow!("Session not found"));
            }
        }

        // Add to table
        {
            let mut tables = self.tables.lock().await;
            if let Some(table) = tables.get_mut(table_id) {
                table.add_player(*session_id, chips)?;
            } else {
                return Err(anyhow::anyhow!("Table not found"));
            }
        }

        Ok(())
    }

    pub async fn leave_table(&self, session_id: &Uuid, table_id: &Uuid) -> Result<()> {
        // Update session
        {
            let mut sessions = self.sessions.lock().await;
            if let Some(session) = sessions.get_mut(session_id) {
                session.leave_table();
            }
        }

        // Remove from table
        {
            let mut tables = self.tables.lock().await;
            if let Some(table) = tables.get_mut(table_id) {
                table.remove_player(session_id)?;
                
                // Remove empty tables
                if table.is_empty() {
                    tables.remove(table_id);
                    info!("Removed empty table {}", table_id);
                }
            }
        }

        Ok(())
    }

    pub async fn process_game_action(&self, session_id: &Uuid, action: Action) -> Result<()> {
        let table_id = {
            let sessions = self.sessions.lock().await;
            if let Some(session) = sessions.get(session_id) {
                session.current_table
            } else {
                return Err(anyhow::anyhow!("Session not found"));
            }
        };

        if let Some(table_id) = table_id {
            let mut tables = self.tables.lock().await;
            if let Some(table) = tables.get_mut(&table_id) {
                table.process_action(session_id, action)?;
            } else {
                return Err(anyhow::anyhow!("Table not found"));
            }
        } else {
            return Err(anyhow::anyhow!("Player not at any table"));
        }

        Ok(())
    }

    pub async fn get_table_state(&self, table_id: &Uuid) -> Option<GameState> {
        let tables = self.tables.lock().await;
        tables.get(table_id).map(|table| table.game_state.clone())
    }

    pub async fn list_tables(&self) -> Vec<(Uuid, String, usize, usize)> {
        let tables = self.tables.lock().await;
        tables.values()
            .map(|table| (table.id, table.name.clone(), table.player_count(), table.max_players))
            .collect()
    }

    pub async fn get_player_table(&self, session_id: &Uuid) -> Option<Uuid> {
        let sessions = self.sessions.lock().await;
        sessions.get(session_id).and_then(|session| session.current_table)
    }

    pub async fn cleanup_expired_sessions(&self) {
        let expired_session_ids: Vec<Uuid> = {
            let sessions = self.sessions.lock().await;
            sessions.iter()
                .filter(|(_, session)| session.is_expired(self.session_timeout))
                .map(|(id, _)| *id)
                .collect()
        };

        for session_id in expired_session_ids {
            warn!("Cleaning up expired session: {}", session_id);
            self.remove_session(&session_id).await;
        }
    }

    pub fn start_cleanup_task(session_manager: Arc<Self>) {
        let cleanup_interval = session_manager.cleanup_interval;
        
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(cleanup_interval);
            loop {
                interval.tick().await;
                session_manager.cleanup_expired_sessions().await;
            }
        });
    }

    pub async fn session_count(&self) -> usize {
        let sessions = self.sessions.lock().await;
        sessions.len()
    }

    pub async fn table_count(&self) -> usize {
        let tables = self.tables.lock().await;
        tables.len()
    }
}

impl Default for SessionManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use database::models::User;

    fn create_test_user(username: &str) -> User {
        User {
            id: Uuid::new_v4(),
            username: username.to_string(),
            email: format!("{}@example.com", username),
            password_hash: "test_hash".to_string(),
            public_key: None,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        }
    }

    #[tokio::test]
    async fn test_session_creation() {
        let manager = SessionManager::new();
        let user = create_test_user("testuser");
        
        let session_id = manager.create_session(user.clone()).await;
        assert!(!session_id.is_nil());
        
        let session = manager.get_session(&session_id).await.unwrap();
        assert_eq!(session.username, user.username);
        assert!(session.is_authenticated);
    }

    #[tokio::test]
    async fn test_table_creation_and_joining() {
        let manager = SessionManager::new();
        let user = create_test_user("testuser");
        
        let session_id = manager.create_session(user).await;
        let table_id = manager.create_table("Test Table".to_string(), 6, 10, 20).await;
        
        assert!(manager.join_table(&session_id, &table_id, 1000).await.is_ok());
        
        let player_table = manager.get_player_table(&session_id).await;
        assert_eq!(player_table, Some(table_id));
    }

    #[tokio::test]
    async fn test_table_cleanup() {
        let manager = SessionManager::new();
        let user = create_test_user("testuser");
        
        let session_id = manager.create_session(user).await;
        let table_id = manager.create_table("Test Table".to_string(), 6, 10, 20).await;
        
        manager.join_table(&session_id, &table_id, 1000).await.unwrap();
        assert_eq!(manager.table_count().await, 1);
        
        manager.leave_table(&session_id, &table_id).await.unwrap();
        assert_eq!(manager.table_count().await, 0); // Table should be removed when empty
    }

    #[tokio::test]
    async fn test_session_activity_tracking() {
        let manager = SessionManager::new();
        let user = create_test_user("testuser");
        
        let session_id = manager.create_session(user).await;
        let session = manager.get_session(&session_id).await.unwrap();
        let initial_activity = session.last_activity;
        
        // Small delay to ensure time difference
        tokio::time::sleep(Duration::from_millis(10)).await;
        
        manager.update_session_activity(&session_id).await;
        let updated_session = manager.get_session(&session_id).await.unwrap();
        
        assert!(updated_session.last_activity > initial_activity);
    }
}