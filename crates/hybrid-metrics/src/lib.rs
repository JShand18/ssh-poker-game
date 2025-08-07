use anyhow::Result;
use async_trait::async_trait;
use axum::{extract::State, http::StatusCode, response::Response};
use dogstatsd::{Client as StatsdClient, Options};
use log::{error, info, warn};
use prometheus::{Counter, Gauge, Histogram, Registry, TextEncoder};
use serde::{Deserialize, Serialize};
use std::time::Duration;
use tokio::time::Instant;

/// Configuration for hybrid monitoring setup
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitoringConfig {
    /// Service name for metrics
    pub service_name: String,
    /// Environment (development, production)
    pub environment: String,
    /// Version/build information
    pub version: String,
    /// Enable Datadog integration (cost consideration)
    pub enable_datadog: bool,
    /// Datadog StatsD endpoint
    pub datadog_endpoint: String,
    /// Prometheus metrics port
    pub prometheus_port: u16,
}

impl Default for MonitoringConfig {
    fn default() -> Self {
        Self {
            service_name: "ssh-poker-charm".to_string(),
            environment: "development".to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            enable_datadog: false, // Start free, upgrade later
            datadog_endpoint: "127.0.0.1:8125".to_string(),
            prometheus_port: 9090,
        }
    }
}

/// Core metrics that we track for the poker game
pub struct PokerMetrics {
    config: MonitoringConfig,
    
    // Prometheus metrics (always enabled - free!)
    registry: Registry,
    
    // Business metrics
    players_total: Counter,
    games_started: Counter,
    games_completed: Counter,
    active_games: Gauge,
    active_players: Gauge,
    
    // Performance metrics
    response_time: Histogram,
    database_query_time: Histogram,
    
    // System metrics
    memory_usage: Gauge,
    cpu_usage: Gauge,
    
    // Datadog client (optional for cost optimization)
    datadog_client: Option<StatsdClient>,
}

impl PokerMetrics {
    /// Create a new hybrid metrics instance
    pub fn new(config: MonitoringConfig) -> Result<Self> {
        let registry = Registry::new();
        
        // Initialize Prometheus metrics (free monitoring)
        let players_total = Counter::new("poker_players_total", "Total players served")?;
        let games_started = Counter::new("poker_games_started_total", "Total games started")?;
        let games_completed = Counter::new("poker_games_completed_total", "Total games completed")?;
        let active_games = Gauge::new("poker_active_games", "Currently active games")?;
        let active_players = Gauge::new("poker_active_players", "Currently active players")?;
        
        let response_time = Histogram::with_opts(prometheus::HistogramOpts::new("poker_response_duration_seconds", "Request response time"))?;
        let database_query_time = Histogram::with_opts(prometheus::HistogramOpts::new("poker_database_query_duration_seconds", "Database query time"))?;
        
        let memory_usage = Gauge::new("poker_memory_usage_bytes", "Memory usage in bytes")?;
        let cpu_usage = Gauge::new("poker_cpu_usage_percent", "CPU usage percentage")?;
        
        // Register all metrics with Prometheus
        registry.register(Box::new(players_total.clone()))?;
        registry.register(Box::new(games_started.clone()))?;
        registry.register(Box::new(games_completed.clone()))?;
        registry.register(Box::new(active_games.clone()))?;
        registry.register(Box::new(active_players.clone()))?;
        registry.register(Box::new(response_time.clone()))?;
        registry.register(Box::new(database_query_time.clone()))?;
        registry.register(Box::new(memory_usage.clone()))?;
        registry.register(Box::new(cpu_usage.clone()))?;
        
        // Initialize Datadog client if enabled (cost-conscious)
        let datadog_client = if config.enable_datadog {
            let options = Options::new("0.0.0.0:0", &config.datadog_endpoint, "poker", vec![
                format!("env:{}", config.environment),
                format!("service:{}", config.service_name),
                format!("version:{}", config.version),
            ]);
            match StatsdClient::new(options) {
                Ok(client) => {
                    info!("Datadog StatsD client initialized at {}", config.datadog_endpoint);
                    Some(client)
                }
                Err(e) => {
                    warn!("Failed to initialize Datadog client: {}. Continuing with Prometheus only.", e);
                    None
                }
            }
        } else {
            info!("Datadog disabled for cost optimization - using free Prometheus monitoring only");
            None
        };
        
        Ok(Self {
            config,
            registry,
            players_total,
            games_started,
            games_completed,
            active_games,
            active_players,
            response_time,
            database_query_time,
            memory_usage,
            cpu_usage,
            datadog_client,
        })
    }
    
