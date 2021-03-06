use web_sys::{Element, Node};
use crate::djed::{AnyScope, NodeRef};
use indexmap::set::IndexSet;
use std::collections::HashMap;
use std::fmt;
use std::rc::Rc;
use gloo::events::EventListener;
use super::VNode;
/// The `Listener` trait is an universal implementation of an event listener
/// which is used to bind Rust-listener to JS-listener (DOM).
pub trait Listener {
    /// Returns the name of the event
    fn kind(&self) -> &'static str;
    /// Attaches a listener to the element.
    fn attach(&self, element: &Element) -> EventListener;
}

impl fmt::Debug for dyn Listener {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Listener {{ kind: {} }}", self.kind())
    }
}

/// A list of event listeners.
pub type Listeners = Vec<Rc<dyn Listener>>;

/// A map of attributes.
pub type Attributes = HashMap<String, String>;

/// A set of classes.
#[derive(Debug, Clone, Default)]
pub struct Classes {
    set: IndexSet<String>,
}

impl Classes {
    /// Creates an empty set of classes.
    pub fn new() -> Self {
        Self {
            set: IndexSet::new(),
        }
    }

    /// Adds a class to a set.
    ///
    /// If the provided class has already been added, this method will ignore it.
    pub fn push(&mut self, class: &str) {
        let classes_to_add: Classes = class.into();
        self.set.extend(classes_to_add.set);
    }

    /// Check the set contains a class.
    pub fn contains(&self, class: &str) -> bool {
        self.set.contains(class)
    }

    /// Check the set is empty.
    pub fn is_empty(&self) -> bool {
        self.set.is_empty()
    }

    /// Adds other classes to this set of classes; returning itself.
    ///
    /// Takes the logical union of both `Classes`.
    pub fn extend<T: Into<Classes>>(mut self, other: T) -> Self {
        self.set.extend(other.into().set.into_iter());
        self
    }
}

impl ToString for Classes {
    fn to_string(&self) -> String {
        self.set
            .iter()
            .map(String::as_str)
            .collect::<Vec<&str>>()
            .join(" ")
    }
}

impl From<&str> for Classes {
    fn from(t: &str) -> Self {
        let set = t
            .split_whitespace()
            .map(String::from)
            .filter(|c| !c.is_empty())
            .collect();
        Self { set }
    }
}

impl From<String> for Classes {
    fn from(t: String) -> Self {
        Classes::from(t.as_str())
    }
}

impl From<&String> for Classes {
    fn from(t: &String) -> Self {
        Classes::from(t.as_str())
    }
}

impl<T: AsRef<str>> From<Option<T>> for Classes {
    fn from(t: Option<T>) -> Self {
        t.as_ref()
            .map(|s| <Classes as From<&str>>::from(s.as_ref()))
            .unwrap_or_default()
    }
}

impl<T: AsRef<str>> From<&Option<T>> for Classes {
    fn from(t: &Option<T>) -> Self {
        t.as_ref()
            .map(|s| <Classes as From<&str>>::from(s.as_ref()))
            .unwrap_or_default()
    }
}

impl<T: AsRef<str>> From<Vec<T>> for Classes {
    fn from(t: Vec<T>) -> Self {
        let set = t
            .iter()
            .map(|x| x.as_ref())
            .flat_map(|s| s.split_whitespace())
            .map(String::from)
            .filter(|c| !c.is_empty())
            .collect();
        Self { set }
    }
}

impl PartialEq for Classes {
    fn eq(&self, other: &Self) -> bool {
        self.set.len() == other.set.len() && self.set.iter().eq(other.set.iter())
    }
}

/// Patch for DOM node modification.
#[derive(Debug, PartialEq)]
pub enum Patch<ID, T> {
    Add(ID, T),
    Replace(ID, T),
    Remove(ID),
}

// TODO(#938): What about implementing `VDiff` for `Element`?
// It would make it possible to include ANY element into the tree.
// `Ace` editor embedding for example?

/// This trait provides features to update a tree by calculating a difference against another tree.
pub trait VDiff {
    /// Remove self from parent.
    fn detach(&mut self, parent: &Element);

    /// Scoped diff apply to other tree.
    ///
    /// Virtual rendering for the node. It uses parent node and existing
    /// children (virtual and DOM) to check the difference and apply patches to
    /// the actual DOM representation.
    ///
    /// Parameters:
    /// - `parent_scope`: the parent `Scope` used for passing messages to the
    ///   parent `Component`.
    /// - `parent`: the parent node in the DOM.
    /// - `next_sibling`: the next sibling, used to efficiently find where to
    ///   put the node.
    /// - `ancestor`: the node that this node will be replacing in the DOM. This
    ///   method will _always_ remove the `ancestor` from the `parent`.
    ///
    /// ### Internal Behavior Notice:
    ///
    /// Note that these modify the DOM by modifying the reference that _already_
    /// exists on the `ancestor`. If `self.reference` exists (which it
    /// _shouldn't_) this method will panic.
    ///
    /// The exception to this is obviously `VRef` which simply uses the inner
    /// `Node` directly (always removes the `Node` that exists).
    fn apply(
        &mut self,
        parent_scope: &AnyScope,
        parent: &Element,
        next_sibling: NodeRef,
        ancestor: Option<VNode>,
    ) -> NodeRef;
}

pub fn insert_node(node: &Node, parent: &Element, next_sibling: Option<Node>) {
    match next_sibling {
        Some(next_sibling) => parent
            .insert_before(&node, Some(&next_sibling))
            .expect("failed to insert tag before next sibling"),
        None => parent.append_child(node).expect("failed to append child"),
    };
}

/// Transform properties to the expected type.
pub trait Transformer<FROM, TO> {
    /// Transforms one type to another.
    fn transform(from: FROM) -> TO;
}
