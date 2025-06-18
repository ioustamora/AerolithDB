//! TUI User Interface Components
//!
//! This module contains the rendering logic for all TUI components,
//! including tab navigation, status bars, and individual tab content.

use ratatui::{
    backend::Backend,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style, Stylize},
    symbols::DOT,
    text::{Line, Span, Text},
    widgets::{
        Block, Borders, Cell, Clear, Gauge, List, ListItem, Paragraph, Row, Table, Tabs,
        Wrap, canvas::{Canvas, Map, MapResolution, Rectangle},
    },
    Frame,
};
use std::time::{Duration, Instant};

use super::app::{App, AlertLevel, NodeState, TestResultStatus, TestExecutionStatus, ConsoleMode};

/// Render the complete TUI interface
pub fn render(f: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Tab bar
            Constraint::Min(0),    // Main content
            Constraint::Length(3), // Status bar
        ])
        .split(f.area());

    // Render tab bar
    render_tabs(f, app, chunks[0]);

    // Render main content based on current tab
    match app.current_tab {
        0 => render_dashboard(f, app, chunks[1]),
        1 => render_node_manager(f, app, chunks[1]),
        2 => render_cluster_monitor(f, app, chunks[1]),
        3 => render_test_runner(f, app, chunks[1]),
        4 => render_configuration(f, app, chunks[1]),
        5 => render_console(f, app, chunks[1]),
        _ => render_dashboard(f, app, chunks[1]),
    }

    // Render status bar
    render_status_bar(f, app, chunks[2]);

    // Render error/status overlays if needed
    if app.error_message.is_some() {
        render_error_overlay(f, app);
    }
}

/// Render the tab navigation bar
fn render_tabs(f: &mut Frame, app: &App, area: Rect) {
    let tabs = Tabs::new(app.tabs.iter().map(|&tab| tab).collect::<Vec<_>>())
        .block(
            Block::default()
                .title("AerolithDB TUI - Terminal User Interface")
                .title_style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Gray)),
        )
        .select(app.current_tab)
        .style(Style::default().fg(Color::White))
        .highlight_style(
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD)
                .bg(Color::DarkGray),
        );

    f.render_widget(tabs, area);
}

/// Render the status bar at the bottom
fn render_status_bar(f: &mut Frame, app: &App, area: Rect) {
    let status_text = if let Some(ref error) = app.error_message {
        format!("ERROR: {}", error)
    } else if let Some(ref status) = app.status_message {
        status.clone()
    } else {
        format!(
            "Tab: {} | Press 'Tab' to switch tabs, 'q' to quit, 'h' for help",
            app.current_tab_name()
        )
    };

    let status_style = if app.error_message.is_some() {
        Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)
    } else if app.status_message.is_some() {
        Style::default().fg(Color::Green)
    } else {
        Style::default().fg(Color::Gray)
    };

    let status_bar = Paragraph::new(status_text)
        .style(status_style)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Gray)),
        )
        .wrap(Wrap { trim: true });

    f.render_widget(status_bar, area);
}

/// Render the dashboard tab
fn render_dashboard(f: &mut Frame, app: &App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(8),  // System metrics
            Constraint::Length(8),  // Quick stats
            Constraint::Min(0),     // Activity log
        ])
        .split(area);

    // System metrics section
    render_system_metrics(f, &app.dashboard, chunks[0]);

    // Quick stats section
    render_quick_stats(f, &app.dashboard, chunks[1]);

    // Activity log section
    render_activity_log(f, &app.dashboard, chunks[2]);
}

