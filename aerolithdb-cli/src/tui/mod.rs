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
