//! TUI Event Handling
//!
//! This module handles all user input events and state transitions
//! for the TUI interface, including keyboard input, mouse events,
//! and timer-based updates.

use crossterm::event::{KeyCode, KeyModifiers, KeyEvent};
use anyhow::Result;
use std::sync::Arc;

use crate::client::aerolithsClient;
use super::app::{App, ConsoleMode, TestExecutionStatus, NodeState};

/// Handle keyboard input events
pub async fn handle_key_event(app: &mut App, key: KeyEvent, client: Arc<aerolithsClient>) -> Result<()> {    match key.code {
        // Global navigation
        KeyCode::Char('q') | KeyCode::Char('Q') => {
            app.quit();
            return Ok(());
        },
        KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            app.quit();
            return Ok(());
        },
        KeyCode::Tab => {
            app.next_tab();
        },
        KeyCode::BackTab => {
            app.previous_tab();
        },
        KeyCode::Esc => {
            app.clear_error();
            app.clear_status();
        },
        KeyCode::Char('h') | KeyCode::F(1) => {
            show_help(app);
        },
        KeyCode::F(5) => {
            refresh_data(app, client.clone()).await?;
        },
        _ => {
            // Handle tab-specific events
            match app.current_tab {
                0 => handle_dashboard_events(app, key, client).await?,
                1 => handle_node_manager_events(app, key, client).await?,
                2 => handle_cluster_monitor_events(app, key, client).await?,
                3 => handle_test_runner_events(app, key, client).await?,
                4 => handle_configuration_events(app, key, client).await?,
                5 => handle_console_events(app, key, client).await?,
                _ => {},
            }
        }
    }

    Ok(())
}

/// Handle dashboard tab events
async fn handle_dashboard_events(app: &mut App, key: KeyEvent, _client: Arc<aerolithsClient>) -> Result<()> {
    match key.code {
        KeyCode::Char('r') => {
            app.set_status("Refreshing dashboard data...".to_string());
            // Trigger dashboard refresh
        },
        KeyCode::Char('c') => {
            app.dashboard.recent_activity.clear();
            app.set_status("Activity log cleared".to_string());
        },
        _ => {},
    }
    Ok(())
}

/// Handle node manager tab events
async fn handle_node_manager_events(app: &mut App, key: KeyEvent, client: Arc<aerolithsClient>) -> Result<()> {
    match key.code {
        KeyCode::Up => {
            if let Some(selected) = app.node_manager.selected_node {
                if selected > 0 {
                    app.node_manager.selected_node = Some(selected - 1);
                    app.node_manager.table_state.select(Some(selected - 1));
                }
            } else if !app.node_manager.nodes.is_empty() {
                app.node_manager.selected_node = Some(0);
                app.node_manager.table_state.select(Some(0));
            }
        },
        KeyCode::Down => {
            if let Some(selected) = app.node_manager.selected_node {
                if selected < app.node_manager.nodes.len() - 1 {
                    app.node_manager.selected_node = Some(selected + 1);
                    app.node_manager.table_state.select(Some(selected + 1));
                }
            } else if !app.node_manager.nodes.is_empty() {
                app.node_manager.selected_node = Some(0);
                app.node_manager.table_state.select(Some(0));
            }
        },
        KeyCode::Char('s') | KeyCode::Char('S') => {
            start_selected_node(app, client).await?;
        },
        KeyCode::Char('t') | KeyCode::Char('T') => {
            stop_selected_node(app, client).await?;
        },
        KeyCode::Char('r') | KeyCode::Char('R') => {
            restart_selected_node(app, client).await?;
        },
        KeyCode::Char('c') | KeyCode::Char('C') => {
            configure_selected_node(app);
        },
        KeyCode::Char('a') | KeyCode::Char('A') => {
            add_new_node(app);
        },
        KeyCode::Delete => {
            remove_selected_node(app);
        },
        KeyCode::Enter => {
            show_node_details(app);
        },
        _ => {},
    }
    Ok(())
}