/// Render system metrics
fn render_system_metrics(f: &mut Frame, dashboard: &super::app::DashboardState, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(25),
            Constraint::Percentage(25),
            Constraint::Percentage(25),
            Constraint::Percentage(25),
        ])
        .split(area);

    // CPU Usage
    let cpu_gauge = Gauge::default()
        .block(
            Block::default()
                .title("CPU Usage")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Blue)),
        )
        .gauge_style(Style::default().fg(Color::Cyan))
        .percent(dashboard.system_metrics.cpu_usage as u16)
        .label(format!("{:.1}%", dashboard.system_metrics.cpu_usage));

    f.render_widget(cpu_gauge, chunks[0]);

    // Memory Usage
    let memory_gauge = Gauge::default()
        .block(
            Block::default()
                .title("Memory Usage")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Blue)),
        )
        .gauge_style(Style::default().fg(Color::Green))
        .percent(dashboard.system_metrics.memory_usage as u16)
        .label(format!("{:.1}%", dashboard.system_metrics.memory_usage));

    f.render_widget(memory_gauge, chunks[1]);

    // Disk Usage
    let disk_gauge = Gauge::default()
        .block(
            Block::default()
                .title("Disk Usage")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Blue)),
        )
        .gauge_style(Style::default().fg(Color::Yellow))
        .percent(dashboard.system_metrics.disk_usage as u16)
        .label(format!("{:.1}%", dashboard.system_metrics.disk_usage));

    f.render_widget(disk_gauge, chunks[2]);

    // Database Stats
    let db_info = format!(
        "Documents: {}\nCollections: {}\nOps/sec: {:.1}",
        dashboard.system_metrics.database_stats.total_documents,
        dashboard.system_metrics.database_stats.total_collections,
        dashboard.system_metrics.database_stats.operations_per_second
    );

    let db_stats = Paragraph::new(db_info)
        .block(
            Block::default()
                .title("Database Stats")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Blue)),
        )
        .style(Style::default().fg(Color::White));

    f.render_widget(db_stats, chunks[3]);
}

/// Render quick stats
fn render_quick_stats(f: &mut Frame, dashboard: &super::app::DashboardState, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(25),
            Constraint::Percentage(25),
            Constraint::Percentage(25),
            Constraint::Percentage(25),
        ])
        .split(area);

    // Active Nodes
    let nodes_info = format!("{}", dashboard.quick_stats.active_nodes);
    let nodes_widget = Paragraph::new(nodes_info)
        .block(
            Block::default()
                .title("Active Nodes")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Green)),
        )
        .style(Style::default().fg(Color::Green).add_modifier(Modifier::BOLD))
        .alignment(Alignment::Center);

    f.render_widget(nodes_widget, chunks[0]);

    // Total Requests
    let requests_info = format!("{}", dashboard.quick_stats.total_requests);
    let requests_widget = Paragraph::new(requests_info)
        .block(
            Block::default()
                .title("Total Requests")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Blue)),
        )
        .style(Style::default().fg(Color::Blue).add_modifier(Modifier::BOLD))
        .alignment(Alignment::Center);

    f.render_widget(requests_widget, chunks[1]);

    // Error Rate
    let error_info = format!("{:.2}%", dashboard.quick_stats.error_rate);
    let error_style = if dashboard.quick_stats.error_rate > 5.0 {
        Style::default().fg(Color::Red)
    } else if dashboard.quick_stats.error_rate > 1.0 {
        Style::default().fg(Color::Yellow)
    } else {
        Style::default().fg(Color::Green)
    };

    let error_widget = Paragraph::new(error_info)
        .block(
            Block::default()
                .title("Error Rate")
                .borders(Borders::ALL)
                .border_style(error_style),
        )
        .style(error_style.add_modifier(Modifier::BOLD))
        .alignment(Alignment::Center);

    f.render_widget(error_widget, chunks[2]);

    // Avg Response Time
    let response_info = format!("{}ms", dashboard.quick_stats.avg_response_time.as_millis());
    let response_widget = Paragraph::new(response_info)
        .block(
            Block::default()
                .title("Avg Response")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Cyan)),
        )
        .style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
        .alignment(Alignment::Center);

    f.render_widget(response_widget, chunks[3]);
}

/// Render activity log
fn render_activity_log(f: &mut Frame, dashboard: &super::app::DashboardState, area: Rect) {
    let log_items: Vec<ListItem> = dashboard
        .recent_activity
        .iter()
        .map(|activity| {
            let style = match activity.level.as_str() {
                "ERROR" => Style::default().fg(Color::Red),
                "WARN" => Style::default().fg(Color::Yellow),
                "INFO" => Style::default().fg(Color::Green),
                _ => Style::default().fg(Color::White),
            };

            let content = format!(
                "[{}] {}: {}",
                activity.level,
                activity.source,
                activity.message
            );

            ListItem::new(content).style(style)
        })
        .collect();

    let activity_list = List::new(log_items)
        .block(
            Block::default()
                .title("Recent Activity")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::White)),
        )
        .style(Style::default().fg(Color::White));

    f.render_widget(activity_list, area);
}

