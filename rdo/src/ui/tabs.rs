use crate::app::{App, Tab};
use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Tabs},
    Frame,
};

pub fn render_tabs(frame: &mut Frame, app: &App, area: Rect) {
    let titles = vec![
        Tab::Home.title(),
        Tab::Profile.title(),
        Tab::Backups.title(),
        Tab::Restore.title(),
    ];

    let selected = match app.current_tab {
        Tab::Home => 0,
        Tab::Profile => 1,
        Tab::Backups => 2,
        Tab::Restore => 3,
    };

    let tabs = Tabs::new(titles)
        .block(Block::default().borders(Borders::ALL).title("RDO - Restaurador de Bases de Datos Odoo"))
        .select(selected)
        .style(Style::default().fg(Color::White))
        .highlight_style(
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        );

    frame.render_widget(tabs, area);
}