    /// Track when a player joins the game
    pub fn track_player_joined(&self, player_id: &str, is_bot: bool) {
        // Prometheus (free)
        self.players_total.inc();
        
        // Datadog (paid, if enabled)
        if let Some(ref client) = self.datadog_client {
            let tags = vec![
                format!("env:{}", self.config.environment),
                format!("player_type:{}", if is_bot { "bot" } else { "human" }),
            ];
            let _ = client.incr("poker.player.joined", &tags);
        }
        
        info!("Player joined: {} (bot: {})", player_id, is_bot);
    }
    
    /// Track game lifecycle events
    pub fn track_game_started(&self, game_id: &str, player_count: usize) {
        self.games_started.inc();
        
        if let Some(ref client) = self.datadog_client {
            let tags = vec![
                format!("env:{}", self.config.environment),
                format!("players:{}", player_count),
            ];
            let _ = client.incr("poker.game.started", &tags);
            let _ = client.gauge("poker.game.player_count", &player_count.to_string(), &tags);
        }
        
        info!("Game started: {} with {} players", game_id, player_count);
    }
    
    pub fn track_game_completed(&self, game_id: &str, duration_seconds: u64, winner_id: &str) {
        self.games_completed.inc();
        
        if let Some(ref client) = self.datadog_client {
            let tags = vec![
                format!("env:{}", self.config.environment),
                format!("winner:{}", winner_id),
            ];
            let _ = client.incr("poker.game.completed", &tags);
            let _ = client.histogram("poker.game.duration", &duration_seconds.to_string(), &tags);
        }
        
        info!("Game completed: {} in {}s, winner: {}", game_id, duration_seconds, winner_id);
    }
    
    /// Update real-time gauges
    pub fn update_active_counts(&self, games: i64, players: i64) {
        self.active_games.set(games as f64);
        self.active_players.set(players as f64);
        
        if let Some(ref client) = self.datadog_client {
            let tags = vec![format!("env:{}", self.config.environment)];
            let _ = client.gauge("poker.active.games", &games.to_string(), &tags);
            let _ = client.gauge("poker.active.players", &players.to_string(), &tags);
        }
    }
    
    /// Track performance metrics
    pub fn track_response_time(&self, endpoint: &str, duration: Duration) {
        let duration_seconds = duration.as_secs_f64();
        self.response_time.observe(duration_seconds);
        
        if let Some(ref client) = self.datadog_client {
            let tags = vec![
                format!("env:{}", self.config.environment),
                format!("endpoint:{}", endpoint),
            ];
            let _ = client.histogram("poker.response.duration", &duration.as_millis().to_string(), &tags);
        }
    }
    
    pub fn track_database_query(&self, query_type: &str, duration: Duration, success: bool) {
        let duration_seconds = duration.as_secs_f64();
        self.database_query_time.observe(duration_seconds);
        
        if let Some(ref client) = self.datadog_client {
            let tags = vec![
                format!("env:{}", self.config.environment),
                format!("query_type:{}", query_type),
                format!("success:{}", success),
            ];
            let _ = client.histogram("poker.database.query.duration", &duration.as_millis().to_string(), &tags);
            
            if !success {
                let _ = client.incr("poker.database.errors", &tags);
            }
        }
    }
    
    /// Track system resource usage
    pub fn track_resource_usage(&self, memory_bytes: u64, cpu_percent: f64) {
        self.memory_usage.set(memory_bytes as f64);
        self.cpu_usage.set(cpu_percent);
        
        if let Some(ref client) = self.datadog_client {
            let tags = vec![format!("env:{}", self.config.environment)];
            let _ = client.gauge("poker.system.memory_bytes", &memory_bytes.to_string(), &tags);
            let _ = client.gauge("poker.system.cpu_percent", &cpu_percent.to_string(), &tags);
        }
    }
    
    /// Get Prometheus metrics for scraping (free endpoint)
    pub fn get_prometheus_metrics(&self) -> Result<String> {
        let encoder = TextEncoder::new();
        let metric_families = self.registry.gather();
        let encoded = encoder.encode_to_string(&metric_families)?;
        Ok(encoded)
    }
    
