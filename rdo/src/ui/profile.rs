use crate::app::{App, ProfileField};
use crate::ui::components::input::InputField;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

pub fn render(frame: &mut Frame, app: &App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(0),     // Form
            Constraint::Length(8),  // JSON preview
        ])
        .split(area);

    render_form(frame, app, chunks[0]);
    render_json_preview(frame, app, chunks[1]);
}

fn render_form(frame: &mut Frame, app: &App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Min(0),
        ])
        .split(area);

    let editor = &app.profile_editor;

    // Render each field
    let xhost_widget = InputField::new("Host", &editor.xhost)
        .focused(editor.current_field == ProfileField::XHost)
        .editing(editor.editing && editor.current_field == ProfileField::XHost);
    frame.render_widget(xhost_widget, chunks[0]);

    let port_widget = InputField::new("Puerto", &editor.port)
        .focused(editor.current_field == ProfileField::Port)
        .editing(editor.editing && editor.current_field == ProfileField::Port);
    frame.render_widget(port_widget, chunks[1]);

    let username_widget = InputField::new("Usuario", &editor.username)
        .focused(editor.current_field == ProfileField::Username)
        .editing(editor.editing && editor.current_field == ProfileField::Username);
    frame.render_widget(username_widget, chunks[2]);

    let password_display = if editor.password.is_empty() {
        String::new()
    } else {
        "*".repeat(editor.password.len())
    };
    let password_widget = InputField::new("Contraseña", &password_display)
        .focused(editor.current_field == ProfileField::Password)
        .editing(editor.editing && editor.current_field == ProfileField::Password);
    frame.render_widget(password_widget, chunks[3]);

    let container_widget = InputField::new("Container ID (vacío = modo local)", &editor.container_id)
        .focused(editor.current_field == ProfileField::ContainerId)
        .editing(editor.editing && editor.current_field == ProfileField::ContainerId);
    frame.render_widget(container_widget, chunks[4]);

    let dir_backup_widget = InputField::new("Directorio Backup", &editor.dir_backup)
        .focused(editor.current_field == ProfileField::DirBackup)
        .editing(editor.editing && editor.current_field == ProfileField::DirBackup);
    frame.render_widget(dir_backup_widget, chunks[5]);
}

fn render_json_preview(frame: &mut Frame, app: &App, area: Rect) {
    let json_text = if let Some(ref profile) = app.profile {
        let mut profile_clone = profile.clone();
        // Hide password
        if profile_clone.password.is_some() {
            profile_clone.password = Some("********".to_string());
        }

        serde_json::to_string_pretty(&profile_clone)
            .unwrap_or_else(|_| "Error serializing profile".to_string())
    } else {
        "No hay perfil cargado".to_string()
    };

    let paragraph = Paragraph::new(json_text)
        .block(Block::default().borders(Borders::ALL).title("Perfil Actual (JSON)"))
        .style(Style::default().fg(Color::Gray));

    frame.render_widget(paragraph, area);
}
