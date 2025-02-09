use std::io;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{prelude::*, Terminal};
use boardgame_core::{db::BoardgameDb, Boardgame};
use anyhow::Error;
use std::time::{Duration, Instant};

mod ui;

const DB_PATH: &str = "boardgame.db";

struct App {
    should_quit: bool,
    button_area: Option<Rect>,
    db: BoardgameDb,
    message: Option<(String, Instant)>,
    message_timeout: Duration,
}

impl App {
    fn new() -> App {
        App {
            should_quit: false,
            button_area: None,
            db: BoardgameDb::new(DB_PATH).expect("failed to create database"),
            message: None,
            message_timeout: Duration::from_secs(3),
        }
    }

    fn on_key(&mut self, key: KeyCode) {
        match key {
            KeyCode::Char('q') => self.should_quit = true,
            KeyCode::Char('a') => self.add_new_boardgame(),
            _ => {}
        }
    }

    fn on_mouse_click(&mut self, x: u16, y: u16) {
        if let Some(area) = self.button_area {
            if x >= area.x && x < area.x + area.width &&
               y >= area.y && y < area.y + area.height {
                self.add_new_boardgame();
            }
        }
    }

    fn set_message(&mut self, msg: String) {
        self.message = Some((msg, Instant::now()));
    }

    fn clear_message(&mut self) {
        self.message = None;
    }

    fn check_message_timeout(&mut self) {
        if let Some((_, timestamp)) = &self.message {
            if timestamp.elapsed() >= self.message_timeout {
                self.clear_message();
            }
        }
    }

    fn add_new_boardgame(&mut self) {
        match self.db.create(&Boardgame {
            id: None,
            name: "New Boardgame".to_string(),
            min_players: 1,
            max_players: 4,
            play_time_minutes: 30,
            description: "New Boardgame Description".to_string(),
        }) {
            Ok(_) => self.set_message("Successfully added new boardgame!".to_string()),
            Err(e) => self.set_message(format!("Error adding boardgame: {}", e)),
        }
    }
}

fn main() -> Result<(), io::Error> {
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Create app state
    let mut app = App::new();

    // Main loop
    loop {
        // Draw UI
        terminal.draw(|frame| ui::render(frame, &mut app))?;

        // Check for message timeout
        app.check_message_timeout();

        // Handle input
        if event::poll(std::time::Duration::from_millis(50))? {
            match event::read()? {
                Event::Key(key) => app.on_key(key.code),
                Event::Mouse(event) => {
                    if event::MouseEventKind::Down(event::MouseButton::Left) == event.kind {
                        app.on_mouse_click(event.column, event.row);
                    }
                }
                _ => {}
            }
        }

        if app.should_quit {
            break;
        }
    }

    // Restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    Ok(())
}
