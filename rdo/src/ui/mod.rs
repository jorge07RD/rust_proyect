pub mod components;
pub mod tabs;
pub mod home;
pub mod profile;
pub mod backups;
pub mod restore;

use crate::app::App;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    Frame,
};

pub fn render(frame: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),  // Tabs
            Constraint::Min(0),     // Content
            Constraint::Length(3),  // Help/Status bar
        ])
        .split(frame.size());

    // Render tabs
    tabs::render_tabs(frame, app, chunks[0]);

    // Render current tab content
    match app.current_tab {
        crate::app::Tab::Home => home::render(frame, app, chunks[1]),
        crate::app::Tab::Profile => profile::render(frame, app, chunks[1]),
        crate::app::Tab::Backups => backups::render(frame, app, chunks[1]),
        crate::app::Tab::Restore => restore::render(frame, app, chunks[1]),
    }

    // Render help/status bar
    render_status_bar(frame, app, chunks[2]);
}

fn render_status_bar(frame: &mut Frame, app: &App, area: Rect) {
    use ratatui::{
        style::{Color, Style},
        text::{Line, Span},
        widgets::{Block, Borders, Paragraph},
    };

    let help_text = match app.current_tab {
        crate::app::Tab::Home => "q: Salir | Tab/Shift+Tab: Cambiar pestaña | 1-4: Ir a pestaña",
        crate::app::Tab::Profile => {
            if app.profile_editor.editing {
                "Escribir | Esc/Enter: Finalizar edición | Tab: Cambiar pestaña"
            } else {
                "e: Editar | ↑/↓: Navegar | Enter: Editar campo | Ctrl+S: Guardar"
            }
        },
        crate::app::Tab::Backups => {
            if !app.backup_filter.is_empty() {
                "Escribir para filtrar | Esc: Limpiar filtro | ↑/↓: Navegar | Tab: Cambiar pestaña"
            } else {
                "r: Refrescar | ↑/↓: Navegar | /: Filtrar | Enter: Restaurar | Tab: Cambiar pestaña"
            }
        },
        crate::app::Tab::Restore => {
            if app.restore_state == crate::app::RestoreState::Idle {
                if app.restore_db_name.is_empty() {
                    "Escribir nombre de BD | Tab: Cambiar pestaña | 1-4: Ir a pestaña"
                } else {
                    "Escribir nombre de BD | Enter: Iniciar | Tab: Cambiar pestaña"
                }
            } else {
                "↑/↓: Scroll logs | Esc: Cancelar"
            }
        },
    };

    let status = if let Some(ref msg) = app.status_message {
        Line::from(vec![
            Span::styled("Status: ", Style::default().fg(Color::Yellow)),
            Span::raw(msg),
        ])
    } else {
        Line::from(help_text)
    };

    let paragraph = Paragraph::new(status)
        .block(Block::default().borders(Borders::ALL));

    frame.render_widget(paragraph, area);
}
