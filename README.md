# Rim

A text editor written in Rust. I don't know what it will be yet.
this is a line of tezxxt writte n in Rim (don't haeve backspace implemented yet)

## TODO list:
- [x] Data structure for keybinds, so that no nested match tree and also user customization
- [x] Multiple keys in a row (like \<leader\>f)
- [ ] Edit modes
    - [x] Normal
    - [x] Insert
    - [x] Command
    - [ ] Visual
- [x] Data structure for text so that you aren't allowed to move cursor off of text
- [x] Scroll
- [ ] Sideways scrolling--currently if you have a line wider than the screen it just panics and dies
- [x] Files
    - [ ] async for stream stuff? or at least buffered read?
- [x] Is there a way to gracefully exit on panic?
- [ ] Line numbers
- [x] editing
- [ ] unit tests?
- [ ] splits/windows
- [x] Status bar
- [ ] internal dev thing but should all commands be routed through state? as in
  reexport so that you don't have to do `state.screen_mut().load_file()` but
  instead just `state.load_file()`?
- [x] More commands--I, a, A, o, O, $
