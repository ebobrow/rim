use std::{collections::HashMap, process::exit, rc::Rc};

use crossterm::{cursor::SetCursorStyle, Result};

use crate::{
    command::Commands,
    keys::keyhandler::{new_keymap_trie, Key, KeymapTrie},
    screen::Screen,
};

#[derive(PartialEq, Eq, Hash)]
pub enum Mode {
    Normal,
    Insert,
    Command,
}

pub struct State {
    screen: Screen,
    keymaps: Rc<HashMap<Mode, KeymapTrie>>,
    commands: Rc<Commands>,
    current_key_event: Vec<Key>,
    mode: Mode,
}

macro_rules! keymaps {
    ( $($key:expr => $f:expr),* $(,)? ) => {
        new_keymap_trie(vec![
            $( ($key, Box::new($f)) ),*
        ])
    };
}

macro_rules! commands {
    ( $($key:literal => $f:expr),* $(,)? ) => {
        Commands::new(vec![
            $( ($key.to_string(), Box::new($f)) ),*
        ])
    };
}

impl State {
    pub fn init() -> Result<Self> {
        Ok(Self {
            screen: Screen::new()?,
            mode: Mode::Normal,
            current_key_event: Vec::new(),
            keymaps: Rc::new(HashMap::from([
                (
                    Mode::Normal,
                    keymaps! {
                        "h" => |state| state.screen_mut().active_window_mut().move_cursor_col(-1),
                        "j" => |state| state.screen_mut().active_window_mut().move_cursor_row(1),
                        "k" => |state| state.screen_mut().active_window_mut().move_cursor_row(-1),
                        "l" => |state| state.screen_mut().active_window_mut().move_cursor_col(1),
                        "ZQ" => |_| State::finish(),
                        "ZZ" => |state| {
                            state.screen_mut().write()?;
                            State::finish()
                        },
                        "i" => |state| state.enter_insert_mode(),
                        "I" => |state| {
                            state.screen_mut().active_window_mut().zero_cursor_col()?;
                            state.enter_insert_mode()
                        },
                        "a" => |state| {
                            state.enter_insert_mode()?;
                            state.screen_mut().active_window_mut().move_cursor_col(1)
                        },
                        "A" => |state| {
                            state.enter_insert_mode()?;
                            state.screen_mut().active_window_mut().move_cursor_end_of_line()
                        },
                        "o" => |state| {
                            state.screen_mut().active_window_mut().new_line_below()?;
                            state.enter_insert_mode()
                        },
                        "O" => |state| {
                            state.screen_mut().active_window_mut().new_line_above()?;
                            state.enter_insert_mode()
                        },
                        "$" => |state| state.screen_mut().active_window_mut().move_cursor_end_of_line(),
                        "0" => |state| state.screen_mut().active_window_mut().zero_cursor_col(),
                        // TODO: `_` (start of text)
                        // - `gg`, `G`, 10G
                        // - r
                        // - u
                        ":" => |state| state.enter_command_mode(),
                        "dd" => |state| state.screen_mut().active_window_mut().delete_line(),
                        "cc" => |state| {
                            state.screen_mut().active_window_mut().change_line()?;
                            state.enter_insert_mode()
                        },
                        "<space>h" => |state| state.screen_mut().move_to_left_window(),
                        "<space>l" => |state| state.screen_mut().move_to_right_window(),
                        "<space>j" => |state| state.screen_mut().move_to_down_window(),
                        "<space>k" => |state| state.screen_mut().move_to_up_window(),
                    },
                ),
                (
                    Mode::Insert,
                    keymaps! {
                        "jk" => |state| state.enter_normal_mode(),
                        "<Esc>" => |state| state.enter_normal_mode()
                    },
                ),
                (
                    Mode::Command,
                    keymaps! {
                        "<Esc>" => |state| state.leave_command_mode()
                    },
                ),
            ])),
            commands: Rc::new(commands! {
                "w" => |state, arg| {
                    if let Some(arg) = arg {
                        state.screen_mut().write_to_filename(arg)
                    } else {
                        state.screen_mut().write()
                    }
                },
                "q" => |state, arg| {
                    if let Some(arg) = arg {
                        state.screen_mut().set_error_message(format!("unexpeted chars: `{}`", arg))
                    } else if state.screen_mut().active_window().unsaved_changes() {
                        state.screen_mut().set_error_message("no write since last change")
                    } else {
                        State::finish()
                    }
                },
                "q!" => |state, arg| {
                    if let Some(arg) = arg {
                        state.screen_mut().set_error_message(format!("unexpeted chars: `{}`", arg))
                    } else {
                        State::finish()
                    }
                },
                "wq" => |state, arg| {
                    if let Some(arg) = arg {
                        state.screen_mut().set_error_message(format!("unexpeted chars: `{}`", arg))
                    } else {
                        state.screen_mut().write()?;
                        State::finish()
                    }
                },
                // TODO: qa (the others should only quit one window)
                "vne" => |state, filename| state.screen_mut().new_vertical_split(filename),
                "new" => |state, filename| state.screen_mut().new_horizontal_split(filename),
                "e" => |state, filename| state.screen_mut().load_file(filename),
            }),
        })
    }

    pub fn finish() -> Result<()> {
        Screen::finish()?;
        exit(0);
    }

    pub fn screen_mut(&mut self) -> &mut Screen {
        &mut self.screen
    }

    pub fn keymaps(&self) -> Rc<HashMap<Mode, KeymapTrie>> {
        self.keymaps.clone()
    }

    pub fn mode(&self) -> &Mode {
        &self.mode
    }

    pub fn current_key_event(&self) -> &[Key] {
        self.current_key_event.as_ref()
    }

    pub fn clear_current_key_event(&mut self) {
        self.current_key_event = Vec::new();
    }

    pub fn append_current_key_event(&mut self, c: Key) {
        self.current_key_event.push(c);
    }

    pub fn set_current_key_event(&mut self, key: Vec<Key>) {
        self.current_key_event = key;
    }

    pub fn enter_insert_mode(&mut self) -> Result<()> {
        self.mode = Mode::Insert;
        self.screen_mut().set_message("-- INSERT --")?;
        self.screen.set_cursor_shape(SetCursorStyle::SteadyBar)
    }

    pub fn enter_normal_mode(&mut self) -> Result<()> {
        self.mode = Mode::Normal;
        self.screen_mut().set_message("")?;
        self.screen.set_cursor_shape(SetCursorStyle::SteadyBlock)?;
        self.screen.active_window_mut().move_cursor_col(-1)
    }

    pub fn enter_command_mode(&mut self) -> Result<()> {
        self.mode = Mode::Command;
        self.screen_mut().enter_command_mode()
    }

    pub fn leave_command_mode(&mut self) -> Result<()> {
        self.mode = Mode::Normal;
        self.screen_mut().leave_command_mode()
    }

    pub fn enter_command(&mut self) -> Result<()> {
        if let Some((f, arg)) = self
            .commands
            .clone()
            .get(self.screen_mut().get_curr_command())
        {
            f(self, arg)?;
        } else {
            let error_msg = format!("Unknown command `{}`", self.screen_mut().get_curr_command());
            self.screen_mut().set_error_message(error_msg)?;
        }
        self.leave_command_mode()
    }
}
