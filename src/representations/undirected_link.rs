use uom::si::{f64::Length, length::kilometer};

use crate::{constellation::node::NodeId, networkx_graph::Link as NxLink};

#[derive(Debug, Clone, Copy, PartialEq, Hash)]
#[allow(clippy::upper_case_acronyms)]
pub enum LinkType {
    ISL,
    GSL,
}

#[derive(Debug, Clone, Copy)]
pub struct UndirectedLink {
    link_type: LinkType,
    first: NodeId,
    second: NodeId,
    distance: Length,
}
impl UndirectedLink {
    pub(crate) fn new_isl(first: NodeId, second: NodeId, distance: Length) -> UndirectedLink {
        UndirectedLink {
            link_type: LinkType::ISL,
            first,
            second,
            distance,
        }
    }

    pub(crate) fn new_gsl(first: NodeId, second: NodeId, distance: Length) -> UndirectedLink {
        UndirectedLink {
            link_type: LinkType::GSL,
            first,
            second,
            distance,
        }
    }

    pub(crate) fn link_type(&self) -> LinkType {
        self.link_type
    }
}

impl From<UndirectedLink> for NxLink {
    fn from(value: UndirectedLink) -> Self {
        NxLink {
            source: value.first.into(),
            target: value.second.into(),
            weight: value.distance.get::<kilometer>().round() as i32,
        }
    }
}