/// Handle cluster monitor tab events
async fn handle_cluster_monitor_events(app: &mut App, key: KeyEvent, _client: Arc<aerolithsClient>) -> Result<()> {
    match key.code {
        KeyCode::Char('r') => {
            app.set_status("Refreshing cluster status...".to_string());
            // Trigger cluster status refresh
        },
        KeyCode::Char('t') => {
            app.set_status("Showing topology view...".to_string());
            // Switch to topology visualization
        },
        KeyCode::Char('a') => {
            app.set_status("Showing alerts view...".to_string());
            // Switch to alerts view
        },
        KeyCode::Char('c') => {
            app.cluster_monitor.alerts.clear();
            app.set_status("Alerts cleared".to_string());
        },
        _ => {},
    }
    Ok(())
}

/// Handle test runner tab events
async fn handle_test_runner_events(app: &mut App, key: KeyEvent, client: Arc<aerolithsClient>) -> Result<()> {
    match key.code {
        KeyCode::Up => {
            if let Some(selected) = app.test_runner.selected_suite {
                if selected > 0 {
                    app.test_runner.selected_suite = Some(selected - 1);
                }
            } else if !app.test_runner.test_suites.is_empty() {
                app.test_runner.selected_suite = Some(0);
            }
        },
        KeyCode::Down => {
            if let Some(selected) = app.test_runner.selected_suite {
                if selected < app.test_runner.test_suites.len() - 1 {
                    app.test_runner.selected_suite = Some(selected + 1);
                }
            } else if !app.test_runner.test_suites.is_empty() {
                app.test_runner.selected_suite = Some(0);
            }
        },
        KeyCode::Enter | KeyCode::Char('r') => {
            run_selected_test_suite(app, client).await?;
        },
        KeyCode::Char('s') => {
            stop_test_execution(app);
        },
        KeyCode::Char('c') => {
            app.test_runner.test_output.clear();
            app.set_status("Test output cleared".to_string());
        },
        KeyCode::Char('a') => {
            run_all_test_suites(app, client).await?;
        },
        _ => {},
    }
    Ok(())
}

/// Handle configuration tab events
async fn handle_configuration_events(app: &mut App, key: KeyEvent, _client: Arc<aerolithsClient>) -> Result<()> {
    match key.code {
        KeyCode::Up => {
            if let Some(selected) = app.configuration.selected_section {
                if selected > 0 {
                    app.configuration.selected_section = Some(selected - 1);
                }
            } else if !app.configuration.config_sections.is_empty() {
                app.configuration.selected_section = Some(0);
            }
        },
        KeyCode::Down => {
            if let Some(selected) = app.configuration.selected_section {
                if selected < app.configuration.config_sections.len() - 1 {
                    app.configuration.selected_section = Some(selected + 1);
                }
            } else if !app.configuration.config_sections.is_empty() {
                app.configuration.selected_section = Some(0);
            }
        },
        KeyCode::Enter => {
            edit_selected_config_section(app);
        },
        KeyCode::Char('v') => {
            validate_configuration(app);
        },
        KeyCode::Char('s') => {
            save_configuration(app);
        },
        KeyCode::Char('l') => {
            load_configuration(app);
        },
        KeyCode::Char('r') => {
            reset_configuration(app);
        },
        _ => {
            // Handle text editing if in edit mode
            if app.configuration.editor_state.is_editing {
                handle_config_text_input(app, key);
            }
        },
    }
    Ok(())
}

/// Handle console tab events
async fn handle_console_events(app: &mut App, key: KeyEvent, client: Arc<aerolithsClient>) -> Result<()> {
    match app.console.mode {
        ConsoleMode::Command => {
            match key.code {
                KeyCode::Enter => {
                    execute_console_command(app, client).await?;
                },
                KeyCode::Up => {
                    navigate_command_history_up(app);
                },
                KeyCode::Down => {
                    navigate_command_history_down(app);
                },
                KeyCode::Char(c) => {
                    app.console.input.push(c);
                },
                KeyCode::Backspace => {
                    app.console.input.pop();
                },
                KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                    app.console.input.clear();
                },
                KeyCode::Char('l') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                    app.console.output.clear();
                },
                _ => {},
            }
        },
        ConsoleMode::LogViewing => {
            match key.code {
                KeyCode::Char('c') => {
                    app.console.mode = ConsoleMode::Command;
                    app.set_status("Switched to command mode".to_string());
                },
                KeyCode::Char('l') => {
                    app.console.output.clear();
                    app.set_status("Console output cleared".to_string());
                },
                _ => {},
            }
        },
    }
    Ok(())
}

