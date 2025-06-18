//! TUI Application State and Management
//!
//! This module manages the overall application state for the TUI interface,
//! including tab navigation, background tasks, and state synchronization
//! between different components.

use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tokio::sync::mpsc;
use tokio::time::interval;
use anyhow::Result;
use ratatui::widgets::TableState;

use crate::client::aerolithsClient;

/// Main application state for the TUI
#[derive(Clone)]
pub struct App {
    /// Current active tab
    pub current_tab: usize,
    /// Available tabs
    pub tabs: Vec<&'static str>,
    /// Should the application quit
    pub should_quit: bool,
    /// Error message to display
    pub error_message: Option<String>,
    /// Status message to display
    pub status_message: Option<String>,
    /// Dashboard state
    pub dashboard: DashboardState,
    /// Node manager state
    pub node_manager: NodeManagerState,
    /// Cluster monitor state
    pub cluster_monitor: ClusterMonitorState,
    /// Test runner state
    pub test_runner: TestRunnerState,
    /// Configuration state
    pub configuration: ConfigurationState,
    /// Console state
    pub console: ConsoleState,
    /// Background task handles
    pub background_tasks: BackgroundTasks,
}

/// Dashboard tab state
#[derive(Clone)]
pub struct DashboardState {
    /// System metrics
    pub system_metrics: SystemMetrics,
    /// Node status overview
    pub node_overview: Vec<NodeStatus>,
    /// Recent activity logs
    pub recent_activity: Vec<ActivityLog>,
    /// Quick stats
    pub quick_stats: QuickStats,
    /// Last update time
    pub last_updated: Option<Instant>,
}

/// Node manager tab state
#[derive(Clone)]
pub struct NodeManagerState {
    /// List of managed nodes
    pub nodes: Vec<ManagedNode>,
    /// Selected node index
    pub selected_node: Option<usize>,
    /// Table state for node list
    pub table_state: TableState,
    /// Node operation status
    pub operation_status: Option<String>,
    /// Node configuration dialog
    pub config_dialog: Option<NodeConfigDialog>,
}

/// Cluster monitor tab state
#[derive(Clone)]
pub struct ClusterMonitorState {
    /// Cluster topology
    pub topology: ClusterTopology,
    /// Network status
    pub network_status: NetworkStatus,
    /// Replication status
    pub replication_status: ReplicationStatus,
    /// Performance metrics
    pub performance_metrics: PerformanceMetrics,
    /// Alerts and warnings
    pub alerts: Vec<ClusterAlert>,
}

/// Test runner tab state
#[derive(Clone)]
pub struct TestRunnerState {
    /// Available test suites
    pub test_suites: Vec<TestSuite>,
    /// Currently running tests
    pub running_tests: Vec<RunningTest>,
    /// Test results history
    pub test_results: Vec<TestResult>,
    /// Selected test suite
    pub selected_suite: Option<usize>,
    /// Test output console
    pub test_output: Vec<String>,
    /// Test execution status
    pub execution_status: TestExecutionStatus,
}

/// Configuration tab state
#[derive(Clone)]
pub struct ConfigurationState {
    /// Current configuration
    pub current_config: String,
    /// Configuration sections
    pub config_sections: Vec<ConfigSection>,
    /// Selected section
    pub selected_section: Option<usize>,
    /// Configuration validation status
    pub validation_status: Option<ConfigValidationResult>,
    /// Configuration editor state
    pub editor_state: ConfigEditorState,
}

/// Console tab state
#[derive(Clone)]
pub struct ConsoleState {
    /// Console output lines
    pub output: Vec<String>,
    /// Input buffer
    pub input: String,
    /// Command history
    pub history: Vec<String>,
    /// Current history index
    pub history_index: Option<usize>,
    /// Console mode (command/log viewing)
    pub mode: ConsoleMode,
}

/// Background task management
#[derive(Clone)]
pub struct BackgroundTasks {
    /// Metrics update sender
    pub metrics_sender: Option<mpsc::UnboundedSender<MetricsUpdate>>,
    /// Node status update sender
    pub node_status_sender: Option<mpsc::UnboundedSender<NodeStatusUpdate>>,
    /// Log update sender
    pub log_sender: Option<mpsc::UnboundedSender<LogUpdate>>,
}

/// System metrics structure
#[derive(Clone, Debug)]
pub struct SystemMetrics {
    pub cpu_usage: f64,
    pub memory_usage: f64,
    pub disk_usage: f64,
    pub network_io: NetworkIO,
    pub database_stats: DatabaseStats,
}

