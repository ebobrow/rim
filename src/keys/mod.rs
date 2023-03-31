pub mod keyhandler;
mod trie;

#[cfg(test)]
mod tests {
    use crossterm::event::{KeyCode, KeyModifiers};

    use crate::keys::keyhandler::{str_to_keys, Key};

    #[test]
    fn str_to_keys_works() {
        assert_eq!(
            str_to_keys("<Esc>"),
            vec![Key {
                code: KeyCode::Esc,
                modifiers: KeyModifiers::empty()
            }]
        );
    }
}
