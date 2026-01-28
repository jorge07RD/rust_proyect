mod app;
mod core;
mod events;
mod tui;
mod ui;

use app::App;
use core::history::HistoryManager;
use core::profile;
use crossterm::event::{self, Event};
use std::time::Duration;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize app
    let mut app = App::new();

    // Try to load profile if it exists
    if let Ok(profile) = profile::load_profile() {
        app.profile = Some(profile);
        app.profile_editor = app::ProfileEditor::from_profile(app.profile.as_ref().unwrap());

        // Trigger initial backups load
        let _ = app.action_tx.send(events::Action::RefreshBackups);
    }

    // Load history
    if let Ok(history_manager) = HistoryManager::load() {
        app.history = history_manager.get_recent(10);
    }

    // Setup terminal
    tui::setup_terminal()?;
    let mut terminal = tui::init()?;

    // Run the event loop
    let result = run_event_loop(&mut terminal, &mut app).await;

    // Restore terminal
    tui::restore_terminal()?;

    result
}

async fn run_event_loop(
    terminal: &mut ratatui::Terminal<ratatui::backend::CrosstermBackend<std::io::Stdout>>,
    app: &mut App,
) -> anyhow::Result<()> {
    loop {
        // Draw UI
        terminal.draw(|f| ui::render(f, app))?;

        // Poll for keyboard events with timeout
        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                if let Some(action) = events::handle_key_event(key, app) {
                    app.update(action).await?;
                }
            }
        }

        // Process async actions from the channel
        while let Ok(action) = app.action_rx.try_recv() {
            app.update(action).await?;
        }

        // Check if should quit
        if app.should_quit {
            break;
        }
    }

    Ok(())
}
