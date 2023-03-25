use std::collections::HashMap;

use crossterm::Result;

use crate::state::State;

type CommandFn = Box<dyn Fn(&mut State, Option<String>) -> Result<()>>;

pub struct Commands {
    commands: HashMap<String, CommandFn>,
}

impl Commands {
    pub fn new(maps: Vec<(String, CommandFn)>) -> Self {
        let mut commands = HashMap::new();
        for (k, v) in maps {
            commands.insert(k, v);
        }
        Self { commands }
    }

    pub fn get(&self, key: &str) -> Option<(&CommandFn, Option<String>)> {
        let (cmd, arg) = match key.trim().split_once(' ') {
            Some((cmd, arg)) => (cmd, Some(arg.to_owned())),
            None => (key.trim(), None),
        };
        self.commands.get(cmd).map(|f| (f, arg))
    }
}
