use crossterm::Result;

use state::State;

mod buffer;
mod keyhandler;
mod screen;
mod state;

// TODO list:
// [x] Data structure for keybinds, so that no nested match tree and also user customization
// [ ] Multiple keys in a row (like <leader>f)
// [ ] Edit modes (for now, Normal, Insert, and Command)
// [x] Data structure for text so that you aren't allowed to move cursor off of text
// [ ] Scroll
// [x] Files
// [ ] (async for stream stuff? or at least buffered read?)
// [ ] Is there a way to gracefully exit on panic?
// [ ] Line numbers
// [ ] editing
// [ ] unit tests?
// [ ] splits/windows

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