/// Render the node manager tab
fn render_node_manager(f: &mut Frame, app: &App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(60),  // Node list
            Constraint::Percentage(40),  // Node details/actions
        ])
        .split(area);

    // Node list
    render_node_list(f, &app.node_manager, chunks[0]);

    // Node details and actions
    render_node_details(f, &app.node_manager, chunks[1]);
}

/// Render node list
fn render_node_list(f: &mut Frame, node_manager: &super::app::NodeManagerState, area: Rect) {
    let header = Row::new(vec!["Name", "ID", "Endpoint", "Status", "Capabilities"])
        .style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))
        .height(1);

    let rows: Vec<Row> = node_manager
        .nodes
        .iter()
        .map(|node| {
            let status_style = match node.status {
                NodeState::Running => Style::default().fg(Color::Green),
                NodeState::Stopped => Style::default().fg(Color::Red),
                NodeState::Starting | NodeState::Stopping => Style::default().fg(Color::Yellow),
                NodeState::Error(_) => Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
                NodeState::Unknown => Style::default().fg(Color::Gray),
            };

            Row::new(vec![
                Cell::from(node.name.clone()),
                Cell::from(node.id.clone()),
                Cell::from(node.endpoint.clone()),
                Cell::from(node.status.to_string()).style(status_style),
                Cell::from(node.capabilities.join(", ")),
            ])
        })
        .collect();

    let table = Table::new(rows, [
        Constraint::Percentage(20), // Name
        Constraint::Percentage(15), // Status
        Constraint::Percentage(15), // Port
        Constraint::Percentage(25), // Address
        Constraint::Percentage(25), // Last Seen
    ])
        .header(header)
        .block(
            Block::default()
                .title("Managed Nodes")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::White)),
        )
        .widths(&[
            Constraint::Length(15),
            Constraint::Length(10),
            Constraint::Length(20),
            Constraint::Length(10),
            Constraint::Min(20),
        ])
        .highlight_style(Style::default().bg(Color::DarkGray));

    f.render_stateful_widget(table, area, &mut node_manager.table_state.clone());
}

/// Render node details and actions
fn render_node_details(f: &mut Frame, node_manager: &super::app::NodeManagerState, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(8),   // Node details
            Constraint::Length(6),   // Quick actions
            Constraint::Min(0),      // Configuration/logs
        ])
        .split(area);

    // Node details
    let details_text = if let Some(selected) = node_manager.selected_node {
        if let Some(node) = node_manager.nodes.get(selected) {
            format!(
                "Name: {}\nID: {}\nEndpoint: {}\nStatus: {}\nCapabilities: {}",
                node.name,
                node.id,
                node.endpoint,
                node.status,
                node.capabilities.join(", ")
            )
        } else {
            "No node selected".to_string()
        }
    } else {
        "Select a node from the list".to_string()
    };

    let details_widget = Paragraph::new(details_text)
        .block(
            Block::default()
                .title("Node Details")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Blue)),
        )
        .style(Style::default().fg(Color::White))
        .wrap(Wrap { trim: true });

    f.render_widget(details_widget, chunks[0]);

    // Quick actions
    let actions_text = "Actions:\n[S] Start Node\n[T] Stop Node\n[R] Restart Node\n[C] Configure";
    let actions_widget = Paragraph::new(actions_text)
        .block(
            Block::default()
                .title("Quick Actions")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Green)),
        )
        .style(Style::default().fg(Color::White));

    f.render_widget(actions_widget, chunks[1]);

    // Status or configuration    let default_status = "Ready for operations".to_string();
    let status_text = node_manager
        .operation_status
        .as_ref()
        .unwrap_or(&default_status);

    let status_widget = Paragraph::new(status_text.clone())
        .block(
            Block::default()
                .title("Operation Status")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Gray)),
        )
        .style(Style::default().fg(Color::White))
        .wrap(Wrap { trim: true });

    f.render_widget(status_widget, chunks[2]);
}

/// Render the cluster monitor tab
fn render_cluster_monitor(f: &mut Frame, app: &App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(8),   // Cluster overview
            Constraint::Length(8),   // Performance metrics
            Constraint::Min(0),      // Alerts and topology
        ])
        .split(area);

    // Cluster overview
    render_cluster_overview(f, &app.cluster_monitor, chunks[0]);

    // Performance metrics
    render_performance_metrics(f, &app.cluster_monitor, chunks[1]);

    // Alerts and topology
    render_alerts_and_topology(f, &app.cluster_monitor, chunks[2]);
}

