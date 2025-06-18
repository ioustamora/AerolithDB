//! # TUI (Terminal User Interface) Module
//!
//! This module provides a comprehensive Terminal User Interface for AerolithDB CLI
//! using the ratatui crate. It serves as the main entry point for the modern TUI
//! interface that replaces traditional command-line interaction with an interactive,
//! real-time management console.
//!
//! ## Features
//!
//! - **Dashboard**: Real-time system metrics, node status, and activity monitoring
//! - **Node Manager**: Interactive node lifecycle management with visual feedback  
//! - **Cluster Monitor**: Network topology visualization and health monitoring
//! - **Test Runner**: Integrated test suite execution with progress tracking
//! - **Configuration**: Live configuration editing with validation and hot-reload
//! - **Console**: Interactive command execution with history and auto-completion
//!
//! ## Architecture
//!
//! The TUI is built with a modular architecture:
//! - `App`: Central application state management
//! - Event handling for keyboard/mouse input
//! - Background task coordination for real-time updates
//! - Pluggable UI components for each functional area
//!
//! ## Usage
//!
//! The TUI can be launched as the default CLI mode or explicitly via flags:
//! ```bash
//! aerolithsdb-cli --tui              # Explicit TUI mode
//! aerolithsdb-cli                    # Default mode (TUI)
//! aerolithsdb-cli --no-tui <command> # Traditional CLI mode
//! ```

use anyhow::Result;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::{Backend, CrosstermBackend},
    Terminal,
};
use std::{
    io,
    sync::Arc,
    time::{Duration, Instant},
};
use tokio::sync::mpsc;

use crate::client::aerolithsClient;

/// Main TUI application state
pub mod app;

/// User interface rendering components  
pub mod ui;

/// Event handling and input processing
pub mod events;

use app::App;

/// Default tick rate for the TUI event loop (60 FPS)
const TICK_RATE: Duration = Duration::from_millis(16);

/// TUI application runner
pub struct TuiApp {
    /// Main application state
    app: App,
    /// aerolithsDB client for API communication
    client: Arc<aerolithsClient>,
}

impl TuiApp {
    /// Create a new TUI application instance
    pub fn new(client: aerolithsClient) -> Self {
        Self {
            app: App::new(),
            client: Arc::new(client),
        }
    }

    /// Run the TUI application
    pub async fn run(&mut self) -> Result<()> {
        // Setup terminal
        enable_raw_mode()?;
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
        let backend = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend)?;

        // Start background tasks for real-time data updates
        self.app.start_background_tasks(self.client.clone()).await?;

        // Create the main event loop
        let result = self.run_app(&mut terminal).await;

        // Cleanup terminal
        disable_raw_mode()?;
        execute!(
            terminal.backend_mut(),
            LeaveAlternateScreen,
            DisableMouseCapture
        )?;
        terminal.show_cursor()?;

        result
    }

    /// Main application event loop
    async fn run_app<B: Backend>(&mut self, terminal: &mut Terminal<B>) -> Result<()> {
        let mut last_tick = Instant::now();

        loop {
            // Render the UI
            terminal.draw(|f| ui::render(f, &self.app))?;

            // Handle events with timeout for periodic updates
            let timeout = TICK_RATE.saturating_sub(last_tick.elapsed());
            
            if crossterm::event::poll(timeout)? {
                if let Event::Key(key) = event::read()? {
                    // Only process key press events, ignore key release
                    if key.kind == KeyEventKind::Press {
                        events::handle_key_event(&mut self.app, key, self.client.clone()).await?;
                    }
                }
            }

            // Periodic tick for animations and updates
            if last_tick.elapsed() >= TICK_RATE {
                self.app_tick().await?;
                last_tick = Instant::now();
            }

            // Check if application should quit
            if self.app.should_quit {
                break;
            }
        }

        Ok(())
    }

    /// Handle periodic application updates (animations, data refresh, etc.)
    async fn app_tick(&mut self) -> Result<()> {
        // Update any time-sensitive UI elements
        // Handle background task results
        // Trigger periodic data refreshes
        
        // Clear status messages after a timeout
        if let Some(_) = &self.app.status_message {
            // In a real implementation, you'd track when the message was set
            // and clear it after a certain duration
        }

        Ok(())
    }
}

/// Launch the TUI interface
pub async fn launch_tui(client: aerolithsClient) -> Result<()> {
    let mut tui_app = TuiApp::new(client);
    tui_app.run().await
}

/// Main TUI application state
pub struct App {
    /// Current tab selection
    current_tab: usize,
    
    /// Should the application quit
    should_quit: bool,
    
