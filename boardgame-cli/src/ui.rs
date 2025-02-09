use std::rc::Rc;

use ratatui::{
    prelude::*,
    widgets::*,
};

use crate::{app::Mode, App};

pub fn render(frame: &mut Frame, app: &mut App) {
    match app.mode {
        Mode::Main => render_main(frame, app),
        Mode::Adding => render_adding(frame, app),
        Mode::Quitting => render_quitting(frame, app),
    }
}

fn render_quitting(frame: &mut Frame, app: &mut App) {
    let constraints = [
        Constraint::Length(3),
        Constraint::Length(3),
        Constraint::Length(3),
        Constraint::Min(2),
    ];
    let chunks = create_chunks(frame, &constraints);
    add_title("Are you sure you want to quit?", chunks[0], frame);
    add_button("Yes", chunks[1], App::quit, frame, app);
    add_button("No", chunks[2], App::go_to_previous_mode, frame, app);
    add_messages(app, chunks[3], frame);

}

fn render_adding(frame: &mut Frame, app: &mut App) {
    let chunks = create_chunks(frame, &[
        Constraint::Length(3),  // Title
        Constraint::Min(2),  // Messages
    ]);
    add_title("Add new boardgame...", chunks[0], frame);
    add_messages(app, chunks[1], frame);
}

pub fn render_main(frame: &mut Frame, app: &mut App) {
    // Create the layout
    let chunks = create_chunks(frame, &[
        Constraint::Length(3),  // Title
        Constraint::Length(3),  // Button
        Constraint::Min(2),  // Messages
    ]);
    add_title("Boardgame Manager", chunks[0], frame);
    add_button("Add Boardgame", chunks[1], App::go_to_add_new, frame, app);
    add_messages(app, chunks[2], frame);
}


fn create_chunks(frame: &mut Frame, constraints: &[Constraint]) -> Rc<[Rect]> {
    Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints(constraints)
        .split(frame.area())
}

fn add_title(title: &str, area: Rect, frame: &mut Frame) {
    let title = Paragraph::new(title)
        .alignment(Alignment::Center)
        .block(Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded));
    frame.render_widget(title, area);
}

fn add_button(text: &str, area: Rect, func: fn(&mut App) -> (), frame: &mut Frame, app: &mut App) {
    let button = Paragraph::new(text)
        .alignment(Alignment::Center)
        .block(Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded));
    app.add_button(area, func);
    frame.render_widget(button, area);
}

fn add_messages(app: &mut App, area: Rect, frame: &mut Frame) {
    let message_style = Style::default().fg(Color::Green);
    let messages = app.get_messages();
    let message_text = messages.iter().map(|(msg, _)| msg.to_owned()).collect::<Vec<String>>().join("\n");
    let message = Paragraph::new(message_text)
        .style(message_style)
        .alignment(Alignment::Center)
        .block(Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .title("Messages"));
        frame.render_widget(message, area);
}