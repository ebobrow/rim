use std::{collections::HashMap, process::exit, rc::Rc};

use crossterm::Result;

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
                        (
                            "h".to_string(),
                            Box::new(|state| state.screen_mut().move_cursor(-1, 0)),
                        ),
                        (
                            "j".to_string(),
                            Box::new(|state| state.screen_mut().move_cursor(0, 1)),
                        ),
                        (
                            "k".to_string(),
                            Box::new(|state| state.screen_mut().move_cursor(0, -1)),
                        ),
                        (
                            "l".to_string(),
                            Box::new(|state| state.screen_mut().move_cursor(1, 0)),
                        ),
                        ("ZZ".to_string(), Box::new(|state| state.finish())),
                        (
                            "i".to_string(),
                            Box::new(|state| {
                                state.set_mode(Mode::Insert);
                                Ok(())
                            }),
                        ),
                    ]),
                ),
                (
                    Mode::Insert,
                    new_keymap_trie(vec![(
                        "jk".to_string(),
                        Box::new(|state| {
                            state.set_mode(Mode::Normal);
                            state.screen_mut().move_cursor(-1, 0)
                        }),
                    )]),
                ),
            ])),
        })
    }

    pub fn finish(&mut self) -> Result<()> {
        self.screen.finish()?;
        exit(0);
    }

    pub fn screen_mut(&mut self) -> &mut Screen {
        &mut self.screen
    }

    pub fn keymaps(&self) -> Rc<HashMap<Mode, KeymapTrie>> {
        self.keymaps.clone()
    }

    pub fn set_mode(&mut self, mode: Mode) {
        self.mode = mode;
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
}
