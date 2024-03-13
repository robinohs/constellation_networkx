use constellation::{Constellation, ConstellationType};

use networkx_graph::Graph as NxGraph;
use nyx_space::time::Epoch;
use pyo3::{
    prelude::*,
    types::{PyDict, PyTuple},
};

use uom::si::{
    angle::degree,
    f64::{Angle, Length},
    length::kilometer,
};

mod constellation;
mod groundstation;
mod helper;
mod networkx_graph;
mod representations;
mod satellite;

#[pyfunction]
fn create_constellation(
    satellites: u32,
    planes: u32,
    ipc: u32,
    altitude: u32,
    inclination: f64,
    min_elevation: f64,
    constellation_type: ConstellationType,
) -> PyResult<Constellation> {
    let altitude: Length = Length::new::<kilometer>(altitude as f64);
    let inclination: Angle = Angle::new::<degree>(inclination);
    let min_elevation: Angle = Angle::new::<degree>(min_elevation);
    let epoch = Epoch::now().unwrap();
    Ok(Constellation::new(
        constellation_type,
        satellites,
        planes,
        ipc,
        altitude,
        inclination,
        epoch,
        min_elevation,
    ))
}

#[pyfunction]
fn extract_graph<'a>(py: Python<'a>, constellation: &'a Constellation) -> PyResult<&'a PyAny> {
    let internal_graph: NxGraph = constellation.clone().into();
    Ok(internal_graph.to_object(py).into_ref(py))
}

#[pyfunction]
fn extract_positions_3d<'a>(
    py: Python<'a>,
    constellation: &'a Constellation,
) -> PyResult<&'a PyDict> {
    let dict = PyDict::new(py);
    constellation
        .get_nodes()
        .into_iter()
        .map(|node| {
            (
                node.get_id(),
                node.get_node_type(),
                node.get_position_ecef(),
            )
        })
        .for_each(|(id, typ, pos)| {
            let id: u32 = id.into();
            let typ = char::from(typ).to_object(py);
            let xyz = PyTuple::new(py, vec![pos.get_x(), pos.get_y(), pos.get_z()]).to_object(py);
            dict.set_item(id, PyTuple::new(py, vec![typ, xyz])).unwrap();
        });
    Ok(dict)
}

#[pyfunction]
fn project_3d_positions<'a>(
    py: Python<'a>,
    constellation: &'a Constellation,
) -> PyResult<&'a PyDict> {
    let dict = PyDict::new(py);
    constellation
        .get_nodes()
        .into_iter()
        .map(|node| (node.get_id(), node.get_node_type(), node.get_position_lla()))
        .for_each(|(id, typ, lla)| {
            let id: u32 = id.into();
            let typ = char::from(typ).to_object(py);
            let lla =
                PyTuple::new(py, vec![lla.get_lat(), lla.get_lon(), lla.get_alt()]).to_object(py);
            dict.set_item(id, PyTuple::new(py, vec![typ, lla])).unwrap();
        });
    Ok(dict)
}

/// A Python module implemented in Rust.
#[pymodule]
fn cstl_ntwkx(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<ConstellationType>()?;
    m.add_class::<Constellation>()?;
    m.add_function(wrap_pyfunction!(create_constellation, m)?)?;
    m.add_function(wrap_pyfunction!(extract_graph, m)?)?;
    m.add_function(wrap_pyfunction!(extract_positions_3d, m)?)?;
    m.add_function(wrap_pyfunction!(project_3d_positions, m)?)?;
    Ok(())
}
