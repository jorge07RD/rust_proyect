use crate::app::App;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame,
};

pub fn render(frame: &mut Frame, app: &App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),  // Filter input
            Constraint::Min(0),     // Backups list
            Constraint::Length(5),  // Selected backup info
        ])
        .split(area);

    render_filter(frame, app, chunks[0]);
    render_backups_list(frame, app, chunks[1]);
    render_selected_info(frame, app, chunks[2]);
}

fn render_filter(frame: &mut Frame, app: &App, area: Rect) {
    let text = if app.backup_filter.is_empty() {
        "Presione / para filtrar...".to_string()
    } else {
        format!("Filtro: {}", app.backup_filter)
    };

    let paragraph = Paragraph::new(text)
        .block(Block::default().borders(Borders::ALL).title("Buscar"));

    frame.render_widget(paragraph, area);
}

fn render_backups_list(frame: &mut Frame, app: &App, area: Rect) {
    let items: Vec<ListItem> = app
        .filtered_backups
        .iter()
        .enumerate()
        .map(|(i, backup)| {
            let marker = if backup.has_dump { "✓" } else { "✗" };
            let status = if backup.has_dump {
                "Tiene dump.sql"
            } else {
                "No tiene dump.sql"
            };

            let style = if Some(i) == app.selected_backup {
                Style::default()
                    .fg(Color::Black)
                    .bg(Color::Cyan)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };

            let marker_style = if backup.has_dump {
                Style::default().fg(Color::Green)
            } else {
                Style::default().fg(Color::Red)
            };

            let content = if Some(i) == app.selected_backup {
                Line::from(vec![
                    Span::raw("> "),
                    Span::styled(marker, marker_style),
                    Span::raw(" "),
                    Span::raw(&backup.name),
                    Span::raw(" - "),
                    Span::raw(status),
                ])
            } else {
                Line::from(vec![
                    Span::raw("  "),
                    Span::styled(marker, marker_style),
                    Span::raw(" "),
                    Span::raw(&backup.name),
                    Span::raw(" - "),
                    Span::raw(status),
                ])
            };

            ListItem::new(content).style(style)
        })
        .collect();

    let title = format!("Backups Disponibles ({}/{})", app.filtered_backups.len(), app.backups.len());

    let list = List::new(items)
        .block(Block::default().borders(Borders::ALL).title(title));

    frame.render_widget(list, area);
}

fn render_selected_info(frame: &mut Frame, app: &App, area: Rect) {
    let text = if let Some(backup) = app.get_selected_backup() {
        let base_dir = app.profile.as_ref()
            .and_then(|p| p.dir_backup.as_ref())
            .map(|s| s.as_str())
            .unwrap_or("/tmp/backups");

        let mode_info = if let Some(ref profile) = app.profile {
            if profile.is_docker_mode() {
                format!("(En contenedor Docker)")
            } else {
                format!("(En sistema local)")
            }
        } else {
            String::new()
        };

        vec![
            Line::from(vec![
                Span::styled("Seleccionado: ", Style::default().fg(Color::Yellow)),
                Span::raw(&backup.name),
            ]),
            Line::from(vec![
                Span::styled("Ruta: ", Style::default().fg(Color::Yellow)),
                Span::raw(format!("{}/{} {}", base_dir, backup.name, mode_info)),
            ]),
            Line::from(vec![
                Span::styled("Estado: ", Style::default().fg(Color::Yellow)),
                if backup.has_dump {
                    Span::styled("✓ Listo para restaurar", Style::default().fg(Color::Green))
                } else {
                    Span::styled("✗ No tiene dump.sql", Style::default().fg(Color::Red))
                },
            ]),
        ]
    } else {
        vec![Line::from("No hay backup seleccionado")]
    };

    let paragraph = Paragraph::new(text)
        .block(Block::default().borders(Borders::ALL).title("Información del Backup"));

    frame.render_widget(paragraph, area);
}
