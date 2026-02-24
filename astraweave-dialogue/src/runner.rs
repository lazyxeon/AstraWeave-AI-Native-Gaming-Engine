//! Dialogue runner — state machine that walks a [`DialogueGraph`] node by node.
//!
//! The runner tracks the current node, exposes available choices, and advances
//! when the player picks one. It emits events that the game loop can react to
//! (e.g. trigger cinematics, update UI, modify game state).

use crate::{DialogueGraph, DialogueNode};
use anyhow::{Context, Result};

// ── Events ─────────────────────────────────────────────────────────────────

/// Events emitted by the dialogue runner as the player progresses.
#[derive(Debug, Clone, PartialEq)]
pub enum DialogueEvent {
    /// A new node has become active; the UI should display its text & choices.
    NodeEntered {
        node_id: String,
        text: String,
        choices: Vec<String>,
    },
    /// The player selected a choice. `choice_index` is 0-based.
    ChoiceMade {
        node_id: String,
        choice_index: usize,
        choice_text: String,
        next_node_id: Option<String>,
    },
    /// The dialogue has ended (terminal node reached or explicit end).
    Ended { last_node_id: String },
}

// ── Runner State ───────────────────────────────────────────────────────────

/// Operational state of the dialogue runner.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RunnerState {
    /// Runner is idle — no dialogue loaded or dialogue has ended.
    Idle,
    /// Waiting for the player to pick a choice on the current node.
    WaitingForChoice,
    /// The dialogue reached a terminal node and is finished.
    Finished,
}

// ── DialogueRunner ─────────────────────────────────────────────────────────

/// Drives a branching dialogue from start to finish.
///
/// ```text
/// start() → NodeEntered → choose(idx) → NodeEntered → … → Ended
/// ```
#[derive(Debug, Clone)]
pub struct DialogueRunner {
    graph: DialogueGraph,
    current_node_id: Option<String>,
    state: RunnerState,
    /// Accumulated events since the last `drain_events` call.
    pending_events: Vec<DialogueEvent>,
    /// History of visited node IDs (in order).
    history: Vec<String>,
}

impl DialogueRunner {
    /// Creates a new runner backed by the given graph. Call [`start`] to begin.
    #[must_use]
    pub fn new(graph: DialogueGraph) -> Self {
        Self {
            graph,
            current_node_id: None,
            state: RunnerState::Idle,
            pending_events: Vec::new(),
            history: Vec::new(),
        }
    }

    // ── Lifecycle ──────────────────────────────────────────────────────

    /// Starts (or restarts) the dialogue at the given node ID.
    ///
    /// Emits [`DialogueEvent::NodeEntered`] for the start node.
    pub fn start(&mut self, start_node_id: &str) -> Result<()> {
        let node = self
            .graph
            .get_node(start_node_id)
            .with_context(|| format!("Start node '{}' not found in dialogue graph", start_node_id))?
            .clone();

        self.history.clear();
        self.enter_node(&node);
        Ok(())
    }

    /// Selects a choice by 0-based index on the current node.
    ///
    /// Emits [`DialogueEvent::ChoiceMade`] and either [`NodeEntered`] or [`Ended`].
    pub fn choose(&mut self, choice_index: usize) -> Result<()> {
        anyhow::ensure!(
            self.state == RunnerState::WaitingForChoice,
            "Cannot choose when runner state is {:?}",
            self.state
        );

        let node_id = self
            .current_node_id
            .as_ref()
            .context("No current node")?
            .clone();

        let node = self
            .graph
            .get_node(&node_id)
            .context("Current node disappeared from graph")?
            .clone();

        let response = node
            .responses
            .get(choice_index)
            .with_context(|| {
                format!(
                    "Choice index {} out of range (node '{}' has {} choices)",
                    choice_index,
                    node_id,
                    node.responses.len()
                )
            })?
            .clone();

        self.pending_events.push(DialogueEvent::ChoiceMade {
            node_id: node_id.clone(),
            choice_index,
            choice_text: response.text.clone(),
            next_node_id: response.next_id.clone(),
        });

        match response.next_id {
            Some(next_id) => {
                let next_node = self
                    .graph
                    .get_node(&next_id)
                    .with_context(|| format!("Next node '{}' not found in graph", next_id))?
                    .clone();
                self.enter_node(&next_node);
            }
            None => {
                // Terminal response — dialogue ends.
                self.state = RunnerState::Finished;
                self.pending_events.push(DialogueEvent::Ended {
                    last_node_id: node_id,
                });
            }
        }

        Ok(())
    }

    /// Resets the runner to idle, clearing all state.
    pub fn reset(&mut self) {
        self.current_node_id = None;
        self.state = RunnerState::Idle;
        self.pending_events.clear();
        self.history.clear();
    }

    // ── Queries ────────────────────────────────────────────────────────

    /// Returns the current runner state.
    #[must_use]
    pub fn state(&self) -> RunnerState {
        self.state
    }

    /// Returns the current node if the runner is active.
    #[must_use]
    pub fn current_node(&self) -> Option<&DialogueNode> {
        self.current_node_id
            .as_ref()
            .and_then(|id| self.graph.get_node(id))
    }

    /// Returns the current node's ID.
    #[must_use]
    pub fn current_node_id(&self) -> Option<&str> {
        self.current_node_id.as_deref()
    }

    /// Returns the choice texts available on the current node.
    #[must_use]
    pub fn available_choices(&self) -> Vec<&str> {
        self.current_node()
            .map(|node| node.responses.iter().map(|r| r.text.as_str()).collect())
            .unwrap_or_default()
    }

