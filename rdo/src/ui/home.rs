use crate::app::App;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Row, Table},
    Frame,
};

pub fn render(frame: &mut Frame, app: &App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(7),  // Logo
            Constraint::Length(5),  // Status
            Constraint::Min(0),     // History
        ])
        .split(area);

    // Render logo
    render_logo(frame, chunks[0]);

    // Render status
    render_status(frame, app, chunks[1]);

    // Render history
    render_history(frame, app, chunks[2]);
}

fn render_logo(frame: &mut Frame, area: Rect) {
    let logo = vec![
        Line::from(""),
        Line::from("  ██████╗ ██████╗  ██████╗ "),
        Line::from("  ██╔══██╗██╔══██╗██╔═══██╗"),
        Line::from("  ██████╔╝██║  ██║██║   ██║"),
        Line::from("  ██╔══██╗██║  ██║██║   ██║"),
        Line::from("  ██║  ██║██████╔╝╚██████╔╝"),
        Line::from("  ╚═╝  ╚═╝╚═════╝  ╚═════╝ "),
    ];

    let paragraph = Paragraph::new(logo)
        .style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
        .block(Block::default());

    frame.render_widget(paragraph, area);
}

fn render_status(frame: &mut Frame, app: &App, area: Rect) {
    let profile_status = if app.profile.is_some() {
        Span::styled("✓ Perfil cargado", Style::default().fg(Color::Green))
    } else {
        Span::styled("✗ Sin perfil configurado", Style::default().fg(Color::Red))
    };

    let mode = if let Some(ref profile) = app.profile {
        if profile.is_docker_mode() {
            format!("Modo: Docker (Container ID: {})", profile.container_id.as_ref().unwrap_or(&String::new()))
        } else {
            "Modo: Local (PostgreSQL en el sistema)".to_string()
        }
    } else {
        "Modo: No configurado".to_string()
    };

    let backups_count = format!("Backups disponibles: {}", app.backups.len());
    let history_count = format!("Restauraciones totales: {}", app.history.len());

    let lines = vec![
        Line::from(vec![Span::raw("Estado: "), profile_status]),
        Line::from(mode),
        Line::from(backups_count),
        Line::from(history_count),
    ];

    let paragraph = Paragraph::new(lines)
        .block(Block::default().borders(Borders::ALL).title("Información"));

    frame.render_widget(paragraph, area);
}

fn render_history(frame: &mut Frame, app: &App, area: Rect) {
    let header = Row::new(vec!["Fecha", "Base de Datos", "Backup", "Estado"])
        .style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD));

    let rows: Vec<Row> = app
        .history
        .iter()
        .take(5)
        .map(|entry| {
            let date = entry.timestamp.format("%Y-%m-%d %H:%M:%S").to_string();
            let status_text = if entry.success {
                "✓ Exitoso"
            } else {
                "✗ Fallido"
            };

            Row::new(vec![
                date,
                entry.database_name.clone(),
                entry.backup_folder.clone(),
                status_text.to_string(),
            ])
            .bottom_margin(0)
        })
        .collect();

    let table = Table::new(
        rows,
        [
            Constraint::Length(20),
            Constraint::Length(25),
            Constraint::Length(25),
            Constraint::Min(10),
        ],
    )
    .header(header)
    .block(Block::default().borders(Borders::ALL).title("Historial Reciente (últimas 5)"));

    frame.render_widget(table, area);
}
