use std::{
    cell::{Ref, RefCell},
    collections::{HashMap, VecDeque},
    io::{self, Stdout},
    time::{Duration, Instant},
};

use boardgame_core::{db::{Boardgame, BoardgameDb}, strings::*};
use crossterm::event::{self, Event, KeyCode};
use ratatui::{
    layout::{Position, Rect},
    prelude::CrosstermBackend,
    Terminal,
};

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
    pub cursor: Option<Position>,
    config: AppConfig,
    db: BoardgameDb,
    debug: bool,
}

#[derive(Debug)]
pub struct AppState {
    pub should_quit: bool,
    pub inputs: HashMap<Rect, String>,
    pub input_state: HashMap<String, String>,
    pub selected_input: Option<String>,
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
            inputs: HashMap::new(),
            input_state: HashMap::new(),
            selected_input: None,
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
            cursor: None,
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
                        } else if event.kind == event::MouseEventKind::Moved {
                            self.update_cursor((event.column, event.row));
                        }
                    }
                    _ => {}
                }
            }
        }
        Ok(())
    }

    fn clear_state(&mut self) {
        self.buttons.clear();
        self.state.inputs.clear();
        self.state.input_state.clear();
        self.state.selected_input = None;
    }

    pub fn switch_mode(&mut self, mode: Mode) {
        self.modes.push(mode);
        self.clear_state();
    }

    pub fn prev_mode(&mut self) {
        if self.modes.len() > 1 {
            self.modes.pop();
            self.clear_state();
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
        if let Some(input) = &self.state.selected_input {
            if !self.state.input_state.contains_key(input) {
                self.state.input_state.insert(input.clone(), String::new());
            }
            let mut input_state = self.state.input_state.get_mut(input).expect("how is this not present?");
            match key {
                KeyCode::Enter => self.state.selected_input = None,
                KeyCode::Tab => {},
                KeyCode::Backspace => input_state.pop().map_or((), |_| ()),
                KeyCode::Char(ch) => input_state.push(ch),
                key => self.send_message(format!("Unhandled key: {:?}", key))
            }
        } else {
            match key {
                KeyCode::Char('q') => self.go_to_quit(),
                KeyCode::Backspace => self.prev_mode(),
                KeyCode::Char('d') if self.debug => self.send_debug_message(),
                key => {
                    self.send_message(format!("Unhandled key: {:?}", key));
                }
            }
        }
    }

    pub fn on_mouse_click(&mut self, x: u16, y: u16) {
        let mut key = None;
        for (area, k) in &self.state.inputs {
            if area.contains((x, y).into()) {
                key = Some(k);
            }
        }
        if let Some(key) = key {
            self.state.selected_input = Some(key.to_owned());
            return;
        } else {
            self.state.selected_input = None;
        }
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

    pub fn add_input(&mut self, area: Rect, key: &str) {
        self.state.inputs.insert(area, key.to_string());
        // self.state.input_state.insert(key.to_string(), String::new());
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
        let mut should_clear = false;
        if let Some((_, timestamp)) = &self.messages.borrow().front() {
            if timestamp.elapsed() >= self.config.message_timeout {
                should_clear = true;
            }
        }
        if should_clear {
            self.clear_message();
        }
    }

    fn update_cursor(&mut self, (x, y): (u16, u16)) {
        self.cursor = Some(Position::new(x, y))
    }

    fn send_debug_message(&mut self) {
        self.send_message(format!("previous_mode: {:?}", self.get_prev_mode()));
        self.send_message(format!("input state: {:?}", self.state.input_state));
        self.send_message(format!("inputs: {:?}", self.state.inputs));
    }

    pub fn add_new_boardgame(&mut self) {
        let name = self.state.input_state.get(BG_NAME).expect(&format!("'{}' not in input_state", BG_NAME)).to_owned();
        let description = self.state.input_state.get(BG_DESCRIPTION).expect(&format!("'{}' not in input_state", BG_DESCRIPTION)).to_owned();
        let mut numbers = [0, 0, 0];
        for (field, pos) in [(BG_MIN_PLAYERS, 0), (BG_MAX_PLAYERS, 1), (BG_PLAY_TIME, 2)] {
            match self.state.input_state.get(field).expect(&format!("'{}' not in input_state", field)).parse::<i32>() {
                Err(e) => {
                    self.send_message(format!("Bad value for '{}': {}", field, e));
                    return;
                },
                Ok(v) => numbers[pos] = v
            }
        }
        match self.db.create_boardgame(&Boardgame {
            id: None,
            name,
            min_players: numbers[0],
            max_players: numbers[1],
            play_time_minutes: numbers[2],
            description
        }) {
            Ok(_) => {
                self.switch_mode(Mode::Main);
                self.send_message("Successfully added new boardgame!".to_string())
            },
            Err(e) => self.send_message(format!("Error adding boardgame: {}", e)),
        }
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
