use std::rc::Rc;

use ratatui::{prelude::*, widgets::*};

use crate::{app::Mode, App};

pub fn render(frame: &mut Frame, app: &mut App) {
    if let Some(mode) = app.get_curr_mode() {
        match mode {
            Mode::Main => render_main(frame, app),
            Mode::Adding => render_adding(frame, app),
            Mode::Quitting => render_quitting(frame, app),
        }
    } else {
        panic!("no mode")
    }
}

fn render_quitting(frame: &mut Frame, app: &mut App) {
    let constraints = [
        Constraint::Min(2),
        Constraint::Length(3),
        Constraint::Length(3),
        Constraint::Min(2),
    ];
    let vertical_layout = create_vertical_layout(frame, &constraints);
    add_title("Are you sure you want to quit?", vertical_layout[1], frame);
    let button_line = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Min(10); 2])
        .split(vertical_layout[2]);
    add_button("Yes", button_line[0], App::quit, frame, app);
    add_button("No", button_line[1], App::prev_mode, frame, app);
    // add_messages(app, vertical_layout[3], frame);
}

fn render_adding(frame: &mut Frame, app: &mut App) {
    let vertical_layout = create_vertical_layout(
        frame,
        &[
            Constraint::Length(3), // Title
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Min(2), // Messages
        ],
    );
    add_title("Add new boardgame...", vertical_layout[0], frame);
    for i in 1..=6 {
        let row = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(25), Constraint::Fill(1)])
            .split(vertical_layout[i]);
        let field = Paragraph::new("field").block(Block::bordered());
        let input = Paragraph::new("").block(Block::bordered());
        frame.render_widget(field, row[0]);
        frame.render_widget(input, row[1]);
    }
    add_messages(app, *vertical_layout.last().expect("no constraint"), frame);
}

pub fn render_main(frame: &mut Frame, app: &mut App) {
    // Create the layout
    let vertical_layout = create_vertical_layout(
        frame,
        &[
            Constraint::Length(3), // Title
            Constraint::Length(3), // Button
            Constraint::Min(2),    // Messages
        ],
    );
    add_title("Boardgame Manager", vertical_layout[0], frame);
    add_button(
        "Add Boardgame",
        vertical_layout[1],
        App::go_to_add_new,
        frame,
        app,
    );
    add_messages(app, vertical_layout[2], frame);
}

fn create_vertical_layout(frame: &mut Frame, constraints: &[Constraint]) -> Rc<[Rect]> {
    Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints(constraints)
        .split(frame.area())
}

fn add_title(title: &str, area: Rect, frame: &mut Frame) {
    let title = Paragraph::new(title).alignment(Alignment::Center).block(
        Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded),
    );
    frame.render_widget(title, area);
}

fn add_button(text: &str, area: Rect, func: fn(&mut App) -> (), frame: &mut Frame, app: &mut App) {
    let button = Paragraph::new(text).alignment(Alignment::Center).block(
        Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded),
    );
    app.add_button(area, func);
    frame.render_widget(button, area);
}

fn add_messages(app: &mut App, area: Rect, frame: &mut Frame) {
    let message_style = Style::default().fg(Color::Green);
    let messages = app.get_messages();
    let message_text = messages
        .iter()
        .map(|(msg, _)| msg.to_owned())
        .collect::<Vec<String>>()
        .join("\n");
    let message = Paragraph::new(message_text)
        .style(message_style)
        .alignment(Alignment::Center)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .title("Messages"),
        );
    frame.render_widget(message, area);
}
