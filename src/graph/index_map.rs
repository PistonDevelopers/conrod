
use std::collections::HashMap;
use std::ops::Index;
use super::NodeIndex;
use widget::WidgetId;


/// Maps a WidgetId given by the user to a NodeIndex into the Graph (and vice versa).
#[derive(Debug)]
pub struct IndexMap {
    nodes: HashMap<WidgetId, NodeIndex>,
    widgets: HashMap<NodeIndex, WidgetId>,
}


impl IndexMap {

    /// Construct an IndexMap with the given capacity.
    pub fn with_capacity(capacity: usize) -> IndexMap {
        IndexMap {
            nodes: HashMap::with_capacity(capacity),
            widgets: HashMap::with_capacity(capacity),
        }
    }

    /// Add a WidgetId NodeIndex pair.
    pub fn insert(&mut self, widget_id: WidgetId, node_idx: NodeIndex) {
        self.nodes.insert(widget_id, node_idx);
        self.widgets.insert(node_idx, widget_id);
    }

    /// Return Some NodeIndex for the given WidgetId if there is one.
    #[inline]
    pub fn get_node_index(&self, id: WidgetId) -> Option<NodeIndex> {
        self.nodes.get(&id).map(|&idx| idx)
    }

    /// Return Some WidgetId for the given NodeIndex if there is one.
    #[inline]
    pub fn get_widget_id(&self, idx: NodeIndex) -> Option<WidgetId> {
        self.widgets.get(&idx).map(|&id| id)
    }

    /// Takes an arbitrary GraphIndex and converts it to a node index.
    #[inline]
    pub fn to_node_index<I: GraphIndex>(&self, idx: I) -> Option<NodeIndex> {
        idx.to_node_index(self)
    }

    /// Takes an arbitrary GraphIndex and converts it to a node index.
    #[inline]
    pub fn to_widget_id<I: GraphIndex>(&self, idx: I) -> Option<WidgetId> {
        idx.to_widget_id(self)
    }

}

impl Index<WidgetId> for IndexMap {
    type Output = NodeIndex;
    fn index<'a>(&'a self, id: WidgetId) -> &'a NodeIndex {
        self.nodes.get(&id).expect("No NodeIndex for the given WidgetId")
    }
}

impl Index<NodeIndex> for IndexMap {
    type Output = WidgetId;
    fn index<'a>(&'a self, idx: NodeIndex) -> &'a WidgetId {
        self.widgets.get(&idx).expect("No WidgetId for the given NodeIndex")
    }
}


/// A trait for being generic over both WidgetId and NodeIndex.
/// Each method should only return `Some` if they are contained as a key within the given IndexMap.
pub trait GraphIndex: ::std::fmt::Debug + Copy + Clone {
    fn to_widget_id(self, map: &IndexMap) -> Option<WidgetId>;
    fn to_node_index(self, map: &IndexMap) -> Option<NodeIndex>;
    fn from_idx<I: GraphIndex>(other: I, map: &IndexMap) -> Option<Self>;
}

impl GraphIndex for WidgetId {
    #[inline]
    fn to_widget_id(self, map: &IndexMap) -> Option<WidgetId> {
        if map.nodes.contains_key(&self) { Some(self) } else { None }
    }
    #[inline]
    fn to_node_index(self, map: &IndexMap) -> Option<NodeIndex> {
        map.get_node_index(self)
    }
    #[inline]
    fn from_idx<I: GraphIndex>(other: I, map: &IndexMap) -> Option<Self> {
        other.to_widget_id(map)
    }
}

impl GraphIndex for NodeIndex {
    #[inline]
    fn to_widget_id(self, map: &IndexMap) -> Option<WidgetId> {
        map.get_widget_id(self)
    }
    #[inline]
    fn to_node_index(self, map: &IndexMap) -> Option<NodeIndex> {
        if map.widgets.contains_key(&self) { Some(self) } else { None }
    }
    #[inline]
    fn from_idx<I: GraphIndex>(other: I, map: &IndexMap) -> Option<Self> {
        other.to_node_index(map)
    }
}

