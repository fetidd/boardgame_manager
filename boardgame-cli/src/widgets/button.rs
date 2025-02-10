use ratatui::{prelude::*, widgets::*};

pub struct Button {
    text: String,
    color: Color
}

impl Button {
    pub fn new(text: &str) -> Self {
        Self {
            text: text.to_string(),
            color: Color::White
        }
    }

    pub fn color(self, color: Color) -> Self {
        Self {
            color,
            ..self
        }
    }
}

impl Widget for Button {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let button = Paragraph::new(self.text)
            .alignment(Alignment::Center)
            .style(Style::default().fg(self.color))
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded),
            );
        button.render(area, buf);
    }
}