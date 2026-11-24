//! Mutation records, the raw material for `MutationObserver`.
//!
//! When recording is enabled
//! ([`Document::set_mutation_recording`](crate::Document::set_mutation_recording)),
//! every DOM mutation appends a [`MutationRecord`] to a buffer that the
//! binding layer drains with
//! [`Document::take_mutation_records`](crate::Document::take_mutation_records).
//! Records always carry old values; the observer layer filters by its
//! options (`attributeOldValue`, `subtree`, ...). Recording is off by default
//! and costs one branch per mutation when off.

use crate::NodeId;

/// The `MutationRecord.type` of a record.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MutationKind {
    /// `"childList"`: children were added and/or removed.
    ChildList,
    /// `"attributes"`: an attribute was set, changed or removed.
    Attributes,
    /// `"characterData"`: a text, comment or PI node's data changed.
    CharacterData,
}

impl MutationKind {
    /// The `MutationRecord.type` string.
    pub fn as_str(self) -> &'static str {
        match self {
            MutationKind::ChildList => "childList",
            MutationKind::Attributes => "attributes",
            MutationKind::CharacterData => "characterData",
        }
    }
}

/// One mutation, shaped like the DOM `MutationRecord` interface.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MutationRecord {
    /// The record type.
    pub kind: MutationKind,
    /// The node the mutation happened to (the parent for `childList`).
    pub target: NodeId,
    /// Nodes added (`childList` only), in tree order.
    pub added_nodes: Vec<NodeId>,
    /// Nodes removed (`childList` only), in former tree order.
    pub removed_nodes: Vec<NodeId>,
    /// The sibling before the added/removed nodes (`childList` only).
    pub previous_sibling: Option<NodeId>,
    /// The sibling after the added/removed nodes (`childList` only).
    pub next_sibling: Option<NodeId>,
    /// The attribute's local name (`attributes` only).
    pub attribute_name: Option<String>,
    /// The attribute's namespace (`attributes` only; `None` for no namespace).
    pub attribute_namespace: Option<String>,
    /// The previous attribute value or character data (`None` if the
    /// attribute did not exist).
    pub old_value: Option<String>,
}

impl MutationRecord {
    pub(crate) fn child_list(
        target: NodeId,
        added_nodes: Vec<NodeId>,
        removed_nodes: Vec<NodeId>,
        previous_sibling: Option<NodeId>,
        next_sibling: Option<NodeId>,
    ) -> MutationRecord {
        MutationRecord {
            kind: MutationKind::ChildList,
            target,
            added_nodes,
            removed_nodes,
            previous_sibling,
            next_sibling,
            attribute_name: None,
            attribute_namespace: None,
            old_value: None,
        }
    }

    pub(crate) fn attribute(
        target: NodeId,
        name: &str,
        namespace: &str,
        old_value: Option<String>,
    ) -> MutationRecord {
        MutationRecord {
            kind: MutationKind::Attributes,
            target,
            added_nodes: Vec::new(),
            removed_nodes: Vec::new(),
            previous_sibling: None,
            next_sibling: None,
            attribute_name: Some(name.to_string()),
            attribute_namespace: (!namespace.is_empty()).then(|| namespace.to_string()),
            old_value,
        }
    }

    pub(crate) fn character_data(target: NodeId, old_value: String) -> MutationRecord {
        MutationRecord {
            kind: MutationKind::CharacterData,
            target,
            added_nodes: Vec::new(),
            removed_nodes: Vec::new(),
            previous_sibling: None,
            next_sibling: None,
            attribute_name: None,
            attribute_namespace: None,
            old_value: Some(old_value),
        }
    }
}
