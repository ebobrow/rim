// stolen from https://github.com/jmtuley/rust-trie
use std::collections::HashMap;
use std::hash::Hash;

#[derive(PartialEq, Debug)]
pub enum FetchResult<V> {
    Some(V),
    None,
    MaybeIncomplete,
}

impl<V> From<Option<V>> for FetchResult<V> {
    fn from(value: Option<V>) -> Self {
        match value {
            Some(v) => Self::Some(v),
            None => Self::None,
        }
    }
}

pub struct Trie<K, V>
where
    K: Eq + Hash + Clone,
{
    value: Option<V>,
    children: HashMap<K, Trie<K, V>>,
}

impl<K, V> Trie<K, V>
where
    K: Eq + Hash + Clone,
{
    pub fn new() -> Trie<K, V> {
        Trie {
            value: None,
            children: HashMap::new(),
        }
    }

    pub fn insert(&mut self, path: Vec<K>, v: V) {
        if path.is_empty() {
            match self.value {
                Some(_) => {
                    // panic!("key exists")
                }
                None => {
                    self.value = Some(v);
                }
            }
            return;
        }

        self.children
            .entry(path[0].clone())
            .or_insert(Trie::new())
            .insert(path[1..].to_vec(), v)
    }

    pub fn fetch(&self, path: Vec<K>) -> FetchResult<&V> {
        if path.is_empty() {
            if self.children.is_empty() {
                self.value.as_ref().into()
            } else {
                FetchResult::MaybeIncomplete
            }
        } else {
            match self.children.get(&path[0]) {
                Some(child) => child.fetch(path[1..].to_vec()),
                None => FetchResult::None,
            }
        }
    }

    /// If a keymap is found, also return the start index in case there's some nothing at the start
    ///
    /// If you have a keymap <space> that does something, but you also have some keymaps <space>a
    /// <space>b whatever, the <space> keymap will NEVER be triggered because it always registers
    /// as MaybeIncomplete. To be honest, I don't think that's an issue because why would you want
    /// a keymap that delays for a second before triggering. Unless you rely on chaining like
    /// <space> right into a second keymap. But that seems stupid. anyways maybe I'll fix it later
    /// but for now eh.
    pub fn fetch_maybe_pad_start(&self, path: Vec<K>) -> FetchResult<(usize, &V)> {
        for i in 0..path.len() {
            if let FetchResult::Some(val) = self.fetch(path[i..].to_vec()) {
                return FetchResult::Some((i, val));
            }
        }
        match self.fetch(path) {
            FetchResult::Some(val) => FetchResult::Some((0, val)),
            FetchResult::None => FetchResult::None,
            FetchResult::MaybeIncomplete => FetchResult::MaybeIncomplete,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fetch() {
        let mut trie = Trie::new();
        trie.insert("ZZ".into(), 1);
        assert_eq!(FetchResult::MaybeIncomplete, trie.fetch("Z".into()));
        assert_eq!(FetchResult::None, trie.fetch("i".into()));
    }

    #[test]
    fn fetch_maybe_pad_start() {
        let mut trie = Trie::new();
        trie.insert("jk".into(), 'a');
        trie.insert("jjj".into(), 'b');
        assert_eq!(
            FetchResult::Some((0, &'a')),
            trie.fetch_maybe_pad_start("jk".into())
        );
        assert_eq!(
            FetchResult::Some((1, &'a')),
            trie.fetch_maybe_pad_start("jjk".into())
        );
    }
}
