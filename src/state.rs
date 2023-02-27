use std::{collections::HashMap, process::exit, rc::Rc};

use crossterm::{cursor::SetCursorStyle, Result};

use crate::{
    keys::{
        self,
        keyhandler::{new_keymap_trie, KeymapTrie},
    },
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
    current_key_event: Vec<u8>,
    mode: Mode,
}

impl State {
    pub fn init() -> Result<Self> {
        Ok(Self {
            screen: Screen::new()?,
            mode: Mode::Normal,
            current_key_event: Vec::new(),
            // TODO: macro for this?
            keymaps: Rc::new(HashMap::from([
                (
                    Mode::Normal,
                    new_keymap_trie(vec![
                        (
                            b"h",
                            Box::new(|state| state.screen_mut().move_cursor(-1, 0)),
                        ),
                        (b"j", Box::new(|state| state.screen_mut().move_cursor(0, 1))),
                        (
                            b"k",
                            Box::new(|state| state.screen_mut().move_cursor(0, -1)),
                        ),
                        (b"l", Box::new(|state| state.screen_mut().move_cursor(1, 0))),
                        // TODO: warn about quitting without writing
                        (b"ZZ", Box::new(|_| State::finish())),
                        (b"i", Box::new(|state| state.enter_insert_mode())),
                        (
                            b"w",
                            Box::new(|state| {
                                state.screen_mut().write();
                                Ok(())
                            }),
                        ),
                    ]),
                ),
                (
                    Mode::Insert,
                    new_keymap_trie(vec![
                        (b"jk", Box::new(|state| state.enter_normal_mode())),
                        (&[keys::ESCAPE], Box::new(|state| state.enter_normal_mode())),
                    ]),
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

    pub fn current_key_event(&self) -> &[u8] {
        self.current_key_event.as_ref()
    }

    pub fn clear_current_key_event(&mut self) {
        self.current_key_event = Vec::new();
    }

    pub fn append_current_key_event(&mut self, c: u8) {
        self.current_key_event.push(c);
    }

    pub fn set_current_key_event(&mut self, key: Vec<u8>) {
        self.current_key_event = key;
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
