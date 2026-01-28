use crate::app::{App, RestoreState};
use crate::ui::components::scrollable_text::ScrollableText;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    widgets::{Block, Borders, Gauge, Paragraph},
    Frame,
};

pub fn render(frame: &mut Frame, app: &App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),  // DB name input
            Constraint::Length(3),  // Backup folder display
            Constraint::Length(3),  // Progress bar
            Constraint::Min(0),     // Logs
        ])
        .split(area);

    render_db_name_input(frame, app, chunks[0]);
    render_backup_folder(frame, app, chunks[1]);
    render_progress(frame, app, chunks[2]);
    render_logs(frame, app, chunks[3]);
}

fn render_db_name_input(frame: &mut Frame, app: &App, area: Rect) {
    let text = if app.restore_db_name.is_empty() {
        "Ingrese el nombre de la base de datos destino...".to_string()
    } else {
        app.restore_db_name.clone()
    };

    let style = if app.restore_state == RestoreState::Idle {
        Style::default().fg(Color::Cyan)
    } else {
        Style::default().fg(Color::Gray)
    };

    let paragraph = Paragraph::new(text)
        .block(Block::default().borders(Borders::ALL).title("Nombre de Base de Datos Destino"))
        .style(style);

    frame.render_widget(paragraph, area);
}

fn render_backup_folder(frame: &mut Frame, app: &App, area: Rect) {
    let text = if let Some(ref folder) = app.restore_backup_folder {
        folder.clone()
    } else {
        "No hay backup seleccionado. Vaya a la pestaña Backups.".to_string()
    };

    let paragraph = Paragraph::new(text)
        .block(Block::default().borders(Borders::ALL).title("Carpeta de Backup"));

    frame.render_widget(paragraph, area);
}

fn render_progress(frame: &mut Frame, app: &App, area: Rect) {
    let (label, progress, style) = match app.restore_state {
        RestoreState::Idle => ("Listo para restaurar", 0, Style::default().fg(Color::White)),
        RestoreState::Restoring => ("Restaurando...", 50, Style::default().fg(Color::Cyan)),
        RestoreState::Completed => ("Completado exitosamente", 100, Style::default().fg(Color::Green)),
        RestoreState::Failed => ("Falló la restauración", 100, Style::default().fg(Color::Red)),
    };

    let gauge = Gauge::default()
        .block(Block::default().borders(Borders::ALL).title("Progreso"))
        .gauge_style(style)
        .percent(progress)
        .label(label);

    frame.render_widget(gauge, area);
}

fn render_logs(frame: &mut Frame, app: &App, area: Rect) {
    // Calculate if we're at the bottom (showing latest logs)
    let total_lines = app.restore_logs.len() as u16;
    let visible_lines = area.height.saturating_sub(2); // Subtract border
    let max_scroll = if total_lines > visible_lines {
        total_lines - visible_lines
    } else {
        0
    };

    let at_bottom = app.log_scroll >= max_scroll || max_scroll == 0;

    let title = if app.restore_state == RestoreState::Restoring && at_bottom {
        format!("Logs ({} líneas) [Auto-scroll: ON]", app.restore_logs.len())
    } else if app.restore_state == RestoreState::Restoring {
        format!("Logs ({} líneas) [↑↓ para navegar]", app.restore_logs.len())
    } else {
        format!("Logs ({} líneas)", app.restore_logs.len())
    };

    let scrollable = ScrollableText::new(app.restore_logs.clone(), &title)
        .scroll(app.log_scroll);

    frame.render_widget(scrollable, area);
}