    /// Track custom poker-specific events
    pub fn track_poker_action(&self, action: &str, player_id: &str, game_id: &str, amount: Option<u64>) {
        if let Some(ref client) = self.datadog_client {
            let mut tags = vec![
                format!("env:{}", self.config.environment),
                format!("action:{}", action),
                format!("game_id:{}", game_id),
            ];
            
            if let Some(amt) = amount {
                tags.push(format!("amount_bucket:{}", self.amount_bucket(amt)));
                let _ = client.histogram("poker.action.amount", &amt.to_string(), &tags);
            }
            
            let _ = client.incr("poker.action", &tags);
        }
        
        info!("Poker action: {} by {} in game {} (amount: {:?})", action, player_id, game_id, amount);
    }
    
    /// Helper function to categorize bet amounts for analysis
    fn amount_bucket(&self, amount: u64) -> &'static str {
        match amount {
            0..=50 => "small",
            51..=200 => "medium", 
            201..=500 => "large",
            _ => "very_large",
        }
    }
}

/// Async trait for metrics middleware
#[async_trait]
pub trait MetricsMiddleware: Send + Sync {
    async fn track_request<T, F, Fut>(&self, endpoint: &str, handler: F) -> T
    where
        F: FnOnce() -> Fut + Send,
        Fut: std::future::Future<Output = T> + Send,
        T: Send;
}

#[async_trait]
impl MetricsMiddleware for PokerMetrics {
    async fn track_request<T, F, Fut>(&self, endpoint: &str, handler: F) -> T
    where
        F: FnOnce() -> Fut + Send,
        Fut: std::future::Future<Output = T> + Send,
        T: Send,
    {
        let start = Instant::now();
        let result = handler().await;
        let duration = start.elapsed();
        
        self.track_response_time(endpoint, duration);
        result
    }
}

/// Error types for metrics
#[derive(Debug, thiserror::Error)]
pub enum MetricsError {
    #[error("Prometheus error: {0}")]
    Prometheus(#[from] prometheus::Error),
    #[error("Datadog error: {0}")]
    Datadog(String),
    #[error("Metrics server error: {0}")]
    Server(#[from] anyhow::Error),
}

/// Start the Prometheus metrics HTTP server (free monitoring endpoint)
pub async fn start_metrics_server(metrics: std::sync::Arc<PokerMetrics>, port: u16) -> Result<()> {
    use axum::{routing::get, Router};
    use tower::ServiceBuilder;
    
    let app = Router::new()
        .route("/metrics", get(prometheus_metrics))
        .route("/health", get(health_check))
        .layer(ServiceBuilder::new())
        .with_state(metrics);
    
    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", port)).await?;
    
    info!("Starting Prometheus metrics server on port {}", port);
    
    axum::serve(listener, app).await?;
    
    Ok(())
}

async fn prometheus_metrics(State(metrics): State<std::sync::Arc<PokerMetrics>>) -> Result<Response<String>, StatusCode> {
    match metrics.get_prometheus_metrics() {
        Ok(metrics_text) => {
            let response = Response::builder()
                .status(StatusCode::OK)
                .header("content-type", "text/plain; version=0.0.4; charset=utf-8")
                .body(metrics_text)
                .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
            Ok(response)
        }
        Err(e) => {
            error!("Failed to generate Prometheus metrics: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

async fn health_check() -> Result<Response<String>, StatusCode> {
    let health_info = serde_json::json!({
        "status": "healthy",
        "service": "ssh-poker-charm",
        "version": env!("CARGO_PKG_VERSION"),
        "timestamp": chrono::Utc::now().to_rfc3339()
    });
    
    let response = Response::builder()
        .status(StatusCode::OK)
        .header("content-type", "application/json")
        .body(health_info.to_string())
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    
    Ok(response)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;
    
    #[tokio::test]
    async fn test_metrics_creation() {
        let config = MonitoringConfig::default();
        let metrics = PokerMetrics::new(config).unwrap();
        
        // Test Prometheus metrics (always available)
        metrics.track_player_joined("player1", false);
        metrics.track_game_started("game1", 4);
        metrics.update_active_counts(1, 4);
        
        let prometheus_output = metrics.get_prometheus_metrics().unwrap();
        assert!(prometheus_output.contains("poker_players_total"));
        assert!(prometheus_output.contains("poker_active_games"));
    }
    
    #[tokio::test]
    async fn test_datadog_disabled_cost_optimization() {
        let config = MonitoringConfig {
            enable_datadog: false, // Cost optimization
            ..Default::default()
        };
        
        let metrics = PokerMetrics::new(config).unwrap();
        assert!(metrics.datadog_client.is_none());
        
        // Should still work with Prometheus only
        metrics.track_player_joined("player1", true);
        let output = metrics.get_prometheus_metrics().unwrap();
        assert!(output.contains("poker_players_total"));
    }
}