/// Render cluster overview
fn render_cluster_overview(f: &mut Frame, cluster_monitor: &super::app::ClusterMonitorState, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(33),
            Constraint::Percentage(33),
            Constraint::Percentage(34),
        ])
        .split(area);

    // Network status
    let network_text = format!(
        "Total Nodes: {}\nHealthy Nodes: {}\nPartitions: {}\nConsensus: {}",
        cluster_monitor.network_status.total_nodes,
        cluster_monitor.network_status.healthy_nodes,
        cluster_monitor.network_status.network_partitions,
        cluster_monitor.network_status.consensus_status
    );

    let network_widget = Paragraph::new(network_text)
        .block(
            Block::default()
                .title("Network Status")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Blue)),
        )
        .style(Style::default().fg(Color::White));

    f.render_widget(network_widget, chunks[0]);

    // Replication status
    let replication_text = format!(
        "Replication Lag: {}ms\nSync Status: {}\nFailed Replications: {}",
        cluster_monitor.replication_status.replication_lag.as_millis(),
        cluster_monitor.replication_status.sync_status,
        cluster_monitor.replication_status.failed_replications
    );

    let replication_widget = Paragraph::new(replication_text)
        .block(
            Block::default()
                .title("Replication Status")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Green)),
        )
        .style(Style::default().fg(Color::White));

    f.render_widget(replication_widget, chunks[1]);

    // Topology summary
    let topology_text = format!(
        "Topology Nodes: {}\nConnections: {}\nPartitions: {}",
        cluster_monitor.topology.nodes.len(),
        cluster_monitor.topology.connections.len(),
        cluster_monitor.topology.partitions.len()
    );

    let topology_widget = Paragraph::new(topology_text)
        .block(
            Block::default()
                .title("Topology")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Cyan)),
        )
        .style(Style::default().fg(Color::White));

    f.render_widget(topology_widget, chunks[2]);
}

/// Render performance metrics
fn render_performance_metrics(f: &mut Frame, cluster_monitor: &super::app::ClusterMonitorState, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(25),
            Constraint::Percentage(25),
            Constraint::Percentage(25),
            Constraint::Percentage(25),
        ])
        .split(area);

    // Throughput
    let throughput_text = format!("{:.1} ops/s", cluster_monitor.performance_metrics.throughput);
    let throughput_widget = Paragraph::new(throughput_text)
        .block(
            Block::default()
                .title("Throughput")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Green)),
        )
        .style(Style::default().fg(Color::Green).add_modifier(Modifier::BOLD))
        .alignment(Alignment::Center);

    f.render_widget(throughput_widget, chunks[0]);

    // P50 Latency
    let p50_text = format!("{}ms", cluster_monitor.performance_metrics.latency_p50.as_millis());
    let p50_widget = Paragraph::new(p50_text)
        .block(
            Block::default()
                .title("P50 Latency")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Blue)),
        )
        .style(Style::default().fg(Color::Blue).add_modifier(Modifier::BOLD))
        .alignment(Alignment::Center);

    f.render_widget(p50_widget, chunks[1]);

    // P95 Latency
    let p95_text = format!("{}ms", cluster_monitor.performance_metrics.latency_p95.as_millis());
    let p95_widget = Paragraph::new(p95_text)
        .block(
            Block::default()
                .title("P95 Latency")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Yellow)),
        )
        .style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))
        .alignment(Alignment::Center);

    f.render_widget(p95_widget, chunks[2]);

    // P99 Latency
    let p99_text = format!("{}ms", cluster_monitor.performance_metrics.latency_p99.as_millis());
    let p99_widget = Paragraph::new(p99_text)
        .block(
            Block::default()
                .title("P99 Latency")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Red)),
        )
        .style(Style::default().fg(Color::Red).add_modifier(Modifier::BOLD))
        .alignment(Alignment::Center);

    f.render_widget(p99_widget, chunks[3]);
}