/// Network I/O metrics
#[derive(Clone, Debug)]
pub struct NetworkIO {
    pub bytes_in: u64,
    pub bytes_out: u64,
    pub packets_in: u64,
    pub packets_out: u64,
}

/// Database statistics
#[derive(Clone, Debug)]
pub struct DatabaseStats {
    pub total_documents: u64,
    pub total_collections: u64,
    pub storage_size: u64,
    pub index_size: u64,
    pub operations_per_second: f64,
}

/// Node status information
#[derive(Clone, Debug)]
pub struct NodeStatus {
    pub id: String,
    pub name: String,
    pub status: String,
    pub health: String,
    pub uptime: Duration,
    pub last_seen: Instant,
}

/// Activity log entry
#[derive(Clone, Debug)]
pub struct ActivityLog {
    pub timestamp: Instant,
    pub level: String,
    pub message: String,
    pub source: String,
}

/// Quick statistics
#[derive(Clone, Debug)]
pub struct QuickStats {
    pub active_nodes: u32,
    pub total_requests: u64,
    pub error_rate: f64,
    pub avg_response_time: Duration,
}

/// Managed node information
#[derive(Clone, Debug)]
pub struct ManagedNode {
    pub id: String,
    pub name: String,
    pub endpoint: String,
    pub status: NodeState,
    pub capabilities: Vec<String>,
    pub configuration: String,
}

/// Node state enumeration
#[derive(Clone, Debug)]
pub enum NodeState {
    Running,
    Stopped,
    Starting,
    Stopping,
    Error(String),
    Unknown,
}

/// Node configuration dialog
#[derive(Clone, Debug)]
pub struct NodeConfigDialog {
    pub node_id: String,
    pub config_text: String,
    pub cursor_position: usize,
    pub is_valid: bool,
    pub validation_errors: Vec<String>,
}

/// Cluster topology information
#[derive(Clone, Debug)]
pub struct ClusterTopology {
    pub nodes: Vec<TopologyNode>,
    pub connections: Vec<NodeConnection>,
    pub partitions: Vec<Partition>,
}

/// Topology node
#[derive(Clone, Debug)]
pub struct TopologyNode {
    pub id: String,
    pub name: String,
    pub role: String,
    pub status: String,
    pub load: f64,
}

/// Node connection
#[derive(Clone, Debug)]
pub struct NodeConnection {
    pub from: String,
    pub to: String,
    pub status: String,
    pub latency: Duration,
}

/// Partition information
#[derive(Clone, Debug)]
pub struct Partition {
    pub id: String,
    pub primary_node: String,
    pub replica_nodes: Vec<String>,
    pub status: String,
}

/// Network status
#[derive(Clone, Debug)]
pub struct NetworkStatus {
    pub total_nodes: u32,
    pub healthy_nodes: u32,
    pub network_partitions: u32,
    pub consensus_status: String,
}

/// Replication status
#[derive(Clone, Debug)]
pub struct ReplicationStatus {
    pub replication_lag: Duration,
    pub sync_status: String,
    pub failed_replications: u32,
}

/// Performance metrics
#[derive(Clone, Debug)]
pub struct PerformanceMetrics {
    pub throughput: f64,
    pub latency_p50: Duration,
    pub latency_p95: Duration,
    pub latency_p99: Duration,
}

/// Cluster alert
#[derive(Clone, Debug)]
pub struct ClusterAlert {
    pub id: String,
    pub level: AlertLevel,
    pub message: String,
    pub timestamp: Instant,
    pub source: String,
}

/// Alert level enumeration
#[derive(Clone, Debug)]
pub enum AlertLevel {
    Info,
    Warning,
    Error,
    Critical,
}

/// Test suite information
#[derive(Clone, Debug)]
pub struct TestSuite {
    pub name: String,
    pub description: String,
    pub tests: Vec<TestCase>,
    pub last_run: Option<Instant>,
    pub last_result: Option<TestSuiteResult>,
}

/// Test case information
#[derive(Clone, Debug)]
pub struct TestCase {
    pub name: String,
    pub description: String,
    pub timeout: Duration,
    pub dependencies: Vec<String>,
}

/// Running test information
#[derive(Clone, Debug)]
pub struct RunningTest {
    pub suite_name: String,
    pub test_name: String,
    pub started_at: Instant,
    pub progress: f64,
    pub current_step: String,
}

