use crossterm::{
    event::{self, Event, KeyCode},
    Result,
};

use crate::state::{Mode, State};

use super::trie::Trie;

type KeymapFn = Box<dyn Fn(&mut State) -> Result<()>>;

// TODO: modifiers
pub type KeymapTrie = Trie<u8, KeymapFn>;

pub fn new_keymap_trie(maps: Vec<(String, KeymapFn)>) -> KeymapTrie {
    let mut trie = Trie::new();
    for (k, v) in maps {
        trie.insert(k.into_bytes(), v);
    }
    trie
}

pub fn watch(state: &mut State) -> Result<()> {
    loop {
        // TODO: also something like jjk should still trigger `jk` map and type a j
        if let Event::Key(key_event) = event::read()? {
            match state.mode() {
                Mode::Normal => {
                    if let KeyCode::Char(c) = key_event.code {
                        state.append_current_key_event(c);
                        match state
                            .keymaps()
                            .get(&Mode::Normal)
                            .unwrap()
                            .fetch(state.current_key_event().into())
                        {
                            Some(Some(f)) => {
                                f(state)?;
                                state.clear_current_key_event();
                            }
                            Some(None) => state.clear_current_key_event(),
                            None => {}
                        }
                    }
                }
                Mode::Insert => {
                    // TODO: also clear current key event after like a second and don't move cursor
                    // forward if current key event has something but then do move forward after
                    // you clear it
                    if let KeyCode::Char(c) = key_event.code {
                        state.append_current_key_event(c);
                        state.screen_mut().type_char(c)?;
                        match state
                            .keymaps()
                            .get(&Mode::Insert)
                            .unwrap()
                            .fetch(state.current_key_event().into())
                        {
                            Some(Some(f)) => {
                                let len = state.current_key_event().len();
                                state.screen_mut().delete_chars(len)?;
                                f(state)?;
                                state.clear_current_key_event();
                            }
                            Some(None) => state.clear_current_key_event(),
                            None => {}
                        }
                    }
                }
            }
        }
    }
}
