use std::{process::exit, rc::Rc};

use crossterm::{event::KeyModifiers, Result};

use crate::{keyhandler::Keymap, screen::Screen};

pub struct State {
    screen: Screen,
    keymaps: Rc<Vec<Keymap>>,
}

impl State {
    pub fn init() -> Result<Self> {
        Ok(Self {
            screen: Screen::new()?,
            // TODO: macro for this?
            keymaps: Rc::new(vec![
                Keymap::char('h', Box::new(|state| state.screen_mut().move_cursor(-1, 0))),
                Keymap::char('j', Box::new(|state| state.screen_mut().move_cursor(0, 1))),
                Keymap::char('k', Box::new(|state| state.screen_mut().move_cursor(0, -1))),
                Keymap::char('l', Box::new(|state| state.screen_mut().move_cursor(1, 0))),
                Keymap::char_with_mods(
                    'c',
                    vec![KeyModifiers::CONTROL],
                    Box::new(|state| state.finish()),
                ),
            ]),
        })
    }

    pub fn finish(&mut self) -> Result<()> {
        self.screen.finish()?;
        exit(0);
    }

    pub fn screen_mut(&mut self) -> &mut Screen {
        &mut self.screen
    }

    pub fn keymaps(&self) -> Rc<Vec<Keymap>> {
        self.keymaps.clone()
    }
}

