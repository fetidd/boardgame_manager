use std::{
    cell::{Ref, RefCell}, collections::{HashMap, VecDeque}, io::{self, Stdout}, time::{Duration, Instant}
};

use boardgame_core::db::{Boardgame, BoardgameDb};
use crossterm::event::{self, Event, KeyCode};
use ratatui::{layout::Rect, prelude::CrosstermBackend, Terminal};

use crate::ui;

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Mode {
    Main,
    Adding,
    // Editing,
    // Deleting,
    Quitting,
}

#[derive(Debug)]
pub struct App {
    pub modes: Vec<Mode>,
    pub state: AppState,
    pub buttons: HashMap<Rect, fn(&mut App) -> ()>,
    pub messages: RefCell<MessageQueue>,
    config: AppConfig,
    db: BoardgameDb,
    debug: bool,
}

#[derive(Debug)]
pub struct AppState {
    pub should_quit: bool,
}

#[derive(Debug)]
struct AppConfig {
    message_timeout: Duration,
}

type MessageQueue = VecDeque<(String, Instant)>;

impl App {
    pub fn new(db_path: &str) -> App {
        let state = AppState {
            should_quit: false,
        };
        let config = AppConfig {
            message_timeout: Duration::from_secs(3),
        };
        App {
            state,
            config,
            buttons: HashMap::new(),
            db: BoardgameDb::new(db_path).expect("failed to create database"),
            modes: Vec::from([Mode::Main]),
            debug: true,
            messages: RefCell::new(VecDeque::new()),
        }
    }

    pub fn run(
        &mut self,
        terminal: &mut Terminal<CrosstermBackend<Stdout>>,
    ) -> Result<(), io::Error> {
        while !self.state.should_quit {
            terminal.draw(|frame| ui::render(frame, self))?;
            self.check_message_timeout();
            if event::poll(std::time::Duration::from_millis(30))? {
                match event::read()? {
                    Event::Key(key) if key.kind == event::KeyEventKind::Press => {
                        self.on_key(key.code)
                    }
                    Event::Mouse(event) => {
                        if event::MouseEventKind::Down(event::MouseButton::Left) == event.kind {
                            self.on_mouse_click(event.column, event.row);
                        }
                    }
                    _ => {}
                }
            }
        }
        Ok(())
    }

    pub fn switch_mode(&mut self, mode: Mode) {
        self.modes.push(mode);
        self.buttons.clear();
    }

    pub fn prev_mode(&mut self) {
        if self.modes.len() > 1 {
            self.modes.pop();
            self.buttons.clear();
        }
    }

    pub fn get_prev_mode(&self) -> Option<Mode> {
        if self.modes.len() > 1 {
            Some(self.modes[self.modes.len() - 2])
        } else {
            None
        }
    }

    pub fn get_curr_mode(&self) -> Option<Mode> {
        self.modes.last().copied()
    }

    pub fn on_key(&mut self, key: KeyCode) {
        match key {
            KeyCode::Char('q') => self.switch_mode(Mode::Quitting),
            KeyCode::Char('a') => self.switch_mode(Mode::Adding),
            KeyCode::Backspace => self.prev_mode(),
            KeyCode::Char('d') if self.debug => self.send_debug_message(),
            key => {
                self.send_message(format!("Unhandled key: {:?}", key));
            }
        }
    }

    pub fn on_mouse_click(&mut self, x: u16, y: u16) {
        let mut func: Option<fn(&mut App) -> ()> = None;
        for (area, f) in &self.buttons {
            if area.contains((x, y).into()) {
                func = Some(*f);
                break;
            }
        }
        if let Some(f) = func {
            f(self);
        }
    }

    pub fn add_button(&mut self, area: Rect, func: fn(&mut App) -> ()) {
        self.buttons.insert(area, func);
    }

    fn send_message(&self, msg: String) {
        self.messages.borrow_mut().push_back((msg, Instant::now()));
    }

    pub fn get_messages(&self) -> Ref<MessageQueue> {
        self.messages.borrow()
    }

    fn clear_message(&self) {
        self.messages.borrow_mut().pop_front();
    }

    pub fn check_message_timeout(&mut self) {
        if let Some((_, timestamp)) = &self.messages.borrow().front() {
            if timestamp.elapsed() >= self.config.message_timeout {
                self.clear_message();
            }
        }
    }

    fn send_debug_message(&mut self) {
        self.send_message(format!(
            "previous_mode: {:?}",
            self.modes[self.modes.len() - 2]
        ));
    }

    pub fn add_new_boardgame(&self) {
        match self.db.create_boardgame(&Boardgame {
            id: None,
            name: "New Boardgame".to_string(),
            min_players: 1,
            max_players: 4,
            play_time_minutes: 30,
            description: "New Boardgame Description".to_string(),
        }) {
            Ok(_) => self.send_message("Successfully added new boardgame!".to_string()),
            Err(e) => self.send_message(format!("Error adding boardgame: {}", e)),
        }
    }

    pub fn go_to_main(&mut self) {
        self.switch_mode(Mode::Main);
    }

    pub fn go_to_quit(&mut self) {
        self.switch_mode(Mode::Quitting);
    }

    pub fn go_to_add_new(&mut self) {
        self.switch_mode(Mode::Adding);
    }

    pub fn quit(&mut self) {
        self.state.should_quit = true;
    }

    pub fn get_boardgames(&self) -> Vec<Boardgame> {
        let result = self.db.get_all_boardgames();
        match result {
            Ok(boardgames) => boardgames,
            Err(e) => {
                self.send_message(format!("Error getting boardgames: {}", e));
                Vec::new()
            }
        }
    }
}