/// Render alerts and topology
fn render_alerts_and_topology(f: &mut Frame, cluster_monitor: &super::app::ClusterMonitorState, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(50),
            Constraint::Percentage(50),
        ])
        .split(area);

    // Alerts
    let alert_items: Vec<ListItem> = cluster_monitor
        .alerts
        .iter()
        .map(|alert| {
            let style = match alert.level {
                AlertLevel::Critical => Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
                AlertLevel::Error => Style::default().fg(Color::Red),
                AlertLevel::Warning => Style::default().fg(Color::Yellow),
                AlertLevel::Info => Style::default().fg(Color::Blue),
            };

            let content = format!("[{}] {}: {}", alert.level, alert.source, alert.message);
            ListItem::new(content).style(style)
        })
        .collect();

    let alerts_list = List::new(alert_items)
        .block(
            Block::default()
                .title("Cluster Alerts")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Red)),
        )
        .style(Style::default().fg(Color::White));

    f.render_widget(alerts_list, chunks[0]);

    // Topology nodes
    let node_items: Vec<ListItem> = cluster_monitor
        .topology
        .nodes
        .iter()
        .map(|node| {
            let content = format!("{} ({}) - Load: {:.1}%", node.name, node.role, node.load * 100.0);
            ListItem::new(content)
        })
        .collect();

    let topology_list = List::new(node_items)
        .block(
            Block::default()
                .title("Topology Nodes")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Green)),
        )
        .style(Style::default().fg(Color::White));

    f.render_widget(topology_list, chunks[1]);
}

/// Render the test runner tab
fn render_test_runner(f: &mut Frame, app: &App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(30),  // Test suites
            Constraint::Percentage(70),  // Test execution and results
        ])
        .split(area);

    // Test suites
    render_test_suites(f, &app.test_runner, chunks[0]);

    // Test execution area
    render_test_execution(f, &app.test_runner, chunks[1]);
}

/// Render test suites
fn render_test_suites(f: &mut Frame, test_runner: &super::app::TestRunnerState, area: Rect) {
    let suite_items: Vec<ListItem> = test_runner
        .test_suites
        .iter()
        .map(|suite| {
            let status_indicator = match &suite.last_result {
                Some(result) => {
                    if result.failed > 0 {
                        "❌"
                    } else {
                        "✅"
                    }
                },
                None => "⚪",
            };

            let content = format!("{} {}", status_indicator, suite.name);
            ListItem::new(content)
        })
        .collect();

    let suites_list = List::new(suite_items)
        .block(
            Block::default()
                .title("Test Suites")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Blue)),
        )
        .style(Style::default().fg(Color::White))
        .highlight_style(Style::default().bg(Color::DarkGray));

    f.render_widget(suites_list, area);
}

/// Render test execution area
fn render_test_execution(f: &mut Frame, test_runner: &super::app::TestRunnerState, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(6),   // Execution status
            Constraint::Length(8),   // Running tests
            Constraint::Min(0),      // Test output/results
        ])
        .split(area);

    // Execution status
    render_test_execution_status(f, test_runner, chunks[0]);

    // Running tests
    render_running_tests(f, test_runner, chunks[1]);

    // Test output
    render_test_output(f, test_runner, chunks[2]);
}

/// Render test execution status
fn render_test_execution_status(f: &mut Frame, test_runner: &super::app::TestRunnerState, area: Rect) {
    let (status_text, style) = match &test_runner.execution_status {
        TestExecutionStatus::Idle => ("Ready to run tests".to_string(), Style::default().fg(Color::Green)),
        TestExecutionStatus::Running { suite, progress } => {
            (format!("Running: {} ({:.1}%)", suite, progress * 100.0), Style::default().fg(Color::Yellow))
        },
        TestExecutionStatus::Completed { suite, result } => {
            (format!("Completed: {} - {}/{} passed", suite, result.passed, result.total_tests), Style::default().fg(Color::Green))
        },
        TestExecutionStatus::Failed { suite, error } => {
            (format!("Failed: {} - {}", suite, error), Style::default().fg(Color::Red))
        },
    };

    let status_widget = Paragraph::new(status_text)
        .block(
            Block::default()
                .title("Test Execution Status")
                .borders(Borders::ALL)
                .border_style(style),
        )
        .style(style)
        .alignment(Alignment::Center);

    f.render_widget(status_widget, area);
}

/// Render running tests
fn render_running_tests(f: &mut Frame, test_runner: &super::app::TestRunnerState, area: Rect) {
    let running_items: Vec<ListItem> = test_runner
        .running_tests
        .iter()
        .map(|test| {
            let elapsed = test.started_at.elapsed();
            let content = format!(
                "{}.{} - {} ({:.1}%) - {}s",
                test.suite_name,
                test.test_name,
                test.current_step,
                test.progress * 100.0,
                elapsed.as_secs()
            );
            ListItem::new(content)
        })
        .collect();

    let running_list = List::new(running_items)
        .block(
            Block::default()
                .title("Running Tests")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Yellow)),
        )
        .style(Style::default().fg(Color::White));

    f.render_widget(running_list, area);
}

