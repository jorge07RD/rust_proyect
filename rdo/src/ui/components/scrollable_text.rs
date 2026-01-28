use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Style},
    text::{Line, Text},
    widgets::{Block, Borders, Paragraph, Widget, Wrap},
};

pub struct ScrollableText<'a> {
    lines: Vec<String>,
    title: &'a str,
    scroll: u16,
}

impl<'a> ScrollableText<'a> {
    pub fn new(lines: Vec<String>, title: &'a str) -> Self {
        Self {
            lines,
            title,
            scroll: 0,
        }
    }

    pub fn scroll(mut self, scroll: u16) -> Self {
        self.scroll = scroll;
        self
    }
}

impl<'a> Widget for ScrollableText<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let block = Block::default()
            .borders(Borders::ALL)
            .title(self.title)
            .border_style(Style::default().fg(Color::Gray));

        let inner = block.inner(area);
        block.render(area, buf);

        let text_lines: Vec<Line> = self
            .lines
            .iter()
            .map(|line| Line::from(line.clone()))
            .collect();

        // Calculate maximum scroll position
        let total_lines = self.lines.len() as u16;
        let visible_lines = inner.height;

        // Maximum scroll is total_lines - visible_lines, but at least 0
        let max_scroll = if total_lines > visible_lines {
            total_lines - visible_lines
        } else {
            0
        };

        // Clamp scroll to valid range
        let clamped_scroll = self.scroll.min(max_scroll);

        let text = Text::from(text_lines);
        let paragraph = Paragraph::new(text)
            .wrap(Wrap { trim: false })
            .scroll((clamped_scroll, 0));

        paragraph.render(inner, buf);
    }
}