    /// Node management state
    node_manager: NodeManager,
    
    /// Cluster monitoring state
    cluster_monitor: ClusterMonitor,
    
    /// Test runner state
    test_runner: TestRunner,
    
    /// Configuration manager
    config_manager: ConfigManager,
    
    /// System metrics
    system_metrics: SystemMetrics,
    
    /// Console output and logs
    console: Console,
    
    /// Input handler for various screens
    input: Input,
    
    /// HTTP client for API communication
    client: Arc<aerolithsClient>,
    
    /// Background task communications
    event_sender: mpsc::UnboundedSender<AppEvent>,
    event_receiver: mpsc::UnboundedReceiver<AppEvent>,
}

/// Application events for async operations
#[derive(Debug, Clone)]
pub enum AppEvent {
    NodeStatusUpdate(NodeStatus),
    ClusterMetricsUpdate(ClusterMetrics),
    TestResults(TestResults),
    LogMessage(LogEntry),
    SystemUpdate(SystemInfo),
    Error(String),
}

/// Available tabs in the TUI
#[derive(Debug, Clone, Copy)]
pub enum Tab {
    Dashboard = 0,
    NodeManager = 1,
    ClusterMonitor = 2,
    TestRunner = 3,
    Configuration = 4,
    Console = 5,
}

impl Tab {
    const ALL: [Tab; 6] = [
        Tab::Dashboard,
        Tab::NodeManager,
        Tab::ClusterMonitor,
        Tab::TestRunner,
        Tab::Configuration,
        Tab::Console,
    ];
    
    fn name(self) -> &'static str {
        match self {
            Tab::Dashboard => "Dashboard",
            Tab::NodeManager => "Node Manager",
            Tab::ClusterMonitor => "Cluster Monitor",
            Tab::TestRunner => "Test Runner",
            Tab::Configuration => "Configuration",
            Tab::Console => "Console",
        }
    }
    
    fn icon(self) -> &'static str {
        match self {
            Tab::Dashboard => "📊",
            Tab::NodeManager => "🖥️",
            Tab::ClusterMonitor => "🌐",
            Tab::TestRunner => "🧪",
            Tab::Configuration => "⚙️",
            Tab::Console => "💻",
        }
    }
}

/// Node management functionality
#[derive(Default)]
pub struct NodeManager {
    nodes: Vec<NodeInfo>,
    selected_node: usize,
    node_list_state: ListState,
    is_starting_node: bool,
    start_progress: f64,
    node_config_input: Input,
    show_node_config: bool,
}

