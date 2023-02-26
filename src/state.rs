use std::{collections::HashMap, process::exit, rc::Rc};

use crossterm::{cursor::SetCursorStyle, Result};

use crate::{
    keys::keyhandler::{new_keymap_trie, KeymapTrie},
    screen::Screen,
};

#[derive(PartialEq, Eq, Hash)]
pub enum Mode {
    Normal,
    Insert,
}

pub struct State {
    screen: Screen,
    keymaps: Rc<HashMap<Mode, KeymapTrie>>,

    // TODO: store as &[u8] ?
    current_key_event: String,
    mode: Mode,
}

impl State {
    pub fn init() -> Result<Self> {
        Ok(Self {
            screen: Screen::new()?,
            mode: Mode::Normal,
            current_key_event: String::new(),
            // TODO: macro for this?
            keymaps: Rc::new(HashMap::from([
                (
                    Mode::Normal,
                    new_keymap_trie(vec![
                        ("h", Box::new(|state| state.screen_mut().move_cursor(-1, 0))),
                        ("j", Box::new(|state| state.screen_mut().move_cursor(0, 1))),
                        ("k", Box::new(|state| state.screen_mut().move_cursor(0, -1))),
                        ("l", Box::new(|state| state.screen_mut().move_cursor(1, 0))),
                        ("ZZ", Box::new(|_| State::finish())),
                        ("i", Box::new(|state| state.enter_insert_mode())),
                        (
                            "w",
                            Box::new(|state| {
                                state.screen_mut().write();
                                Ok(())
                            }),
                        ),
                    ]),
                ),
                (
                    Mode::Insert,
                    new_keymap_trie(vec![("jk", Box::new(|state| state.enter_normal_mode()))]),
                ),
            ])),
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

    pub fn current_key_event(&self) -> &str {
        self.current_key_event.as_ref()
    }

    pub fn clear_current_key_event(&mut self) {
        self.current_key_event = String::new();
    }

    pub fn append_current_key_event(&mut self, c: char) {
        self.current_key_event.push(c);
    }

    pub fn enter_insert_mode(&mut self) -> Result<()> {
        self.mode = Mode::Insert;
        self.screen.set_cursor_shape(SetCursorStyle::SteadyBar)
    }

    pub fn enter_normal_mode(&mut self) -> Result<()> {
        self.mode = Mode::Normal;
        self.screen.set_cursor_shape(SetCursorStyle::SteadyBlock)?;
        self.screen.move_cursor(-1, 0)
    }
}