/// Show help information
fn show_help(app: &mut App) {
    let help_text = match app.current_tab {
        0 => "Dashboard Help:\n[R] Refresh data\n[C] Clear activity log",
        1 => "Node Manager Help:\n[↑↓] Navigate nodes\n[S] Start node\n[T] Stop node\n[R] Restart node\n[C] Configure\n[A] Add node\n[Del] Remove node",
        2 => "Cluster Monitor Help:\n[R] Refresh status\n[T] Topology view\n[A] Alerts view\n[C] Clear alerts",
        3 => "Test Runner Help:\n[↑↓] Navigate test suites\n[Enter/R] Run selected suite\n[S] Stop execution\n[C] Clear output\n[A] Run all suites",
        4 => "Configuration Help:\n[↑↓] Navigate sections\n[Enter] Edit section\n[V] Validate\n[S] Save\n[L] Load\n[R] Reset",
        5 => "Console Help:\n[Enter] Execute command\n[↑↓] Command history\n[Ctrl+C] Clear input\n[Ctrl+L] Clear output",
        _ => "Global Help:\n[Tab] Next tab\n[Shift+Tab] Previous tab\n[F1/H] Help\n[F5] Refresh\n[Esc] Clear messages\n[Ctrl+Q] Quit",
    };

    app.set_status(help_text.to_string());
}

/// Refresh data for current tab
async fn refresh_data(app: &mut App, client: Arc<aerolithsClient>) -> Result<()> {
    match app.current_tab {
        0 => {
            // Refresh dashboard data
            app.set_status("Refreshing dashboard...".to_string());
            // Implementation would fetch fresh metrics, node status, etc.
        },
        1 => {
            // Refresh node list
            app.set_status("Refreshing node list...".to_string());
            // Implementation would fetch current node status
        },
        2 => {
            // Refresh cluster status
            app.set_status("Refreshing cluster status...".to_string());
            // Implementation would fetch cluster topology, metrics, alerts
        },
        3 => {
            // Refresh test suites
            app.set_status("Refreshing test suites...".to_string());
            // Implementation would discover available test suites
        },
        4 => {
            // Refresh configuration
            app.set_status("Refreshing configuration...".to_string());
            // Implementation would reload configuration from server
        },
        5 => {
            // No specific refresh for console
            app.set_status("Console is up to date".to_string());
        },
        _ => {},
    }
    Ok(())
}

// Node management functions

async fn start_selected_node(app: &mut App, client: Arc<aerolithsClient>) -> Result<()> {
    if let Some(selected) = app.node_manager.selected_node {
        let node_name = app.node_manager.nodes.get(selected)
            .map(|n| n.name.clone())
            .unwrap_or_else(|| "Unknown".to_string());
        
        app.set_status(format!("Starting node: {}", node_name));
        
        if let Some(node) = app.node_manager.nodes.get_mut(selected) {
            node.status = NodeState::Starting;
            app.node_manager.operation_status = Some(format!("Starting node {}", node_name));
            
            // In a real implementation, this would call the API to start the node
            // For now, simulate the operation
            tokio::spawn(async move {
                tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
                // Update node status to running
            });
        }
    } else {
        app.set_error("No node selected".to_string());
    }
    Ok(())
}

async fn stop_selected_node(app: &mut App, client: Arc<aerolithsClient>) -> Result<()> {
    if let Some(selected) = app.node_manager.selected_node {
        let node_name = app.node_manager.nodes.get(selected)
            .map(|n| n.name.clone())
            .unwrap_or_else(|| "Unknown".to_string());
        
        app.set_status(format!("Stopping node: {}", node_name));
        
        if let Some(node) = app.node_manager.nodes.get_mut(selected) {
            node.status = NodeState::Stopping;
            app.node_manager.operation_status = Some(format!("Stopping node {}", node_name));
            
            // In a real implementation, this would call the API to stop the node
        }
    } else {
        app.set_error("No node selected".to_string());
    }
    Ok(())
}

