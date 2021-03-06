use super::{VDiff, VNode};
use crate::djed::{AnyScope, NodeRef};
use crate::utils::document;
use log::warn;
use std::cmp::PartialEq;
use web_sys::{Element, Text as TextNode};

/// A type for a virtual
/// [`TextNode`](https://developer.mozilla.org/en-US/docs/Web/API/Document/createTextNode)
/// representation.
#[derive(Clone, Debug)]
pub struct VText {
    /// Contains a text of the node.
    pub text: String,
    /// A reference to the `TextNode`.
    pub reference: Option<TextNode>,
}

impl VText {
    /// Creates new virtual text node with a content.
    pub fn new(text: String) -> Self {
        VText {
            text,
            reference: None,
        }
    }
}

impl VDiff for VText {
    /// Remove VText from parent.
    fn detach(&mut self, parent: &Element) {
        let node = self
            .reference
            .take()
            .expect("tried to remove not rendered VText from DOM");
        if parent.remove_child(&node).is_err() {
            warn!("Node not found to remove VText");
        }
    }

    /// Renders virtual node over existing `TextNode`, but only if value of text had changed.
    fn apply(
        &mut self,
        _parent_scope: &AnyScope,
        parent: &Element,
        next_sibling: NodeRef,
        ancestor: Option<VNode>,
    ) -> NodeRef {
        if let Some(mut ancestor) = ancestor {
            if let VNode::VText(mut vtext) = ancestor {
                self.reference = vtext.reference.take();
                let text_node = self
                    .reference
                    .clone()
                    .expect("Rendered VText nodes should have a ref");
                if self.text != vtext.text {
                    text_node.set_node_value(Some(&self.text));
                }

                return NodeRef::new(text_node.into());
            }

            ancestor.detach(parent);
        }

        let text_node = document().create_text_node(&self.text);
        super::insert_node(&text_node, parent, next_sibling.get());
        self.reference = Some(text_node.clone());
        NodeRef::new(text_node.into())
    }
}

impl PartialEq for VText {
    fn eq(&self, other: &VText) -> bool {
        self.text == other.text
    }
}
