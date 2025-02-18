use std::rc::Rc;

use ratatui::{prelude::*, widgets::*};

use crate::{app::Mode, widgets::button::Button, App};

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
    let vertical_layout = create_vertical_layout(frame.area(), &constraints);
    add_title(
        "Are you sure you want to quit?",
        vertical_layout[1],
        frame,
        app,
        false,
    );
    let button_line = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Min(10); 2])
        .split(vertical_layout[2]);
    add_button(
        Button::new("Yes").green(),
        button_line[0],
        App::quit,
        frame,
        app,
    );
    add_button(
        Button::new("No").red(),
        button_line[1],
        App::prev_mode,
        frame,
        app,
    );
    // add_messages(app, vertical_layout[3], frame);
}

fn render_adding(frame: &mut Frame, app: &mut App) {
    let vertical_layout = create_vertical_layout(
        frame.area(),
        &[
            Constraint::Length(3), // Title
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Min(2),
            Constraint::Length(5), // Messages
        ],
    );
    add_title(
        "Add new boardgame...",
        vertical_layout[0],
        frame,
        app,
        false,
    );
    for (i, name) in ["Name", "Min players", "Max players"]
        .into_iter()
        .enumerate()
    {
        let row = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Fill(1)])
            .split(vertical_layout[i + 1]);
        let text = app.state.input_state.get(name).cloned().unwrap_or_default();
        let input = Paragraph::new(text).block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .title(name),
        );
        app.add_input(row[0], name);
        frame.render_widget(input, row[0]);
    }
    add_button(Button::new("Add").green(), vertical_layout[vertical_layout.len() - 2], App::add_new_boardgame, frame, app);
    add_messages(app, *vertical_layout.last().expect("no constraint"), frame);
}

pub fn render_main(frame: &mut Frame, app: &mut App) {
    // Create the layout
    let vertical_layout = create_vertical_layout(
        frame.area(),
        &[
            Constraint::Length(3), // Title
            Constraint::Length(3), // Button
            Constraint::Min(2),
            Constraint::Length(5), // Messages
        ],
    );
    add_title("Boardgame Manager", vertical_layout[0], frame, app, true);
    add_button(
        Button::new("Add Boardgame").green(),
        vertical_layout[1],
        App::go_to_add_new,
        frame,
        app,
    );
    let boardgames = app.get_boardgames();
    let boardgame_list = List::new(
        boardgames
            .iter()
            .map(|b| ListItem::new(format!("{} - {}", b.name, b.min_players))),
    )
    .block(
        Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .title("Boardgames"),
    );
    frame.render_widget(boardgame_list, vertical_layout[2]);
    add_messages(app, vertical_layout[3], frame);
}

fn create_vertical_layout(area: Rect, constraints: &[Constraint]) -> Rc<[Rect]> {
    Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints(constraints)
        .split(area)
}

fn add_title(title: &str, area: Rect, frame: &mut Frame, app: &mut App, quit: bool) {
    let layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Min(1), Constraint::Length(6)])
        .split(area);
    let title = Paragraph::new(title).alignment(Alignment::Center).block(
        Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded),
    );
    frame.render_widget(title, layout[0]);
    let button_text = if quit { "Quit" } else { "Back" };
    let button_function = if quit {
        App::go_to_quit
    } else {
        App::prev_mode
    };
    let color = if quit { Button::red } else { Button::blue };
    add_button(
        color(Button::new(button_text)),
        layout[1],
        button_function,
        frame,
        app,
    );
}

fn add_button(
    mut button: Button,
    area: Rect,
    func: fn(&mut App) -> (),
    frame: &mut Frame,
    app: &mut App,
) {
    app.add_button(area, func);
    if let Some(pos) = app.cursor {
        if area.contains(pos) {
            button.highlight();
        }
    }
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
