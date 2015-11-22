
use std::collections::HashMap;
use std::ops::Index;
use super::NodeIndex;
use widget;
use widget::Id as WidgetId;


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
    /// If not one already, convert Self to a WidgetId if it exists within the IndexMap.
    fn to_widget_id(self, map: &IndexMap) -> Option<WidgetId>;
    /// If not one already, convert Self to a NodeIndex if it exists within the IndexMap.
    fn to_node_index(self, map: &IndexMap) -> Option<NodeIndex>;
    /// Convert some GraphIndex type to Self.
    fn from_idx<I: GraphIndex>(other: I, map: &IndexMap) -> Option<Self>;
}


impl GraphIndex for WidgetId {
    fn to_widget_id(self, _map: &IndexMap) -> Option<WidgetId> {
        Some(self)
    }
    fn to_node_index(self, map: &IndexMap) -> Option<NodeIndex> {
        map.get_node_index(self)
    }
    fn from_idx<I: GraphIndex>(other: I, map: &IndexMap) -> Option<Self> {
        other.to_widget_id(map)
    }
}

impl GraphIndex for NodeIndex {
    fn to_widget_id(self, map: &IndexMap) -> Option<WidgetId> {
        map.get_widget_id(self)
    }
    fn to_node_index(self, _map: &IndexMap) -> Option<NodeIndex> {
        Some(self)
    }
    fn from_idx<I: GraphIndex>(other: I, map: &IndexMap) -> Option<Self> {
        other.to_node_index(map)
    }
}

impl GraphIndex for widget::Index {

    /// Coerce a widget::Index into an Option<NodeIndex>.
    /// If the Index is the Internal variant, that idx will be used directly.
    /// If the Index is the Public variant, the index_map will be used to find the matching
    /// NodeIndex.
    fn to_node_index(self, map: &IndexMap) -> Option<NodeIndex> {
        match self {
            widget::Index::Internal(idx) => Some(idx),
            widget::Index::Public(id) => id.to_node_index(map),
        }
    }

    /// Coerce a widget::Index into an Option<WidgetId>.
    /// If the Index is the Public variant, that id will be used directly.
    /// If the Index is the Internal variant, the index_map will be used to find the matching
    /// WidgetId.
    fn to_widget_id(self, map: &IndexMap) -> Option<WidgetId> {
        match self {
            widget::Index::Internal(idx) => idx.to_widget_id(map),
            widget::Index::Public(id) => Some(id),
        }
    }

    /// Construct a widget::Index from some GraphIndex.
    /// First tries to construct a Public variant by checking the IndexMap for a matching WidgetId.
    /// If not WidgetId is found, then tries to find a matching NodeIndex.
    fn from_idx<I: GraphIndex>(other: I, map: &IndexMap) -> Option<Self> {
        other.to_widget_id(map).map(|id| widget::Index::Public(id))
            .or_else(|| other.to_node_index(map).map(|idx| widget::Index::Internal(idx)))
    }

}