async fn restart_selected_node(app: &mut App, client: Arc<aerolithsClient>) -> Result<()> {
    if let Some(selected) = app.node_manager.selected_node {
        let node_name = app.node_manager.nodes.get(selected)
            .map(|n| n.name.clone())
            .unwrap_or_else(|| "Unknown".to_string());
        
        app.set_status(format!("Restarting node: {}", node_name));
        
        if let Some(node) = app.node_manager.nodes.get_mut(selected) {
            node.status = NodeState::Stopping;
            app.node_manager.operation_status = Some(format!("Restarting node {}", node_name));
            
            // In a real implementation, this would call the API to restart the node
        }
    } else {
        app.set_error("No node selected".to_string());
    }
    Ok(())
}

fn configure_selected_node(app: &mut App) {
    if let Some(selected) = app.node_manager.selected_node {
        if let Some(node) = app.node_manager.nodes.get(selected) {
            app.node_manager.config_dialog = Some(super::app::NodeConfigDialog {
                node_id: node.id.clone(),
                config_text: node.configuration.clone(),
                cursor_position: 0,
                is_valid: true,
                validation_errors: Vec::new(),
            });
            app.set_status("Opened configuration dialog".to_string());
        }
    } else {
        app.set_error("No node selected".to_string());
    }
}

fn add_new_node(app: &mut App) {
    let new_node = super::app::ManagedNode {
        id: format!("node-{}", app.node_manager.nodes.len() + 1),
        name: format!("Node {}", app.node_manager.nodes.len() + 1),
        endpoint: "http://localhost:8080".to_string(),
        status: NodeState::Stopped,
        capabilities: vec!["storage".to_string(), "compute".to_string()],
        configuration: "{}".to_string(),
    };
    
    app.node_manager.nodes.push(new_node);
    app.set_status("Added new node".to_string());
}

fn remove_selected_node(app: &mut App) {
    if let Some(selected) = app.node_manager.selected_node {
        if selected < app.node_manager.nodes.len() {
            let removed_node = app.node_manager.nodes.remove(selected);
            app.set_status(format!("Removed node: {}", removed_node.name));
            
            // Adjust selection
            if app.node_manager.nodes.is_empty() {
                app.node_manager.selected_node = None;
                app.node_manager.table_state.select(None);
            } else if selected >= app.node_manager.nodes.len() {
                app.node_manager.selected_node = Some(app.node_manager.nodes.len() - 1);
                app.node_manager.table_state.select(Some(app.node_manager.nodes.len() - 1));
            }
        }
    } else {
        app.set_error("No node selected".to_string());
    }
}

fn show_node_details(app: &mut App) {
    if let Some(selected) = app.node_manager.selected_node {
        if let Some(node) = app.node_manager.nodes.get(selected) {
            let details = format!(
                "Node Details:\nID: {}\nName: {}\nEndpoint: {}\nStatus: {}\nCapabilities: {}",
                node.id, node.name, node.endpoint, node.status, node.capabilities.join(", ")
            );
            app.set_status(details);
        }
    }
}

// Test runner functions

async fn run_selected_test_suite(app: &mut App, client: Arc<aerolithsClient>) -> Result<()> {
    if let Some(selected) = app.test_runner.selected_suite {
        let suite_name = app.test_runner.test_suites.get(selected)
            .map(|s| s.name.clone())
            .unwrap_or_else(|| "Unknown Suite".to_string());
        
        app.test_runner.execution_status = TestExecutionStatus::Running {
            suite: suite_name.clone(),
            progress: 0.0,
        };
        app.set_status(format!("Running test suite: {}", suite_name));
        
        if app.test_runner.test_suites.get(selected).is_some() {
            // In a real implementation, this would execute the actual test suite
            let _suite_name = suite_name;
            tokio::spawn(async move {
                // Simulate test execution
                for _i in 1..=10 {
                    tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
                    // Update progress
                }
            });
        }
    } else {
        app.set_error("No test suite selected".to_string());
    }
    Ok(())
}

async fn run_all_test_suites(app: &mut App, client: Arc<aerolithsClient>) -> Result<()> {
    if app.test_runner.test_suites.is_empty() {
        app.set_error("No test suites available".to_string());
        return Ok(());
    }

    app.test_runner.execution_status = TestExecutionStatus::Running {
        suite: "All Suites".to_string(),
        progress: 0.0,
    };
    app.set_status("Running all test suites...".to_string());
    
    // In a real implementation, this would execute all test suites
    Ok(())
}

fn stop_test_execution(app: &mut App) {
    app.test_runner.execution_status = TestExecutionStatus::Idle;
    app.test_runner.running_tests.clear();
    app.set_status("Test execution stopped".to_string());
}

