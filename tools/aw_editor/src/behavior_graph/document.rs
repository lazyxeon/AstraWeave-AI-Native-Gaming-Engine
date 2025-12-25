use astraweave_behavior::{BehaviorGraph, BehaviorNode, DecoratorType};
use ron::ser::PrettyConfig;
use serde::{Deserialize, Serialize};
use std::{
    collections::HashSet,
    fs,
    path::{Path, PathBuf},
};
use thiserror::Error;

pub type NodeId = u64;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub struct NodePosition {
    pub x: f32,
    pub y: f32,
}

impl Default for NodePosition {
    fn default() -> Self {
        Self { x: 0.0, y: 0.0 }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DecoratorNode {
    pub decorator: DecoratorKind,
    pub child: Option<NodeId>,
}

impl DecoratorNode {
    pub fn new(decorator: DecoratorKind) -> Self {
        Self {
            decorator,
            child: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum DecoratorKind {
    Inverter,
    Succeeder,
    Failer,
    Repeat(u32),
    Retry(u32),
}

impl DecoratorKind {
    fn to_runtime(&self) -> DecoratorType {
        match self {
            DecoratorKind::Inverter => DecoratorType::Inverter,
            DecoratorKind::Succeeder => DecoratorType::Succeeder,
            DecoratorKind::Failer => DecoratorType::Failer,
            DecoratorKind::Repeat(max) => DecoratorType::Repeat(*max),
            DecoratorKind::Retry(max) => DecoratorType::Retry(*max),
        }
    }

    fn from_runtime(decorator: &DecoratorType) -> Self {
        match decorator {
            DecoratorType::Inverter => DecoratorKind::Inverter,
            DecoratorType::Succeeder => DecoratorKind::Succeeder,
            DecoratorType::Failer => DecoratorKind::Failer,
            DecoratorType::Repeat(max) => DecoratorKind::Repeat(*max),
            DecoratorType::Retry(max) => DecoratorKind::Retry(*max),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum BehaviorGraphNodeKind {
    Action {
        name: String,
    },
    Condition {
        name: String,
    },
    Sequence {
        children: Vec<NodeId>,
    },
    Selector {
        children: Vec<NodeId>,
    },
    Parallel {
        children: Vec<NodeId>,
        success_threshold: usize,
    },
    Decorator(DecoratorNode),
}

impl BehaviorGraphNodeKind {
    pub fn display_name(&self) -> &str {
        match self {
            BehaviorGraphNodeKind::Action { .. } => "Action",
            BehaviorGraphNodeKind::Condition { .. } => "Condition",
            BehaviorGraphNodeKind::Sequence { .. } => "Sequence",
            BehaviorGraphNodeKind::Selector { .. } => "Selector",
            BehaviorGraphNodeKind::Parallel { .. } => "Parallel",
            BehaviorGraphNodeKind::Decorator(_) => "Decorator",
        }
    }

    fn children(&self) -> Vec<NodeId> {
        match self {
            BehaviorGraphNodeKind::Sequence { children }
            | BehaviorGraphNodeKind::Selector { children }
            | BehaviorGraphNodeKind::Parallel { children, .. } => children.clone(),
            BehaviorGraphNodeKind::Decorator(node) => node.child.into_iter().collect(),
            _ => Vec::new(),
        }
    }

    fn children_mut(&mut self) -> Option<&mut Vec<NodeId>> {
        match self {
            BehaviorGraphNodeKind::Sequence { children }
            | BehaviorGraphNodeKind::Selector { children }
            | BehaviorGraphNodeKind::Parallel { children, .. } => Some(children),
            _ => None,
        }
    }

    fn decorator_child_mut(&mut self) -> Option<&mut Option<NodeId>> {
        match self {
            BehaviorGraphNodeKind::Decorator(node) => Some(&mut node.child),
            _ => None,
        }
    }

    pub fn supports_children(&self) -> bool {
        matches!(
            self,
            BehaviorGraphNodeKind::Sequence { .. }
                | BehaviorGraphNodeKind::Selector { .. }
                | BehaviorGraphNodeKind::Parallel { .. }
                | BehaviorGraphNodeKind::Decorator(_)
        )
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct BehaviorGraphNode {
    pub id: NodeId,
    pub label: String,
    pub position: NodePosition,
    pub kind: BehaviorGraphNodeKind,
}

impl BehaviorGraphNode {
    pub fn new(id: NodeId, label: impl Into<String>, kind: BehaviorGraphNodeKind) -> Self {
        Self {
            id,
            label: label.into(),
            position: NodePosition::default(),
            kind,
        }
    }
}

#[derive(Debug, Clone)]
pub struct BehaviorGraphDocument {
    nodes: Vec<BehaviorGraphNode>,
    root_id: NodeId,
    next_id: NodeId,
    file_path: Option<PathBuf>,
    dirty: bool,
}

impl Default for BehaviorGraphDocument {
    fn default() -> Self {
        Self::new_default()
    }
}

impl BehaviorGraphDocument {
    pub fn new_default() -> Self {
        let mut doc = Self {
            nodes: Vec::new(),
            root_id: 0,
            next_id: 1,
            file_path: None,
            dirty: true,
        };
        let root_id = doc.allocate_id();
        doc.nodes.push(BehaviorGraphNode::new(
            root_id,
            "Idle",
            BehaviorGraphNodeKind::Action {
                name: "idle".into(),
            },
        ));
        doc.root_id = root_id;
        doc
    }

    fn allocate_id(&mut self) -> NodeId {
        let id = self.next_id;
        self.next_id += 1;
        id
    }

    fn node_index(&self, id: NodeId) -> Option<usize> {
        self.nodes.iter().position(|n| n.id == id)
    }

    pub fn node(&self, id: NodeId) -> Option<&BehaviorGraphNode> {
        self.nodes.iter().find(|n| n.id == id)
    }

    pub fn node_mut(&mut self, id: NodeId) -> Option<&mut BehaviorGraphNode> {
        let idx = self.node_index(id)?;
        self.nodes.get_mut(idx)
    }

    pub fn nodes(&self) -> &[BehaviorGraphNode] {
        &self.nodes
    }

    pub fn root_id(&self) -> NodeId {
        self.root_id
    }

    pub fn set_root(&mut self, id: NodeId) -> Result<(), BehaviorGraphDocumentError> {
        if self.node(id).is_none() {
            return Err(BehaviorGraphDocumentError::MissingNode(id));
        }
        self.root_id = id;
        self.dirty = true;
        Ok(())
    }

    pub fn file_path(&self) -> Option<&Path> {
        self.file_path.as_deref()
    }

    pub fn set_file_path(&mut self, path: Option<PathBuf>) {
        self.file_path = path;
    }

    pub fn is_dirty(&self) -> bool {
        self.dirty
    }

    pub fn mark_clean(&mut self) {
        self.dirty = false;
    }

    pub fn add_node(&mut self, label: impl Into<String>, kind: BehaviorGraphNodeKind) -> NodeId {
        let id = self.allocate_id();
        self.nodes.push(BehaviorGraphNode::new(id, label, kind));
        self.dirty = true;
        id
    }

    pub fn add_child_node(
        &mut self,
        parent: NodeId,
        label: impl Into<String>,
        kind: BehaviorGraphNodeKind,
    ) -> Result<NodeId, BehaviorGraphDocumentError> {
        let Some(parent_snapshot) = self.node(parent) else {
            return Err(BehaviorGraphDocumentError::MissingNode(parent));
        };
        if !parent_snapshot.kind.supports_children() {
            return Err(BehaviorGraphDocumentError::NodeCannotHaveChildren(parent));
        }
        let child_id = self.add_node(label, kind);
        let parent_node = self
            .node_mut(parent)
            .expect("parent must exist after snapshot");
        match parent_node.kind.children_mut() {
            Some(children) => {
                children.push(child_id);
            }
            None => {
                let Some(slot) = parent_node.kind.decorator_child_mut() else {
                    return Err(BehaviorGraphDocumentError::NodeCannotHaveChildren(parent));
                };
                if slot.replace(child_id).is_some() {
                    return Err(BehaviorGraphDocumentError::DecoratorAlreadyHasChild(parent));
                }
            }
        }
        Ok(child_id)
    }

    pub fn remove_node(&mut self, node_id: NodeId) -> Result<(), BehaviorGraphDocumentError> {
        if node_id == self.root_id {
            return Err(BehaviorGraphDocumentError::CannotDeleteRoot);
        }
        if self.node(node_id).is_none() {
            return Err(BehaviorGraphDocumentError::MissingNode(node_id));
        }

        let mut removable = HashSet::new();
        self.collect_descendants(node_id, &mut removable);
        removable.insert(node_id);
        self.nodes.retain(|node| !removable.contains(&node.id));

        for node in &mut self.nodes {
            if let Some(children) = node.kind.children_mut() {
                children.retain(|child| !removable.contains(child));
            }
            if let Some(slot) = node.kind.decorator_child_mut() {
                if slot.map(|id| removable.contains(&id)).unwrap_or(false) {
                    *slot = None;
                }
            }
        }
        self.dirty = true;
        Ok(())
    }

    fn collect_descendants(&self, node_id: NodeId, acc: &mut HashSet<NodeId>) {
        if let Some(node) = self.node(node_id) {
            for child in node.kind.children() {
                if acc.insert(child) {
                    self.collect_descendants(child, acc);
                }
            }
        }
    }

    pub fn save_to_path(
        &mut self,
        path: impl AsRef<Path>,
    ) -> Result<(), BehaviorGraphDocumentError> {
        let serializable = SerializableDocument::from(&*self);
        let pretty = PrettyConfig::new();
        let text = ron::ser::to_string_pretty(&serializable, pretty)
            .map_err(BehaviorGraphDocumentError::Serialize)?;
        fs::write(path.as_ref(), text).map_err(|err| BehaviorGraphDocumentError::Io {
            action: "write",
            source: err,
        })?;
        self.file_path = Some(path.as_ref().to_path_buf());
        self.dirty = false;
        Ok(())
    }

    pub fn load_from_path(path: impl AsRef<Path>) -> Result<Self, BehaviorGraphDocumentError> {
        let content =
            fs::read_to_string(path.as_ref()).map_err(|err| BehaviorGraphDocumentError::Io {
                action: "read",
                source: err,
            })?;
        let serializable: SerializableDocument =
            ron::from_str(&content).map_err(BehaviorGraphDocumentError::Deserialize)?;
        let mut doc = serializable.into_document();
        doc.file_path = Some(path.as_ref().to_path_buf());
        doc.dirty = false;
        Ok(doc)
    }

    pub fn to_runtime(&self) -> Result<BehaviorGraph, BehaviorGraphDocumentError> {
        let mut visited = HashSet::new();
        let root = self.build_runtime_node(self.root_id, &mut visited)?;
        Ok(BehaviorGraph::new(root))
    }

    fn build_runtime_node(
        &self,
        node_id: NodeId,
        visited: &mut HashSet<NodeId>,
    ) -> Result<BehaviorNode, BehaviorGraphDocumentError> {
        if !visited.insert(node_id) {
            return Err(BehaviorGraphDocumentError::CycleDetected(node_id));
        }

        let node = self
            .node(node_id)
            .ok_or(BehaviorGraphDocumentError::MissingNode(node_id))?;
        let runtime_node = match &node.kind {
            BehaviorGraphNodeKind::Action { name } => BehaviorNode::Action(name.clone()),
            BehaviorGraphNodeKind::Condition { name } => BehaviorNode::Condition(name.clone()),
            BehaviorGraphNodeKind::Sequence { children } => BehaviorNode::Sequence(
                children
                    .iter()
                    .map(|child| self.build_runtime_node(*child, visited))
                    .collect::<Result<Vec<_>, _>>()?,
            ),
            BehaviorGraphNodeKind::Selector { children } => BehaviorNode::Selector(
                children
                    .iter()
                    .map(|child| self.build_runtime_node(*child, visited))
                    .collect::<Result<Vec<_>, _>>()?,
            ),
            BehaviorGraphNodeKind::Parallel {
                children,
                success_threshold,
            } => BehaviorNode::Parallel(
                children
                    .iter()
                    .map(|child| self.build_runtime_node(*child, visited))
                    .collect::<Result<Vec<_>, _>>()?,
                *success_threshold,
            ),
            BehaviorGraphNodeKind::Decorator(data) => {
                let child_id = data
                    .child
                    .ok_or(BehaviorGraphDocumentError::DecoratorMissingChild(node.id))?;
                let child_node = self.build_runtime_node(child_id, visited)?;
                BehaviorNode::Decorator(data.decorator.to_runtime(), Box::new(child_node))
            }
        };
        visited.remove(&node_id);
        Ok(runtime_node)
    }

    pub fn from_runtime(graph: &BehaviorGraph) -> Self {
        let mut doc = BehaviorGraphDocument {
            nodes: Vec::new(),
            root_id: 0,
            next_id: 1,
            file_path: None,
            dirty: true,
        };
        let root_id = doc.import_runtime_node(&graph.root);
        doc.root_id = root_id;
        doc.dirty = false;
        doc
    }

    fn import_runtime_node(&mut self, node: &BehaviorNode) -> NodeId {
        match node {
            BehaviorNode::Action(name) => self.add_node(
                format!("Action: {}", name),
                BehaviorGraphNodeKind::Action { name: name.clone() },
            ),
            BehaviorNode::Condition(name) => self.add_node(
                format!("Condition: {}", name),
                BehaviorGraphNodeKind::Condition { name: name.clone() },
            ),
            BehaviorNode::Sequence(children) => {
                let id = self.allocate_id();
                let child_ids: Vec<NodeId> = children
                    .iter()
                    .map(|child| self.import_runtime_node(child))
                    .collect();
                self.nodes.push(BehaviorGraphNode::new(
                    id,
                    format!("Sequence ({})", child_ids.len()),
                    BehaviorGraphNodeKind::Sequence {
                        children: child_ids,
                    },
                ));
                id
            }
            BehaviorNode::Selector(children) => {
                let id = self.allocate_id();
                let child_ids: Vec<NodeId> = children
                    .iter()
                    .map(|child| self.import_runtime_node(child))
                    .collect();
                self.nodes.push(BehaviorGraphNode::new(
                    id,
                    format!("Selector ({})", child_ids.len()),
                    BehaviorGraphNodeKind::Selector {
                        children: child_ids,
                    },
                ));
                id
            }
            BehaviorNode::Parallel(children, threshold) => {
                let id = self.allocate_id();
                let child_ids: Vec<NodeId> = children
                    .iter()
                    .map(|child| self.import_runtime_node(child))
                    .collect();
                self.nodes.push(BehaviorGraphNode::new(
                    id,
                    format!("Parallel {}", threshold),
                    BehaviorGraphNodeKind::Parallel {
                        children: child_ids,
                        success_threshold: *threshold,
                    },
                ));
                id
            }
            BehaviorNode::Decorator(decorator, child) => {
                let id = self.allocate_id();
                let child_id = self.import_runtime_node(child);
                self.nodes.push(BehaviorGraphNode::new(
                    id,
                    "Decorator".to_string(),
                    BehaviorGraphNodeKind::Decorator(DecoratorNode {
                        decorator: DecoratorKind::from_runtime(decorator),
                        child: Some(child_id),
                    }),
                ));
                id
            }
        }
    }

    pub fn rebuild_next_id(&mut self) {
        self.next_id = self.nodes.iter().map(|node| node.id).max().unwrap_or(0) + 1;
    }
}

#[derive(Debug, Error)]
pub enum BehaviorGraphDocumentError {
    #[error("node {0} not found in behavior document")]
    MissingNode(NodeId),
    #[error("node {0} cannot accept children")]
    NodeCannotHaveChildren(NodeId),
    #[error("cycles detected in graph (node {0})")]
    CycleDetected(NodeId),
    #[error("decorator node {0} is missing required child")]
    DecoratorMissingChild(NodeId),
    #[error("decorator node {0} already has a child assigned")]
    DecoratorAlreadyHasChild(NodeId),
    #[error("root node cannot be deleted")]
    CannotDeleteRoot,
    #[error("serialization error: {0}")]
    Serialize(ron::Error),
    #[error("deserialization error: {0}")]
    Deserialize(ron::error::SpannedError),
    #[error("IO error while attempting to {action}: {source}")]
    Io {
        action: &'static str,
        #[source]
        source: std::io::Error,
    },
}

#[derive(Serialize, Deserialize)]
struct SerializableDocument {
    root_id: NodeId,
    next_id: NodeId,
    nodes: Vec<BehaviorGraphNode>,
}

impl From<&BehaviorGraphDocument> for SerializableDocument {
    fn from(value: &BehaviorGraphDocument) -> Self {
        Self {
            root_id: value.root_id,
            next_id: value.next_id,
            nodes: value.nodes.clone(),
        }
    }
}

impl SerializableDocument {
    fn into_document(self) -> BehaviorGraphDocument {
        BehaviorGraphDocument {
            nodes: self.nodes,
            root_id: self.root_id,
            next_id: self.next_id,
            file_path: None,
            dirty: true,
        }
    }
}
