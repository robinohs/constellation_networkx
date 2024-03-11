use std::fmt::{Display, Formatter};

use pyo3::{types::PyTuple, PyObject, Python, ToPyObject};

use uom::si::{f64::Length, length::kilometer};

use crate::representations::lla::LLA;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default)]
pub struct NodeId(pub u32);

impl Display for NodeId {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl NodeId {
    pub fn next(&self) -> NodeId {
        NodeId(self.0 + 1)
    }
}

impl From<u32> for NodeId {
    fn from(value: u32) -> Self {
        Self(value)
    }
}

impl From<NodeId> for u32 {
    fn from(id: NodeId) -> Self {
        id.0
    }
}

#[derive(Debug, Clone, Copy)]
pub(crate) enum NodeType {
    Satellite,
    Groundstation,
}

impl From<NodeType> for char {
    fn from(typ: NodeType) -> Self {
        match typ {
            NodeType::Satellite => 'S',
            NodeType::Groundstation => 'G',
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct NodePosition {
    x: f64,
    y: f64,
    z: f64,
}

impl NodePosition {
    pub fn new(x: Length, y: Length, z: Length) -> Self {
        NodePosition {
            x: x.get::<kilometer>(),
            y: y.get::<kilometer>(),
            z: z.get::<kilometer>(),
        }
    }

    pub fn get_x(&self) -> f64 {
        self.x
    }

    pub fn get_y(&self) -> f64 {
        self.y
    }

    pub fn get_z(&self) -> f64 {
        self.z
    }
}

impl ToPyObject for NodePosition {
    fn to_object(&self, py: Python<'_>) -> PyObject {
        PyTuple::new(py, vec![self.x, self.y, self.z])
            .to_object(py)
            .into()
    }
}

pub(crate) trait Node {
    fn get_id(&self) -> NodeId;
    fn get_node_type(&self) -> NodeType;
    fn get_position_ecef(&self) -> NodePosition;
    fn get_position_lla(&self) -> LLA;
}
