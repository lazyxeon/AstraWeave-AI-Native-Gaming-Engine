use serde::{Serialize, Deserialize};
use std::collections::HashMap;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Line { pub speaker: String, pub text: String }

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Choice { pub text: String, pub go_to: String }

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Node {
    pub id: String,
    pub line: Option<Line>,
    #[serde(default)]
    pub choices: Vec<Choice>,
    #[serde(default)]
    pub end: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Dialogue {
    pub id: String,
    pub start: String,
    pub nodes: Vec<Node>,
}

impl Dialogue {
    pub fn index(&self) -> HashMap<String, usize> {
        self.nodes.iter().enumerate().map(|(i,n)| (n.id.clone(), i)).collect()
    }
}

pub struct DialogueState {
    pub idx: usize,
    pub map: HashMap<String, usize>,
}

impl DialogueState {
    pub fn new(d: &Dialogue) -> Self {
        let map = d.index();
        let idx = *map.get(&d.start).expect("start node");
        Self { idx, map }
    }
    pub fn current<'a>(&self, d: &'a Dialogue) -> &'a Node { &d.nodes[self.idx] }
    pub fn choose(&mut self, d: &Dialogue, choice_idx: usize) -> bool {
        let n = self.current(d);
        if n.end { return false; }
        if let Some(c) = n.choices.get(choice_idx) {
            if let Some(&ni) = self.map.get(&c.go_to) { self.idx = ni; return true; }
        }
        false
    }
}
