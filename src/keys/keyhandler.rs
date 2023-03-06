use crossterm::{
    event::{self, Event, KeyCode, KeyEvent, KeyModifiers},
    Result,
};

use crate::state::{Mode, State};

use super::trie::{FetchResult, Trie};

pub type KeymapFn = Box<dyn Fn(&mut State) -> Result<()>>;

// TODO: modifiers
pub type KeymapTrie = Trie<u8, KeymapFn>;

pub fn new_keymap_trie(maps: Vec<(&[u8], KeymapFn)>) -> KeymapTrie {
    let mut trie = Trie::new();
    for (k, v) in maps {
        trie.insert(k.to_vec(), v);
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
    let c = match key_event.code {
        KeyCode::Backspace => super::BACKSPACE,
        KeyCode::Enter => b'\n',
        KeyCode::Left => {
            if let Mode::Command = state.mode() {
                state.screen_mut().command_move_cursor(-1)?;
            } else {
                state.screen_mut().move_cursor_col(-1)?;
            }
            return Ok(());
        }
        KeyCode::Right => {
            if let Mode::Command = state.mode() {
                state.screen_mut().command_move_cursor(1)?;
            } else {
                state.screen_mut().move_cursor_col(1)?;
            }
            return Ok(());
        }
        KeyCode::Up => {
            if let Mode::Command = state.mode() {
            } else {
                state.screen_mut().move_cursor_row(-1)?;
            }
            return Ok(());
        }
        KeyCode::Down => {
            if let Mode::Command = state.mode() {
            } else {
                state.screen_mut().move_cursor_row(1)?;
            }
            return Ok(());
        }
        KeyCode::Tab => super::TAB,
        KeyCode::Char(c) => c as u8,
        KeyCode::Esc => super::ESCAPE,

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
        match c {
            super::TAB => {
                for _ in 0..4 {
                    state.screen_mut().type_char(' ')?;
                }
            }
            super::BACKSPACE => state.screen_mut().delete_chars(1)?,
            _ => state.screen_mut().type_char(c as char)?,
        }
    } else if let Mode::Command = state.mode() {
        match c {
            super::TAB => {}
            super::BACKSPACE => state.screen_mut().command_delete_char()?,
            b'\n' => state.enter_command()?,
            _ => state.screen_mut().command_type_char(c as char)?,
        }
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
                //     - and display the char differently so it's clear it's pending completion
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

    Ok(())
}
