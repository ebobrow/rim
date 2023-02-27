use crossterm::Result;

use keys::keyhandler;
use state::State;

mod buffer;
mod keys;
mod screen;
mod state;

// TODO list:
// [x] Data structure for keybinds, so that no nested match tree and also user customization
// [x] Multiple keys in a row (like <leader>f)
// [ ] Edit modes (for now, Normal, Insert, and Command)
// [x] Data structure for text so that you aren't allowed to move cursor off of text
// [x] Scroll
// [ ] Sideways scrolling--currently if you have a line wider than the screen it just panics and dies
// [x] Files
// [ ] (async for stream stuff? or at least buffered read?)
// [x] Is there a way to gracefully exit on panic?
// [ ] Line numbers
// [x] editing
// [ ] unit tests?
// [ ] splits/windows
// [ ] Status bar
// [ ] internal dev thing but should all commands be routed through state? as in reexport so that
//     you don't have to do `state.screen_mut().load_file()` but instead just `state.load_file()`?

fn main() -> Result<()> {
    let mut state = State::init()?;

    let mut args = std::env::args();
    let _ = args.next().unwrap();
    if let Some(filename) = args.next() {
        state.screen_mut().load_file(filename)?;
    }

    // Loops until quit
    keyhandler::watch(&mut state)
}