/// Test result information
#[derive(Clone, Debug)]
pub struct TestResult {
    pub suite_name: String,
    pub test_name: String,
    pub result: TestResultStatus,
    pub duration: Duration,
    pub message: Option<String>,
    pub timestamp: Instant,
}

/// Test result status
#[derive(Clone, Debug)]
pub enum TestResultStatus {
    Passed,
    Failed,
    Skipped,
    Error,
}

/// Test suite result
#[derive(Clone, Debug)]
pub struct TestSuiteResult {
    pub total_tests: u32,
    pub passed: u32,
    pub failed: u32,
    pub skipped: u32,
    pub duration: Duration,
}

/// Test execution status
#[derive(Clone, Debug)]
pub enum TestExecutionStatus {
    Idle,
    Running { suite: String, progress: f64 },
    Completed { suite: String, result: TestSuiteResult },
    Failed { suite: String, error: String },
}

/// Configuration section
#[derive(Clone, Debug)]
pub struct ConfigSection {
    pub name: String,
    pub description: String,
    pub content: String,
    pub is_modified: bool,
}

/// Configuration validation result
#[derive(Clone, Debug)]
pub struct ConfigValidationResult {
    pub is_valid: bool,
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
}

/// Configuration editor state
#[derive(Clone, Debug)]
pub struct ConfigEditorState {
    pub cursor_line: usize,
    pub cursor_column: usize,
    pub scroll_offset: usize,
    pub is_editing: bool,
}

/// Console mode
#[derive(Clone, Debug)]
pub enum ConsoleMode {
    Command,
    LogViewing,
}

/// Background task update types
#[derive(Clone, Debug)]
pub enum MetricsUpdate {
    SystemMetrics(SystemMetrics),
    DatabaseStats(DatabaseStats),
}

#[derive(Clone, Debug)]
pub enum NodeStatusUpdate {
    NodeAdded(NodeStatus),
    NodeUpdated(NodeStatus),
    NodeRemoved(String),
}

#[derive(Clone, Debug)]
pub enum LogUpdate {
    NewEntry(ActivityLog),
    Clear,
}

impl Default for App {
    fn default() -> Self {
        Self {
            current_tab: 0,
            tabs: vec![
                "Dashboard",
                "Node Manager", 
                "Cluster Monitor",
                "Test Runner",
                "Configuration",
                "Console"
            ],
            should_quit: false,
            error_message: None,
            status_message: None,
            dashboard: DashboardState::default(),
            node_manager: NodeManagerState::default(),
            cluster_monitor: ClusterMonitorState::default(),
            test_runner: TestRunnerState::default(),
            configuration: ConfigurationState::default(),
            console: ConsoleState::default(),
            background_tasks: BackgroundTasks::default(),
        }
    }
}

impl Default for DashboardState {
    fn default() -> Self {
        Self {
            system_metrics: SystemMetrics::default(),
            node_overview: Vec::new(),
            recent_activity: Vec::new(),
            quick_stats: QuickStats::default(),
            last_updated: None,
        }
    }
}

impl Default for NodeManagerState {
    fn default() -> Self {
        let mut state = Self {
            nodes: vec![
                ManagedNode {
                    id: "node-01".to_string(),
                    name: "Primary Node".to_string(),
                    endpoint: "127.0.0.1:8080".to_string(),
                    status: NodeState::Stopped,
                    capabilities: vec!["storage".to_string(), "query".to_string(), "consensus".to_string()],
                    configuration: r#"{"storage_path": "/data/node01", "port": 8080}"#.to_string(),
                },
                ManagedNode {
                    id: "node-02".to_string(),
                    name: "Secondary Node".to_string(),
                    endpoint: "127.0.0.1:8081".to_string(),
                    status: NodeState::Stopped,
                    capabilities: vec!["storage".to_string(), "query".to_string()],
                    configuration: r#"{"storage_path": "/data/node02", "port": 8081}"#.to_string(),
                },
                ManagedNode {
                    id: "node-03".to_string(),
                    name: "Worker Node".to_string(),
                    endpoint: "127.0.0.1:8082".to_string(),
                    status: NodeState::Stopped,
                    capabilities: vec!["query".to_string()],
                    configuration: r#"{"storage_path": "/data/node03", "port": 8082}"#.to_string(),
                },
            ],
            selected_node: Some(0),
            table_state: {
                let mut state = TableState::default();
                state.select(Some(0));
                state
            },
            operation_status: None,
            config_dialog: None,
        };
        state
    }
}