/// Node information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeInfo {
    pub id: String,
    pub name: String,
    pub status: NodeStatus,
    pub endpoint: String,
    pub node_type: NodeType,
    pub started_at: Option<chrono::DateTime<chrono::Utc>>,
    pub metrics: NodeMetrics,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NodeStatus {
    Stopped,
    Starting,
    Running,
    Error(String),
    Stopping,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NodeType {
    Bootstrap,
    Regular,
    Witness,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct NodeMetrics {
    pub cpu_usage: f64,
    pub memory_usage: f64,
    pub disk_usage: f64,
    pub network_io: (u64, u64), // (in, out)
    pub api_requests: u64,
    pub consensus_operations: u64,
}

/// Cluster monitoring functionality
#[derive(Default)]
pub struct ClusterMonitor {
    cluster_state: ClusterState,
    cluster_metrics: ClusterMetrics,
    selected_metric: usize,
    metric_history: Vec<(Instant, ClusterMetrics)>,
    auto_refresh: bool,
    refresh_interval: Duration,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ClusterState {
    pub total_nodes: usize,
    pub active_nodes: usize,
    pub consensus_status: ConsensusStatus,
    pub network_partitions: Vec<String>,
    pub replication_factor: u32,
    pub total_documents: u64,
    pub total_collections: u64,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ClusterMetrics {
    pub throughput_ops_per_sec: f64,
    pub avg_latency_ms: f64,
    pub storage_used_gb: f64,
    pub storage_total_gb: f64,
    pub cache_hit_rate: f64,
    pub consensus_time_ms: f64,
    pub active_connections: u32,
    pub errors_per_min: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConsensusStatus {
    Healthy,
    Degraded,
    Failed,
    Unknown,
}

impl Default for ConsensusStatus {
    fn default() -> Self {
        ConsensusStatus::Unknown
    }
}

/// Test runner functionality
#[derive(Default)]
pub struct TestRunner {
    available_tests: Vec<TestSuite>,
    selected_test: usize,
    test_list_state: ListState,
    running_tests: HashMap<String, TestExecution>,
    test_results: Vec<TestResults>,
    show_test_output: bool,
    selected_test_result: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestSuite {
    pub name: String,
    pub description: String,
    pub test_type: TestType,
    pub estimated_duration: Duration,
    pub requirements: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TestType {
    Unit,
    Integration,
    Performance,
    BattleTest,
    NetworkTest,
    SecurityTest,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestExecution {
    pub suite_name: String,
    pub started_at: Instant,
    pub progress: f64,
    pub current_test: String,
    pub status: TestStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TestStatus {
    Running,
    Passed,
    Failed,
    Skipped,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestResults {
    pub suite_name: String,
    pub started_at: chrono::DateTime<chrono::Utc>,
    pub duration: Duration,
    pub total_tests: usize,
    pub passed: usize,
    pub failed: usize,
    pub skipped: usize,
    pub status: TestStatus,
    pub output: String,
    pub errors: Vec<String>,
}

/// Configuration management
#[derive(Default)]
pub struct ConfigManager {
    config_files: Vec<ConfigFile>,
    selected_config: usize,
    config_list_state: ListState,
    current_config: String,
    config_input: Input,
    show_editor: bool,
    validation_status: ConfigValidation,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigFile {
    pub name: String,
    pub path: String,
    pub file_type: ConfigType,
    pub last_modified: chrono::DateTime<chrono::Utc>,
    pub is_valid: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConfigType {
    Node,
    Cluster,
    API,
    Security,
    Storage,
}

#[derive(Debug, Clone, Default)]
pub struct ConfigValidation {
    pub is_valid: bool,
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
}

/// System metrics and information
#[derive(Default)]
pub struct SystemMetrics {
    pub system_info: SystemInfo,
    pub resource_usage: ResourceUsage,
    pub performance_stats: PerformanceStats,
    pub update_interval: Duration,
    pub last_update: Option<Instant>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SystemInfo {
    pub hostname: String,
    pub os: String,
    pub arch: String,
    pub cpu_cores: u32,
    pub total_memory_gb: f64,
    pub uptime: Duration,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ResourceUsage {
    pub cpu_percent: f64,
    pub memory_percent: f64,
    pub disk_percent: f64,
    pub network_io_mbps: (f64, f64), // (in, out)
    pub disk_io_mbps: (f64, f64),    // (read, write)
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PerformanceStats {
    pub operations_per_second: f64,
    pub average_latency_ms: f64,
    pub error_rate_percent: f64,
    pub cache_hit_rate_percent: f64,
    pub active_connections: u32,
}

/// Console for logs and output
#[derive(Default)]
pub struct Console {
    logs: Vec<LogEntry>,
    selected_log: usize,
    log_list_state: ListState,
    log_level_filter: LogLevel,
    auto_scroll: bool,
    search_input: Input,
    show_search: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogEntry {
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub level: LogLevel,
    pub source: String,
    pub message: String,
    pub metadata: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, PartialOrd)]
pub enum LogLevel {
    Trace,
    Debug,
    Info,
    Warn,
    Error,
}

impl Default for LogLevel {
    fn default() -> Self {
        LogLevel::Info
    }
}

impl App {
    /// Create new TUI application
    pub fn new(client: aerolithsClient) -> Result<Self> {
        let (event_sender, event_receiver) = mpsc::unbounded_channel();
        
        let mut app = Self {
            current_tab: 0,
            should_quit: false,
            node_manager: NodeManager::default(),
            cluster_monitor: ClusterMonitor::default(),
            test_runner: TestRunner::default(),
            config_manager: ConfigManager::default(),
            system_metrics: SystemMetrics::default(),
            console: Console::default(),
            input: Input::default(),
            client: Arc::new(client),
            event_sender,
            event_receiver,
        };
        
        // Initialize test suites
        app.initialize_test_suites();
        
        // Initialize configuration files
        app.initialize_config_files();
        
        // Start background tasks
        app.start_background_tasks()?;
        
        Ok(app)
    }
    
    /// Run the TUI application
    pub async fn run<B: Backend>(&mut self, terminal: &mut Terminal<B>) -> Result<()> {
        loop {
            // Draw the UI
            terminal.draw(|f| self.draw(f))?;
            
            // Handle events with timeout
            if crossterm::event::poll(Duration::from_millis(50))? {
                if let Event::Key(key) = event::read()? {
                    if key.kind == KeyEventKind::Press {
                        self.handle_key_event(key.code).await?;
                    }
                }
            }
            
            // Process background events
            while let Ok(event) = self.event_receiver.try_recv() {
                self.handle_app_event(event).await?;
            }
            
            if self.should_quit {
                break;
            }
        }
        
        Ok(())
    }
    
    /// Draw the main UI
    fn draw<B: Backend>(&mut self, f: &mut Frame<B>) {
        let size = f.size();
        
        // Create main layout
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // Header
                Constraint::Min(0),    // Main content
                Constraint::Length(3), // Footer
            ])
            .split(size);
        
        // Draw header with tabs
        self.draw_header(f, chunks[0]);
        
        // Draw main content based on current tab
        match Tab::ALL[self.current_tab] {
            Tab::Dashboard => self.draw_dashboard(f, chunks[1]),
            Tab::NodeManager => self.draw_node_manager(f, chunks[1]),
            Tab::ClusterMonitor => self.draw_cluster_monitor(f, chunks[1]),
            Tab::TestRunner => self.draw_test_runner(f, chunks[1]),
            Tab::Configuration => self.draw_configuration(f, chunks[1]),
            Tab::Console => self.draw_console(f, chunks[1]),
        }
        
        // Draw footer with status and help
        self.draw_footer(f, chunks[2]);
    }
    
    /// Draw header with navigation tabs
    fn draw_header<B: Backend>(&self, f: &mut Frame<B>, area: Rect) {
        let titles: Vec<Line> = Tab::ALL
            .iter()
            .map(|tab| {
                Line::from(format!("{} {}", tab.icon(), tab.name()))
            })
            .collect();
        
        let tabs = Tabs::new(titles)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("🚀 AerolithDB Control Center")
                    .title_style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
            )
            .style(Style::default().fg(Color::White))
            .highlight_style(
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD)
                    .add_modifier(Modifier::UNDERLINED),
            )
            .select(self.current_tab);
        
        f.render_widget(tabs, area);
    }
    
    /// Draw footer with status and help
    fn draw_footer<B: Backend>(&self, f: &mut Frame<B>, area: Rect) {
        let current_tab = Tab::ALL[self.current_tab];
        let help_text = match current_tab {
            Tab::Dashboard => "Space: Refresh | Tab: Next Tab | q: Quit",
            Tab::NodeManager => "Enter: Start/Stop Node | n: New Node | d: Delete Node | Tab: Next Tab | q: Quit",
            Tab::ClusterMonitor => "Space: Refresh | r: Toggle Auto-refresh | Tab: Next Tab | q: Quit",
            Tab::TestRunner => "Enter: Run Test | Space: View Results | Tab: Next Tab | q: Quit",
            Tab::Configuration => "Enter: Edit Config | v: Validate | s: Save | Tab: Next Tab | q: Quit",
            Tab::Console => "c: Clear | f: Filter | /: Search | Tab: Next Tab | q: Quit",
        };
        
        let footer = Paragraph::new(help_text)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("Help")
                    .title_style(Style::default().fg(Color::Green)),
            )
            .style(Style::default().fg(Color::Gray))
            .alignment(Alignment::Left)
            .wrap(Wrap { trim: true });
        
        f.render_widget(footer, area);
    }
    
    /// Initialize available test suites
    fn initialize_test_suites(&mut self) {
        self.test_runner.available_tests = vec![
            TestSuite {
                name: "Unit Tests".to_string(),
                description: "Complete unit test suite for all modules".to_string(),
                test_type: TestType::Unit,
                estimated_duration: Duration::from_secs(30),
                requirements: vec!["None".to_string()],
            },
            TestSuite {
                name: "Integration Tests".to_string(),
                description: "End-to-end integration testing".to_string(),
                test_type: TestType::Integration,
                estimated_duration: Duration::from_secs(120),
                requirements: vec!["Running node".to_string()],
            },
            TestSuite {
                name: "Battle Test".to_string(),
                description: "Comprehensive distributed operations test".to_string(),
                test_type: TestType::BattleTest,
                estimated_duration: Duration::from_secs(300),
                requirements: vec!["Multi-node cluster".to_string()],
            },
            TestSuite {
                name: "Network Test".to_string(),
                description: "Network partition and recovery testing".to_string(),
                test_type: TestType::NetworkTest,
                estimated_duration: Duration::from_secs(180),
                requirements: vec!["Multi-node cluster".to_string()],
            },
            TestSuite {
                name: "Performance Benchmark".to_string(),
                description: "Performance and load testing".to_string(),
                test_type: TestType::Performance,
                estimated_duration: Duration::from_secs(600),
                requirements: vec!["Optimized build".to_string()],
            },
            TestSuite {
                name: "Security Tests".to_string(),
                description: "Security and authentication testing".to_string(),
                test_type: TestType::SecurityTest,
                estimated_duration: Duration::from_secs(90),
                requirements: vec!["Security configuration".to_string()],
            },
        ];
    }
    
    /// Initialize configuration files
    fn initialize_config_files(&mut self) {
        self.config_manager.config_files = vec![
            ConfigFile {
                name: "node.yaml".to_string(),
                path: "./config/node.yaml".to_string(),
                file_type: ConfigType::Node,
                last_modified: chrono::Utc::now(),
                is_valid: true,
            },
            ConfigFile {
                name: "cluster.yaml".to_string(),
                path: "./config/cluster.yaml".to_string(),
                file_type: ConfigType::Cluster,
                last_modified: chrono::Utc::now(),
                is_valid: true,
            },
            ConfigFile {
                name: "api.yaml".to_string(),
                path: "./config/api.yaml".to_string(),
                file_type: ConfigType::API,
                last_modified: chrono::Utc::now(),
                is_valid: true,
            },
            ConfigFile {
                name: "security.yaml".to_string(),
                path: "./config/security.yaml".to_string(),
                file_type: ConfigType::Security,
                last_modified: chrono::Utc::now(),
                is_valid: false,
            },
        ];
    }
    
    /// Start background monitoring tasks
    fn start_background_tasks(&self) -> Result<()> {
        let sender = self.event_sender.clone();
        let client = Arc::clone(&self.client);
        
        // System metrics collection task
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(5));
            loop {
                interval.tick().await;
                
                // Collect system metrics
                let system_info = SystemInfo {
                    hostname: "localhost".to_string(),
                    os: std::env::consts::OS.to_string(),
                    arch: std::env::consts::ARCH.to_string(),
                    cpu_cores: num_cpus::get() as u32,
                    total_memory_gb: 16.0, // This would be actual system memory
                    uptime: Duration::from_secs(3600),
                };
                
                let _ = sender.send(AppEvent::SystemUpdate(system_info));
            }
        });
        
        // Node status monitoring task
        let sender = self.event_sender.clone();
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(3));
            loop {
                interval.tick().await;
                
                // Mock node status update
                let status = NodeStatus::Running;
                let _ = sender.send(AppEvent::NodeStatusUpdate(status));
            }
        });
        
        Ok(())
    }
    
    /// Handle keyboard input
    async fn handle_key_event(&mut self, key: KeyCode) -> Result<()> {
        match key {
            KeyCode::Char('q') => {
                self.should_quit = true;
            }
            KeyCode::Tab => {
                self.current_tab = (self.current_tab + 1) % Tab::ALL.len();
            }
            KeyCode::BackTab => {
                self.current_tab = if self.current_tab == 0 {
                    Tab::ALL.len() - 1
                } else {
                    self.current_tab - 1
                };
            }
            _ => {
                // Handle tab-specific key events
                match Tab::ALL[self.current_tab] {
                    Tab::NodeManager => self.handle_node_manager_key(key).await?,
                    Tab::TestRunner => self.handle_test_runner_key(key).await?,
                    Tab::Configuration => self.handle_configuration_key(key).await?,
                    Tab::Console => self.handle_console_key(key).await?,
                    _ => {}
                }
            }
        }
        
        Ok(())
    }
    
    /// Handle application events from background tasks
    async fn handle_app_event(&mut self, event: AppEvent) -> Result<()> {
        match event {
            AppEvent::NodeStatusUpdate(status) => {
                // Update node status in the UI
                if let Some(node) = self.node_manager.nodes.get_mut(0) {
                    node.status = status;
                }
            }
            AppEvent::SystemUpdate(info) => {
                self.system_metrics.system_info = info;
                self.system_metrics.last_update = Some(Instant::now());
            }
            AppEvent::LogMessage(log) => {
                self.console.logs.push(log);
                // Keep only last 1000 log entries
                if self.console.logs.len() > 1000 {
                    self.console.logs.remove(0);
                }
            }
            AppEvent::Error(error) => {
                let log = LogEntry {
                    timestamp: chrono::Utc::now(),
                    level: LogLevel::Error,
                    source: "TUI".to_string(),
                    message: error,
                    metadata: HashMap::new(),
                };
                self.console.logs.push(log);
            }
            _ => {}
        }
        
        Ok(())
    }
}

/// Entry point for the TUI application
pub async fn run_tui(client: aerolithsClient) -> Result<()> {
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    
    // Create and run app
    let mut app = App::new(client)?;
    let result = app.run(&mut terminal).await;
    
    // Restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;
    
    result
}
