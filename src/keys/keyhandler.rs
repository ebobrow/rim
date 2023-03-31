use crossterm::{
    event::{self, Event, KeyCode, KeyEvent, KeyModifiers},
    Result,
};

use crate::state::{Mode, State};

use super::trie::{FetchResult, Trie};

pub type KeymapFn = Box<dyn Fn(&mut State) -> Result<()>>;

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct Key {
    pub(crate) code: KeyCode,
    pub(crate) modifiers: KeyModifiers,
}

impl Key {
    pub fn char(c: char) -> Self {
        Self {
            code: KeyCode::Char(c),
            modifiers: KeyModifiers::empty(),
        }
    }
}

pub type KeymapTrie = Trie<Key, KeymapFn>;

pub fn str_to_keys(s: &str) -> Vec<Key> {
    let mut keys = Vec::new();
    let mut i = 0;
    while i < s.len() {
        let c = s.chars().nth(i).unwrap();
        if c == '<' {
            let closing = s
                .chars()
                .enumerate()
                .skip(i)
                .find(|&(_, c)| c == '>')
                .unwrap()
                .0;
            let substr = &s[i + 1..closing];
            let key = match substr {
                "space" => Key::char(' '),
                "CR" => Key {
                    code: KeyCode::Enter,
                    modifiers: KeyModifiers::empty(),
                },
                "BS" => Key {
                    code: KeyCode::Backspace,
                    modifiers: KeyModifiers::empty(),
                },
                "Esc" => Key {
                    code: KeyCode::Esc,
                    modifiers: KeyModifiers::empty(),
                },
                _ => {
                    assert!(substr.starts_with("C-"));
                    Key {
                        code: KeyCode::Char(substr.chars().nth(2).unwrap()),
                        modifiers: KeyModifiers::CONTROL,
                    }
                }
            };
            i = closing;
            keys.push(key);
        } else {
            keys.push(Key::char(c));
        }
        i += 1;
    }
    keys
}

pub fn new_keymap_trie(maps: Vec<(&str, KeymapFn)>) -> KeymapTrie {
    let mut trie = Trie::new();
    for (k, v) in maps {
        trie.insert(str_to_keys(k), v);
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
    if key_event.modifiers.intersects(KeyModifiers::CONTROL) {
        return Ok(());
    }
    match key_event.code {
        KeyCode::Backspace => {}
        KeyCode::Enter => {}
        KeyCode::Left => {
            if let Mode::Command = state.mode() {
                state.screen_mut().command_move_cursor(-1)?;
            } else {
                state.screen_mut().active_window_mut().move_cursor_col(-1)?;
            }
            return Ok(());
        }
        KeyCode::Right => {
            if let Mode::Command = state.mode() {
                state.screen_mut().command_move_cursor(1)?;
            } else {
                state.screen_mut().active_window_mut().move_cursor_col(1)?;
            }
            return Ok(());
        }
        KeyCode::Up => {
            if let Mode::Command = state.mode() {
            } else {
                state.screen_mut().active_window_mut().move_cursor_row(-1)?;
            }
            return Ok(());
        }
        KeyCode::Down => {
            if let Mode::Command = state.mode() {
            } else {
                state.screen_mut().active_window_mut().move_cursor_row(1)?;
            }
            return Ok(());
        }
        KeyCode::Tab => {}
        KeyCode::Char(_) => {}
        KeyCode::Esc => {}

        // I don't think I care about any of these
        KeyCode::Home
        | KeyCode::BackTab
        | KeyCode::Delete
        | KeyCode::Insert
        | KeyCode::F(_)
        | KeyCode::Null
        | KeyCode::End
        | KeyCode::PageUp
        | KeyCode::PageDown
        | KeyCode::CapsLock
        | KeyCode::ScrollLock
        | KeyCode::NumLock
        | KeyCode::PrintScreen
        | KeyCode::Pause
        | KeyCode::Menu
        | KeyCode::KeypadBegin
        | KeyCode::Media(_)
        | KeyCode::Modifier(_) => {
            return Ok(());
        }
    };
    if let Mode::Insert = state.mode() {
        match key_event.code {
            KeyCode::Tab => {
                for _ in 0..4 {
                    state.screen_mut().active_window_mut().type_char(' ')?;
                }
            }
            KeyCode::Backspace => state.screen_mut().active_window_mut().delete_chars(1)?,
            KeyCode::Enter => state.screen_mut().active_window_mut().type_char('\n')?,
            KeyCode::Char(c) => state
                .screen_mut()
                .active_window_mut()
                .type_char(c as char)?,
            _ => {}
        }
    } else if let Mode::Command = state.mode() {
        match key_event.code {
            KeyCode::Tab => {}
            KeyCode::Backspace => state.screen_mut().command_delete_char()?,
            KeyCode::Enter => state.enter_command()?,
            KeyCode::Char(c) => state.screen_mut().command_type_char(c as char)?,
            _ => {}
        }
    }
    state.append_current_key_event(Key {
        code: key_event.code,
        modifiers: key_event.modifiers.difference(KeyModifiers::SHIFT),
    });
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
                //     - and display the char differently so it's clear it's pending completion
                let len = state.current_key_event().len();
                state
                    .screen_mut()
                    .active_window_mut()
                    .delete_chars(len - i)?;
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

    Ok(())
}