impl Default for ClusterMonitorState {
    fn default() -> Self {
        Self {
            topology: ClusterTopology::default(),
            network_status: NetworkStatus::default(),
            replication_status: ReplicationStatus::default(),
            performance_metrics: PerformanceMetrics::default(),
            alerts: Vec::new(),
        }
    }
}

impl Default for TestRunnerState {
    fn default() -> Self {
        Self {
            test_suites: Vec::new(),
            running_tests: Vec::new(),
            test_results: Vec::new(),
            selected_suite: None,
            test_output: Vec::new(),
            execution_status: TestExecutionStatus::Idle,
        }
    }
}

impl Default for ConfigurationState {
    fn default() -> Self {
        Self {
            current_config: String::new(),
            config_sections: Vec::new(),
            selected_section: None,
            validation_status: None,
            editor_state: ConfigEditorState::default(),
        }
    }
}

impl Default for ConsoleState {
    fn default() -> Self {
        Self {
            output: Vec::new(),
            input: String::new(),
            history: Vec::new(),
            history_index: None,
            mode: ConsoleMode::Command,
        }
    }
}

impl Default for BackgroundTasks {
    fn default() -> Self {
        Self {
            metrics_sender: None,
            node_status_sender: None,
            log_sender: None,
        }
    }
}

impl Default for SystemMetrics {
    fn default() -> Self {
        Self {
            cpu_usage: 0.0,
            memory_usage: 0.0,
            disk_usage: 0.0,
            network_io: NetworkIO::default(),
            database_stats: DatabaseStats::default(),
        }
    }
}

impl Default for NetworkIO {
    fn default() -> Self {
        Self {
            bytes_in: 0,
            bytes_out: 0,
            packets_in: 0,
            packets_out: 0,
        }
    }
}

impl Default for DatabaseStats {
    fn default() -> Self {
        Self {
            total_documents: 0,
            total_collections: 0,
            storage_size: 0,
            index_size: 0,
            operations_per_second: 0.0,
        }
    }
}

impl Default for QuickStats {
    fn default() -> Self {
        Self {
            active_nodes: 0,
            total_requests: 0,
            error_rate: 0.0,
            avg_response_time: Duration::from_millis(0),
        }
    }
}

impl Default for ClusterTopology {
    fn default() -> Self {
        Self {
            nodes: Vec::new(),
            connections: Vec::new(),
            partitions: Vec::new(),
        }
    }
}

impl Default for NetworkStatus {
    fn default() -> Self {
        Self {
            total_nodes: 0,
            healthy_nodes: 0,
            network_partitions: 0,
            consensus_status: "Unknown".to_string(),
        }
    }
}

impl Default for ReplicationStatus {
    fn default() -> Self {
        Self {
            replication_lag: Duration::from_millis(0),
            sync_status: "Unknown".to_string(),
            failed_replications: 0,
        }
    }
}

impl Default for PerformanceMetrics {
    fn default() -> Self {
        Self {
            throughput: 0.0,
            latency_p50: Duration::from_millis(0),
            latency_p95: Duration::from_millis(0),
            latency_p99: Duration::from_millis(0),
        }
    }
}

impl Default for ConfigEditorState {
    fn default() -> Self {
        Self {
            cursor_line: 0,
            cursor_column: 0,
            scroll_offset: 0,
            is_editing: false,
        }
    }
}

impl App {
    /// Create new application instance
    pub fn new() -> Self {
        Self::default()
    }

    /// Navigate to next tab
    pub fn next_tab(&mut self) {
        self.current_tab = (self.current_tab + 1) % self.tabs.len();
    }

    /// Navigate to previous tab
    pub fn previous_tab(&mut self) {
        if self.current_tab > 0 {
            self.current_tab -= 1;
        } else {
            self.current_tab = self.tabs.len() - 1;
        }
    }

    /// Set error message
    pub fn set_error(&mut self, message: String) {
        self.error_message = Some(message);
    }

    /// Clear error message
    pub fn clear_error(&mut self) {
        self.error_message = None;
    }

    /// Set status message
    pub fn set_status(&mut self, message: String) {
        self.status_message = Some(message);
    }

    /// Clear status message
    pub fn clear_status(&mut self) {
        self.status_message = None;
    }

    /// Request application quit
    pub fn quit(&mut self) {
        self.should_quit = true;
    }

