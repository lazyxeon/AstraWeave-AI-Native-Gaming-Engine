use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Cond {
    Eq { key: String, val: String },
    Ne { key: String, val: String },
    Has { key: String },
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Line {
    pub speaker: String,
    pub text: String,
    #[serde(default)]
    pub set_vars: Vec<(String, String)>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Choice {
    pub text: String,
    pub go_to: String,
    #[serde(default)]
    pub require: Vec<Cond>,
}

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

pub struct DialogueState {
    pub idx: usize,
    pub map: HashMap<String, usize>,
    pub vars: HashMap<String, String>,
}

impl DialogueState {
    pub fn new(d: &Dialogue) -> Self {
        let map: HashMap<String, usize> = d
            .nodes
            .iter()
            .enumerate()
            .map(|(i, n)| (n.id.clone(), i))
            .collect();
        let idx = *map.get(&d.start).expect("start");
        Self {
            idx,
            map,
            vars: HashMap::new(),
        }
    }
    pub fn current<'a>(&self, d: &'a Dialogue) -> &'a Node {
        &d.nodes[self.idx]
    }
    pub fn choose(&mut self, d: &Dialogue, choice_idx: usize) -> bool {
        let n = self.current(d);
        if let Some(c) = n.choices.get(choice_idx) {
            if !c.require.iter().all(|cond| eval(cond, &self.vars)) {
                return false;
            }
            if let Some(&ni) = self.map.get(&c.go_to) {
                // apply set_vars of next line when we move
                self.idx = ni;
                if let Some(l) = &d.nodes[ni].line {
                    for (k, v) in &l.set_vars {
                        self.vars.insert(k.clone(), v.clone());
                    }
                }
                return true;
            }
        }
        false
    }
}

fn eval(c: &Cond, vars: &HashMap<String, String>) -> bool {
    match c {
        Cond::Eq { key, val } => vars.get(key).map(|v| v == val).unwrap_or(false),
        Cond::Ne { key, val } => vars.get(key).map(|v| v != val).unwrap_or(true),
        Cond::Has { key } => vars.contains_key(key),
    }
}

/// “Compiler”: turn a simple banter script into Dialogue nodes.
/// Format:
///   [Speaker] line text
///   -> set var=value
///   ? key == value : goto node_id
pub fn compile_banter_to_nodes(id: &str, src: &str) -> Dialogue {
    let mut nodes = vec![];
    let mut i = 0usize;
    let mut _last_id = "n0".to_string();
    for line in src.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        if let Some(rest) = line.strip_prefix('[') {
            // line
            let (speaker, text) = if let Some(end) = rest.find(']') {
                let spk = &rest[..end];
                let txt = rest[end + 1..].trim();
                (spk.to_string(), txt.to_string())
            } else {
                ("Unknown".into(), line.into())
            };
            let idn = format!("n{}", i);
            nodes.push(Node {
                id: idn.clone(),
                line: Some(Line {
                    speaker,
                    text,
                    set_vars: vec![],
                }),
                choices: vec![],
                end: false,
            });
            _last_id = idn;
            i += 1;
        } else if let Some(rest) = line.strip_prefix("->") {
            let kv = rest.trim();
            if let Some(eq) = kv.find('=') {
                let k = kv[..eq].trim().to_string();
                let v = kv[eq + 1..].trim().to_string();
                if let Some(n) = nodes.last_mut() {
                    if let Some(l) = n.line.as_mut() {
                        l.set_vars.push((k, v));
                    }
                }
            }
        } else if let Some(rest) = line.strip_prefix('?') {
            // condition goto
            // e.g., "? mood == happy : goto n2"
            let parts: Vec<_> = rest.split(':').map(|s| s.trim()).collect();
            if parts.len() == 2 {
                let cond = parts[0];
                let goto = parts[1]
                    .strip_prefix("goto")
                    .map(|s| s.trim())
                    .unwrap_or("n0")
                    .to_string();
                let mut conds = vec![];
                if cond.contains("==") {
                    let z: Vec<_> = cond.split("==").collect();
                    conds.push(Cond::Eq {
                        key: z[0].trim().into(),
                        val: z[1].trim().into(),
                    });
                } else if cond.contains("!=") {
                    let z: Vec<_> = cond.split("!=").collect();
                    conds.push(Cond::Ne {
                        key: z[0].trim().into(),
                        val: z[1].trim().into(),
                    });
                }
                if let Some(n) = nodes.last_mut() {
                    n.choices.push(Choice {
                        text: "Continue".into(),
                        go_to: goto,
                        require: conds,
                    });
                }
            }
        }
    }
    if let Some(n) = nodes.last_mut() {
        n.end = true;
    }
    Dialogue {
        id: id.into(),
        start: nodes.first().map(|n| n.id.clone()).unwrap_or("n0".into()),
        nodes,
    }
}
