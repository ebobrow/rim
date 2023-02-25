// stolen from https://github.com/jmtuley/rust-trie
use std::collections::HashMap;
use std::hash::Hash;

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

    /// `None` if possibly incomplete, `Some(None)` if no match, `Some(&V)` if match
    pub fn fetch(&self, path: Vec<K>) -> Option<Option<&V>> {
        if path.is_empty() {
            if self.children.is_empty() {
                Some(self.value.as_ref())
            } else {
                None
            }
        } else {
            match self.children.get(&path[0]) {
                Some(child) => child.fetch(path[1..].to_vec()),
                None => Some(None),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn works() {
        let mut trie = Trie::new();
        trie.insert("ZZ".into(), 1);
        assert_eq!(None, trie.fetch("Z".into()));
        assert_eq!(Some(None), trie.fetch("i".into()));
    }
}