    /// Get current tab name
    pub fn current_tab_name(&self) -> &'static str {
        self.tabs[self.current_tab]
    }

    /// Start background tasks
    pub async fn start_background_tasks(&mut self, client: Arc<aerolithsClient>) -> Result<()> {
        // Create channels for background task communication
        let (metrics_tx, mut metrics_rx) = mpsc::unbounded_channel();
        let (node_status_tx, mut node_status_rx) = mpsc::unbounded_channel();
        let (log_tx, mut log_rx) = mpsc::unbounded_channel();

        self.background_tasks.metrics_sender = Some(metrics_tx.clone());
        self.background_tasks.node_status_sender = Some(node_status_tx.clone());
        self.background_tasks.log_sender = Some(log_tx.clone());

        // Start metrics collection task
        let client_metrics = client.clone();
        tokio::spawn(async move {
            let mut interval = interval(Duration::from_secs(5));
            loop {
                interval.tick().await;
                
                // Collect system metrics (placeholder implementation)
                let metrics = SystemMetrics {
                    cpu_usage: rand::random::<f64>() * 100.0,
                    memory_usage: rand::random::<f64>() * 100.0,
                    disk_usage: rand::random::<f64>() * 100.0,
                    network_io: NetworkIO {
                        bytes_in: rand::random::<u64>() % 1000000,
                        bytes_out: rand::random::<u64>() % 1000000,
                        packets_in: rand::random::<u64>() % 10000,
                        packets_out: rand::random::<u64>() % 10000,
                    },
                    database_stats: DatabaseStats {
                        total_documents: rand::random::<u64>() % 1000000,
                        total_collections: rand::random::<u64>() % 100,
                        storage_size: rand::random::<u64>() % 10000000000,
                        index_size: rand::random::<u64>() % 1000000000,
                        operations_per_second: rand::random::<f64>() * 1000.0,
                    },
                };

                if metrics_tx.send(MetricsUpdate::SystemMetrics(metrics)).is_err() {
                    break;
                }
            }
        });

        // Start node status monitoring task
        let client_nodes = client.clone();
        tokio::spawn(async move {
            let mut interval = interval(Duration::from_secs(10));
            loop {
                interval.tick().await;
                
                // Monitor node status (placeholder implementation)
                let node_status = NodeStatus {
                    id: "node-1".to_string(),
                    name: "Primary Node".to_string(),
                    status: "Running".to_string(),
                    health: "Healthy".to_string(),
                    uptime: Duration::from_secs(rand::random::<u64>() % 86400),
                    last_seen: Instant::now(),
                };

                if node_status_tx.send(NodeStatusUpdate::NodeUpdated(node_status)).is_err() {
                    break;
                }
            }
        });

        // Start log collection task
        tokio::spawn(async move {
            let mut interval = interval(Duration::from_secs(2));
            loop {
                interval.tick().await;
                
                // Collect logs (placeholder implementation)
                let log_entry = ActivityLog {
                    timestamp: Instant::now(),
                    level: ["INFO", "WARN", "ERROR"][rand::random::<usize>() % 3].to_string(),
                    message: "Sample log message".to_string(),
                    source: "AerolithDB".to_string(),
                };

                if log_tx.send(LogUpdate::NewEntry(log_entry)).is_err() {
                    break;
                }
            }
        });

        Ok(())
    }
}

impl std::fmt::Display for NodeState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            NodeState::Running => write!(f, "Running"),
            NodeState::Stopped => write!(f, "Stopped"),
            NodeState::Starting => write!(f, "Starting"),
            NodeState::Stopping => write!(f, "Stopping"),
            NodeState::Error(msg) => write!(f, "Error: {}", msg),
            NodeState::Unknown => write!(f, "Unknown"),
        }
    }
}

impl std::fmt::Display for AlertLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AlertLevel::Info => write!(f, "INFO"),
            AlertLevel::Warning => write!(f, "WARN"),
            AlertLevel::Error => write!(f, "ERROR"),
            AlertLevel::Critical => write!(f, "CRITICAL"),
        }
    }
}

impl std::fmt::Display for TestResultStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TestResultStatus::Passed => write!(f, "PASSED"),
            TestResultStatus::Failed => write!(f, "FAILED"),
            TestResultStatus::Skipped => write!(f, "SKIPPED"),
            TestResultStatus::Error => write!(f, "ERROR"),
        }
    }
}