/// Render test output
fn render_test_output(f: &mut Frame, test_runner: &super::app::TestRunnerState, area: Rect) {
    let output_text = test_runner.test_output.join("\n");

    let output_widget = Paragraph::new(output_text)
        .block(
            Block::default()
                .title("Test Output")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::White)),
        )
        .style(Style::default().fg(Color::White))
        .wrap(Wrap { trim: true });

    f.render_widget(output_widget, area);
}

/// Render the configuration tab
fn render_configuration(f: &mut Frame, app: &App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(30),  // Config sections
            Constraint::Percentage(70),  // Config editor
        ])
        .split(area);

    // Configuration sections
    render_config_sections(f, &app.configuration, chunks[0]);

    // Configuration editor
    render_config_editor(f, &app.configuration, chunks[1]);
}

/// Render configuration sections
fn render_config_sections(f: &mut Frame, configuration: &super::app::ConfigurationState, area: Rect) {
    let section_items: Vec<ListItem> = configuration
        .config_sections
        .iter()
        .map(|section| {
            let indicator = if section.is_modified { "*" } else { " " };
            let content = format!("{}{}", indicator, section.name);
            ListItem::new(content)
        })
        .collect();

    let sections_list = List::new(section_items)
        .block(
            Block::default()
                .title("Configuration Sections")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Blue)),
        )
        .style(Style::default().fg(Color::White))
        .highlight_style(Style::default().bg(Color::DarkGray));

    f.render_widget(sections_list, area);
}

/// Render configuration editor
fn render_config_editor(f: &mut Frame, configuration: &super::app::ConfigurationState, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(0),      // Editor content
            Constraint::Length(3),   // Validation status
        ])
        .split(area);

    // Configuration content
    let config_widget = Paragraph::new(configuration.current_config.clone())
        .block(
            Block::default()
                .title("Configuration Editor")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Green)),
        )
        .style(Style::default().fg(Color::White))
        .wrap(Wrap { trim: false });

    f.render_widget(config_widget, chunks[0]);

    // Validation status
    let validation_text = match &configuration.validation_status {
        Some(result) => {
            if result.is_valid {
                "✅ Configuration is valid".to_string()
            } else {
                format!("❌ {} errors, {} warnings", result.errors.len(), result.warnings.len())
            }
        },
        None => "Configuration not validated".to_string(),
    };

    let validation_widget = Paragraph::new(validation_text)
        .block(
            Block::default()
                .title("Validation Status")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Gray)),
        )
        .style(Style::default().fg(Color::White));

    f.render_widget(validation_widget, chunks[1]);
}

/// Render the console tab
fn render_console(f: &mut Frame, app: &App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(0),      // Console output
            Constraint::Length(3),   // Input line
        ])
        .split(area);

    // Console output
    render_console_output(f, &app.console, chunks[0]);

    // Input line
    render_console_input(f, &app.console, chunks[1]);
}

/// Render console output
fn render_console_output(f: &mut Frame, console: &super::app::ConsoleState, area: Rect) {
    let output_text = console.output.join("\n");

    let output_widget = Paragraph::new(output_text)
        .block(
            Block::default()
                .title("Console Output")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::White)),
        )
        .style(Style::default().fg(Color::Green))
        .wrap(Wrap { trim: false });

    f.render_widget(output_widget, area);
}

/// Render console input
fn render_console_input(f: &mut Frame, console: &super::app::ConsoleState, area: Rect) {
    let input_widget = Paragraph::new(console.input.clone())
        .block(
            Block::default()
                .title("Command Input")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Yellow)),
        )
        .style(Style::default().fg(Color::White));

    f.render_widget(input_widget, area);
}

/// Render error overlay
fn render_error_overlay(f: &mut Frame, app: &App) {
    if let Some(ref error_msg) = app.error_message {
        let area = centered_rect(60, 20, f.area());

        f.render_widget(Clear, area);

        let error_widget = Paragraph::new(error_msg.clone())
            .block(
                Block::default()
                    .title("Error")
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::Red)),
            )
            .style(Style::default().fg(Color::Red))
            .wrap(Wrap { trim: true })
            .alignment(Alignment::Center);

        f.render_widget(error_widget, area);
    }
}

/// Helper function to center a rectangle
fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}
