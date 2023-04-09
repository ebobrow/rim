use crossterm::Result;

use keys::keyhandler;
use state::State;

mod buffer;
mod command;
mod keys;
mod screen;
mod state;
mod window;

#[tokio::main]
async fn main() -> Result<()> {
    let mut state = State::init()?;

    let mut args = std::env::args();
    let _ = args.next().unwrap();
    if let Some(filename) = args.next() {
        state.screen_mut().active_window_mut().load_file(filename)?;
    }

    // Loops until quit
    keyhandler::watch(&mut state).await
}
