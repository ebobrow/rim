// use std::{cell::RefCell, fmt::Debug};

// const ALPHABET_SIZE: usize = 256;

// #[derive(Debug)]
// pub struct TrieNode<T> {
//     children: [Option<Box<TrieNode<T>>>; ALPHABET_SIZE],
//     terminal: bool,
//     val: Option<T>,
// }

// impl<T> TrieNode<T>
// where
//     T: Debug,
// {
//     fn new() -> Self {
//         let mut children: Vec<Option<Box<TrieNode<T>>>> = Vec::with_capacity(ALPHABET_SIZE);
//         for _ in 0..ALPHABET_SIZE {
//             children.push(None);
//         }
//         Self {
//             children: children.try_into().unwrap(),
//             terminal: false,
//             val: None,
//         }
//     }

//     pub fn find(&self, key: String) -> Option<&T> {
//         let mut cur_node = self;
//         for c in key.chars() {
//             if let Some(node) = &cur_node.children[c as usize] {
//                 cur_node = &node;
//             } else {
//                 return None;
//             }
//         }
//         return cur_node.val.as_ref();
//     }

//     // TODO: I'm giving up for now
//     pub fn insert(&mut self, key: String, val: T) {
//         let first_char = key.chars().next().unwrap();
//         let cur_node = RefCell::new(self.children[first_char as usize]);
//         if cur_node.borrow().is_none() {
//             *cur_node.borrow_mut() = Some(Box::new(TrieNode::new()));
//         }
//         for c in key.chars() {
//             if let Some(node) = (*cur_node.borrow()).unwrap().children[c as usize].take() {
//                 cur_node.replace(Some(node));
//                 continue;
//             }
//             (*cur_node.borrow_mut()).unwrap().children[c as usize] =
//                 Some(Box::new(TrieNode::new()));
//         }
//         (*cur_node.borrow_mut()).unwrap().val = Some(val);
//     }
// }

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
