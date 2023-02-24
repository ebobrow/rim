use crossterm::{
    event::{self, Event, KeyCode, KeyEvent, KeyModifiers},
    Result,
};

use crate::state::State;

type KeymapFn = Box<dyn Fn(&mut State) -> Result<()>>;

pub struct Keymap {
    key: char,
    modifiers: KeyModifiers,
    f: KeymapFn,
}

impl Keymap {
    pub fn char(key: char, f: KeymapFn) -> Self {
        Self {
            key,
            modifiers: KeyModifiers::empty(),
            f,
        }
    }

    pub fn char_with_mods(key: char, modifiers: Vec<KeyModifiers>, f: KeymapFn) -> Self {
        Self {
            key,
            modifiers: modifiers
                .iter()
                .cloned()
                .reduce(KeyModifiers::union)
                .unwrap(),
            f,
        }
    }

    fn matches(&self, key_event: KeyEvent) -> bool {
        // TODO: do we ever want any of the other `KeyCode`s?
        if let KeyCode::Char(c) = key_event.code {
            if c == self.key && key_event.modifiers == self.modifiers {
                return true;
            }
        }
        false
    }

    fn call(&self, screen: &mut State) -> Result<()> {
        (self.f)(screen)
    }
}

pub fn watch(state: &mut State) -> Result<()> {
    loop {
        if let Event::Key(key_event) = event::read()? {
            if let Some(keymap) = state
                .keymaps()
                .iter()
                .find(|keymap| keymap.matches(key_event))
            {
                keymap.call(state)?;
            }
        }
    }
}
