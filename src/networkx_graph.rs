// {
//     "directed": false,
//     "multigraph": false,
//     "graph": {},
//     "nodes": [
//         {"id": 0},
//         {"id": 1},
//         {"id": 2},
//         {"id": 3},
//         {"id": 4},
//         {"id": 5},
//         {"id": 6},
//         {"id": 7},
//         {"id": 8}
//     ],
//     "links": [
//         {"weight": 2, "source": 0, "target": 1},
//         {"weight": 3, "source": 0, "target": 2},
//         {"weight": 7, "source": 0, "target": 3},
//         {"weight": 4, "source": 1, "target": 3},
//         {"weight": 6, "source": 1, "target": 4},
//         {"weight": 5, "source": 2, "target": 3},
//         {"weight": 7, "source": 2, "target": 5},
//         {"weight": 2, "source": 3, "target": 6},
//         {"weight": 1, "source": 3, "target": 4},
//         {"weight": 2, "source": 3, "target": 5},
//         {"weight": 3, "source": 4, "target": 6},
//         {"weight": 2, "source": 4, "target": 7},
//         {"weight": 1, "source": 5, "target": 6},
//         {"weight": 4, "source": 5, "target": 7},
//         {"weight": 3, "source": 6, "target": 8},
//         {"weight": 5, "source": 7, "target": 8}
//     ]
//  }

use pyo3::{pyclass, types::PyDict, PyObject, Python, ToPyObject};
use serde::Serialize;

#[pyclass(module = "graph")]
#[derive(Debug, Clone, Serialize)]
pub struct Graph {
    pub directed: bool,
    pub multigraph: bool,
    pub graph: InternalGraph,
    pub nodes: Vec<Node>,
    pub links: Vec<Link>,
}

#[pyclass(module = "internal_graph")]
#[derive(Debug, Clone, Serialize)]
pub struct InternalGraph {}

#[pyclass(module = "node")]
#[derive(Debug, Clone, Copy, Serialize)]
pub struct Node {
    pub id: u32,
}

#[derive(Debug, Clone, Copy, Serialize)]
pub struct Link {
    pub weight: i32,
    pub source: u32,
    pub target: u32,
}

impl Graph {
    pub fn new(nodes: Vec<Node>, links: Vec<Link>) -> Self {
        Graph {
            directed: false,
            multigraph: false,
            graph: InternalGraph {},
            nodes,
            links,
        }
    }
}

impl ToPyObject for Graph {
    fn to_object(&self, py: Python<'_>) -> PyObject {
        let module = py.import("networkx").unwrap();
        let graph = module.getattr("Graph").unwrap();
        let graph = graph.call0().unwrap();

        // add nodes
        self.nodes.iter().for_each(|node| {
            graph.call_method("add_node", (node.id,), None).unwrap();
        });

        // add edges
        self.links.iter().for_each(|link| {
            let kwargs = PyDict::new(py);
            kwargs.set_item("u_of_edge", link.source).unwrap();
            kwargs.set_item("v_of_edge", link.target).unwrap();
            kwargs.set_item("weight", link.weight).unwrap();
            graph.call_method("add_edge", (), Some(kwargs)).unwrap();
        });

        graph.to_object(py)
    }
}