    /// Returns the number of choices on the current node.
    #[must_use]
    pub fn choice_count(&self) -> usize {
        self.current_node().map(|n| n.response_count()).unwrap_or(0)
    }

    /// Returns the ordered history of visited node IDs.
    #[must_use]
    pub fn history(&self) -> &[String] {
        &self.history
    }

    /// Returns `true` if player has already visited the given node ID.
    #[must_use]
    pub fn has_visited(&self, node_id: &str) -> bool {
        self.history.iter().any(|id| id == node_id)
    }

    /// Returns the underlying graph.
    #[must_use]
    pub fn graph(&self) -> &DialogueGraph {
        &self.graph
    }

    /// Returns `true` if the dialogue is finished.
    #[must_use]
    pub fn is_finished(&self) -> bool {
        self.state == RunnerState::Finished
    }

    /// Returns `true` if the runner is waiting for a player choice.
    #[must_use]
    pub fn is_waiting(&self) -> bool {
        self.state == RunnerState::WaitingForChoice
    }

    // ── Events ─────────────────────────────────────────────────────────

    /// Drains all pending events, returning them in order.
    pub fn drain_events(&mut self) -> Vec<DialogueEvent> {
        std::mem::take(&mut self.pending_events)
    }

    /// Returns a read-only view of pending events (without draining).
    #[must_use]
    pub fn peek_events(&self) -> &[DialogueEvent] {
        &self.pending_events
    }

    // ── Internals ──────────────────────────────────────────────────────

    fn enter_node(&mut self, node: &DialogueNode) {
        let node_id = node.id.clone();
        self.current_node_id = Some(node_id.clone());
        self.history.push(node_id.clone());

        let choices: Vec<String> = node.responses.iter().map(|r| r.text.clone()).collect();

        self.pending_events.push(DialogueEvent::NodeEntered {
            node_id: node_id.clone(),
            text: node.text.clone(),
            choices: choices.clone(),
        });

        if node.is_terminal() {
            self.state = RunnerState::Finished;
            self.pending_events.push(DialogueEvent::Ended {
                last_node_id: node_id,
            });
        } else {
            self.state = RunnerState::WaitingForChoice;
        }
    }
}

// ── Tests ──────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::DialogueResponse;

    fn make_test_graph() -> DialogueGraph {
        DialogueGraph::with_nodes(vec![
            DialogueNode::new("start", "Hello!")
                .with_response(DialogueResponse::with_next("Ask more", "mid"))
                .with_response(DialogueResponse::with_next("Bye", "end")),
            DialogueNode::new("mid", "More info.")
                .with_response(DialogueResponse::with_next("Thanks", "end")),
            DialogueNode::new("end", "Farewell."),
        ])
    }

    #[test]
    fn start_enters_first_node() {
        let mut runner = DialogueRunner::new(make_test_graph());
        runner.start("start").unwrap();
        assert_eq!(runner.state(), RunnerState::WaitingForChoice);
        assert_eq!(runner.current_node_id(), Some("start"));
        assert_eq!(runner.choice_count(), 2);
    }

    #[test]
    fn choose_advances_to_next_node() {
        let mut runner = DialogueRunner::new(make_test_graph());
        runner.start("start").unwrap();
        runner.choose(0).unwrap(); // "Ask more" → mid
        assert_eq!(runner.current_node_id(), Some("mid"));
        assert_eq!(runner.choice_count(), 1);
    }

    #[test]
    fn terminal_node_finishes_dialogue() {
        let mut runner = DialogueRunner::new(make_test_graph());
        runner.start("start").unwrap();
        runner.choose(1).unwrap(); // "Bye" → end (terminal)
        assert!(runner.is_finished());
    }

    #[test]
    fn events_are_emitted_correctly() {
        let mut runner = DialogueRunner::new(make_test_graph());
        runner.start("start").unwrap();
        let events = runner.drain_events();
        assert_eq!(events.len(), 1);
        assert!(
            matches!(&events[0], DialogueEvent::NodeEntered { node_id, .. } if node_id == "start")
        );

        runner.choose(1).unwrap(); // → end
        let events = runner.drain_events();
        // ChoiceMade + NodeEntered(end) + Ended
        assert_eq!(events.len(), 3);
        assert!(matches!(
            &events[0],
            DialogueEvent::ChoiceMade {
                choice_index: 1,
                ..
            }
        ));
        assert!(matches!(&events[2], DialogueEvent::Ended { .. }));
    }

    #[test]
    fn history_tracks_visited_nodes() {
        let mut runner = DialogueRunner::new(make_test_graph());
        runner.start("start").unwrap();
        runner.choose(0).unwrap();
        runner.choose(0).unwrap();
        assert_eq!(runner.history(), &["start", "mid", "end"]);
        assert!(runner.has_visited("mid"));
    }

    #[test]
    fn choose_out_of_range_errors() {
        let mut runner = DialogueRunner::new(make_test_graph());
        runner.start("start").unwrap();
        assert!(runner.choose(99).is_err());
    }

    #[test]
    fn choose_when_idle_errors() {
        let mut runner = DialogueRunner::new(make_test_graph());
        assert!(runner.choose(0).is_err());
    }

    #[test]
    fn reset_clears_state() {
        let mut runner = DialogueRunner::new(make_test_graph());
        runner.start("start").unwrap();
        runner.choose(0).unwrap();
        runner.reset();
        assert_eq!(runner.state(), RunnerState::Idle);
        assert!(runner.history().is_empty());
        assert!(runner.current_node_id().is_none());
    }
}