// Configuration functions

fn edit_selected_config_section(app: &mut App) {
    if let Some(selected) = app.configuration.selected_section {
        if let Some(section) = app.configuration.config_sections.get(selected) {
            app.configuration.current_config = section.content.clone();
            app.configuration.editor_state.is_editing = true;
            app.set_status(format!("Editing configuration section: {}", section.name));
        }
    } else {
        app.set_error("No configuration section selected".to_string());
    }
}

fn validate_configuration(app: &mut App) {
    // In a real implementation, this would validate the configuration
    app.configuration.validation_status = Some(super::app::ConfigValidationResult {
        is_valid: true,
        errors: Vec::new(),
        warnings: Vec::new(),
    });
    app.set_status("Configuration validated successfully".to_string());
}

fn save_configuration(app: &mut App) {
    // In a real implementation, this would save the configuration
    app.set_status("Configuration saved".to_string());
}

fn load_configuration(app: &mut App) {
    // In a real implementation, this would load configuration from server/file
    app.set_status("Configuration loaded".to_string());
}

fn reset_configuration(app: &mut App) {
    app.configuration.current_config.clear();
    app.configuration.editor_state.is_editing = false;
    app.set_status("Configuration reset".to_string());
}

fn handle_config_text_input(app: &mut App, key: KeyEvent) {
    match key.code {
        KeyCode::Char(c) => {
            app.configuration.current_config.push(c);
        },
        KeyCode::Backspace => {
            app.configuration.current_config.pop();
        },
        KeyCode::Enter => {
            app.configuration.current_config.push('\n');
        },
        KeyCode::Esc => {
            app.configuration.editor_state.is_editing = false;
            app.set_status("Exited edit mode".to_string());
        },
        _ => {},
    }
}

// Console functions

async fn execute_console_command(app: &mut App, client: Arc<aerolithsClient>) -> Result<()> {
    let command = app.console.input.trim().to_string();
    if command.is_empty() {
        return Ok(());
    }

    // Add to history
    app.console.history.push(command.clone());
    app.console.history_index = None;

    // Add command to output
    app.console.output.push(format!("> {}", command));

    // Execute command
    match command.as_str() {
        "help" => {
            app.console.output.push("Available commands:".to_string());
            app.console.output.push("  help - Show this help".to_string());
            app.console.output.push("  status - Show system status".to_string());
            app.console.output.push("  nodes - List all nodes".to_string());
            app.console.output.push("  clear - Clear console output".to_string());
            app.console.output.push("  quit - Exit application".to_string());
        },
        "status" => {
            app.console.output.push("System Status: Online".to_string());
            app.console.output.push(format!("Active nodes: {}", app.dashboard.quick_stats.active_nodes));
            app.console.output.push(format!("Total requests: {}", app.dashboard.quick_stats.total_requests));
        },
        "nodes" => {
            app.console.output.push("Managed Nodes:".to_string());
            for (i, node) in app.node_manager.nodes.iter().enumerate() {
                app.console.output.push(format!("  {}: {} ({})", i + 1, node.name, node.status));
            }
        },
        "clear" => {
            app.console.output.clear();
        },
        "quit" => {
            app.quit();
        },
        _ => {
            // In a real implementation, this would parse and execute CLI commands
            app.console.output.push(format!("Unknown command: {}", command));
            app.console.output.push("Type 'help' for available commands".to_string());
        },
    }

    // Clear input
    app.console.input.clear();
    Ok(())
}

fn navigate_command_history_up(app: &mut App) {
    if app.console.history.is_empty() {
        return;
    }

    let new_index = match app.console.history_index {
        None => app.console.history.len() - 1,
        Some(index) => {
            if index > 0 {
                index - 1
            } else {
                index
            }
        },
    };

    app.console.history_index = Some(new_index);
    app.console.input = app.console.history[new_index].clone();
}

fn navigate_command_history_down(app: &mut App) {
    if app.console.history.is_empty() {
        return;
    }

    match app.console.history_index {
        None => {},
        Some(index) => {
            if index < app.console.history.len() - 1 {
                app.console.history_index = Some(index + 1);
                app.console.input = app.console.history[index + 1].clone();
            } else {
                app.console.history_index = None;
                app.console.input.clear();
            }
        },
    }
}
