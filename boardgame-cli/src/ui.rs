use ratatui::{
    prelude::*,
    widgets::*,
};

use crate::App;

pub fn render(frame: &mut Frame, app: &mut App) {
    // Create the layout
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([
            Constraint::Length(3),  // Title
            Constraint::Length(3),  // Button
            Constraint::Min(0),     // Main content
            Constraint::Length(3),  // Messages
        ])
        .split(frame.area());

    // Render title
    let title = Paragraph::new("Boardgame Manager")
        .alignment(Alignment::Center)
        .block(Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded));
    frame.render_widget(title, chunks[0]);

    // Store the button area in the app state
    app.button_area = Some(chunks[1]);

    // Render "Add New Boardgame" button
    let button = Paragraph::new("[ Add New Boardgame (click or press 'a') ]")
        .alignment(Alignment::Center)
        .block(Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded));
    frame.render_widget(button, chunks[1]);

    // Render message area
    let message_style = Style::default().fg(Color::Green);
    let message_text = app.message.as_ref().map(|(msg, _)| msg.as_str()).unwrap_or("");
    let message = Paragraph::new(message_text)
        .style(message_style)
        .alignment(Alignment::Center)
        .block(Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .title("Messages"));
    frame.render_widget(message, chunks[3]);

    // Render main content
    let content = Paragraph::new("Press 'q' to quit")
        .block(Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded));
    frame.render_widget(content, chunks[2]);
} 