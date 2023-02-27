use crossterm::{
    event::{self, Event, KeyCode, KeyEvent},
    Result,
};

use crate::state::{Mode, State};

use super::trie::{FetchResult, Trie};

type KeymapFn = Box<dyn Fn(&mut State) -> Result<()>>;

// TODO: modifiers
pub type KeymapTrie = Trie<u8, KeymapFn>;

pub fn new_keymap_trie(maps: Vec<(impl ToString, KeymapFn)>) -> KeymapTrie {
    let mut trie = Trie::new();
    for (k, v) in maps {
        trie.insert(k.to_string().into_bytes(), v);
    }
    trie
}

pub fn watch(state: &mut State) -> Result<()> {
    loop {
        // TODO: other events like screen resize
        if let Event::Key(key_event) = event::read()? {
            handle_key_event(key_event, state)?;
        }
    }
}

fn handle_key_event(key_event: KeyEvent, state: &mut State) -> Result<()> {
    if let KeyCode::Char(c) = key_event.code {
        if let Mode::Insert = state.mode() {
            state.screen_mut().type_char(c)?;
        }
        state.append_current_key_event(c);
        match state
            .keymaps()
            .get(state.mode())
            .unwrap()
            .fetch_maybe_pad_start(state.current_key_event().into())
        {
            FetchResult::Some((i, f)) => {
                if let Mode::Insert = state.mode() {
                    // TODO: also clear current key event after like a second and don't move cursor
                    //       forward if current key event has something but then do move forward
                    //       after you clear it
                    //     - just like a timeout but I'm worried about race conditions
                    let len = state.current_key_event().len();
                    state.screen_mut().delete_chars(len - i)?;
                }
                f(state)?;
                state.clear_current_key_event();
            }
            FetchResult::None => {
                // so this exact thing isn't anything, but what if the next keypress makes it
                // something?
                state.set_current_key_event(state.current_key_event()[1..].into());
            }
            FetchResult::MaybeIncomplete => {}
        }
    }

    Ok(())
}
