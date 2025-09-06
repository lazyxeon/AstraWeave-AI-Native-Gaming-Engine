use serde::{Serialize, Deserialize};
use std::collections::HashMap;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum TaskKind {
    Gather { kind: String, count: u32 },
    Visit  { marker: String },
    Defeat { enemy: String, count: u32 },
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Task {
    pub id: String,
    pub kind: TaskKind,
    pub done: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Quest {
    pub id: String,
    pub title: String,
    pub tasks: Vec<Task>,
    pub reward_text: String,
    #[serde(default)]
    pub completed: bool,
}

#[derive(Default, Debug)]
pub struct QuestLog {
    pub quests: HashMap<String, Quest>,
}

impl QuestLog {
    pub fn add(&mut self, q: Quest) { self.quests.insert(q.id.clone(), q); }
    pub fn is_done(&self, id: &str) -> bool { self.quests.get(id).map(|q| q.completed).unwrap_or(false) }

    pub fn progress_gather(&mut self, id: &str, kind: &str, n: u32) {
        if let Some(q) = self.quests.get_mut(id) {
            for t in q.tasks.iter_mut() {
                if let TaskKind::Gather { kind: tk, count } = &mut t.kind {
                    if tk == kind && !t.done {
                        if *count > n { *count -= n; } else { *count = 0; t.done = true; }
                    }
                }
            }
            if q.tasks.iter().all(|t| t.done) { q.completed = true; }
        }
    }
}
