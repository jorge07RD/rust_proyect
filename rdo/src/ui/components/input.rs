use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Style},
    text::Text,
    widgets::{Block, Borders, Paragraph, Widget},
};

pub struct InputField<'a> {
    label: &'a str,
    value: &'a str,
    focused: bool,
    editing: bool,
}

impl<'a> InputField<'a> {
    pub fn new(label: &'a str, value: &'a str) -> Self {
        Self {
            label,
            value,
            focused: false,
            editing: false,
        }
    }

    pub fn focused(mut self, focused: bool) -> Self {
        self.focused = focused;
        self
    }

    pub fn editing(mut self, editing: bool) -> Self {
        self.editing = editing;
        self
    }
}

impl<'a> Widget for InputField<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let border_style = if self.editing {
            Style::default().fg(Color::Yellow)
        } else if self.focused {
            Style::default().fg(Color::Cyan)
        } else {
            Style::default()
        };

        let block = Block::default()
            .borders(Borders::ALL)
            .title(self.label)
            .border_style(border_style);

        let inner = block.inner(area);
        block.render(area, buf);

        let text = if self.editing {
            format!("{}_", self.value)
        } else {
            self.value.to_string()
        };

        let paragraph = Paragraph::new(Text::from(text));
        paragraph.render(inner, buf);
    }
}
