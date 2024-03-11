use nyx_space::{
    cosmic::Frame,
    dynamics::OrbitalDynamics,
    propagators::Propagator,
    time::{Duration, Epoch},
    Orbit,
};
use pyo3::pyclass;
use uom::si::{
    angle::degree,
    f64::{Angle, Length, Time},
    length::kilometer,
    time::millisecond,
};

use once_cell::sync::Lazy;

use crate::{
    constellation::node::{Node, NodeId, NodePosition, NodeType},
    networkx_graph::Node as NxNode,
    representations::lla::LLA,
};

// Load the NASA NAIF DE438 planetary ephemeris.
// static COSM: Lazy<Arc<Cosm>> = Lazy::new(|| Cosm::de438());
static PROPAGATOR: Lazy<
    Propagator<'_, OrbitalDynamics<'_>, nyx_space::propagators::RSSCartesianStep>,
> = Lazy::new(|| Propagator::default(OrbitalDynamics::two_body()));

#[pyclass(module = "satellite")]
#[derive(Debug, Clone)]
pub struct Satellite {
    /// Identifier,
    id: NodeId,
    /// Plane index
    plane: u32,
    /// Index of satellite in plane
    number_in_plane: u32,
    /// Epoch of the satellite
    dt: Epoch,
    /// Orbit of the satellite
    orbit: Orbit,
}

impl Satellite {
    pub fn new(
        id: NodeId,
        aol: Angle,
        raan: Angle,
        plane: u32,
        number_in_plane: u32,
        altitude: Length,
        inclination: Angle,
        dt: Epoch,
        frame: Frame,
    ) -> Satellite {
        let orbit = Orbit::keplerian_altitude(
            altitude.get::<kilometer>(),
            0.0,
            inclination.get::<degree>(),
            raan.get::<degree>(),
            0.0,
            aol.get::<degree>(),
            dt,
            frame,
        );
        Satellite {
            id,
            plane,
            number_in_plane,
            dt,
            orbit,
        }
    }

    /// Propagates the satellite orbit for a given duration using the two-body propagator.
    pub fn propagate(&mut self, step: Time) {
        let duration = Duration::from_f64(
            step.get::<millisecond>(),
            nyx_space::time::Unit::Millisecond,
        );
        // println!(
        //     "Propagate SAT({}-{}) for {}!",
        //     self.plane, self.number_in_plane, duration
        // );
        let mut prop = PROPAGATOR.with(self.orbit);
        self.orbit = prop.for_duration(duration).unwrap();
        self.dt = self.dt + duration;
    }

    pub fn get_orbit(&self) -> Orbit {
        self.orbit
    }

    pub fn get_top_right_neighbors(
        &self,
        sats_per_plane: u32,
        number_of_planes: u32,
    ) -> Vec<(NodeId, NodeId)> {
        let top_neighbor =
            ((self.number_in_plane + 1) % sats_per_plane) + self.plane * sats_per_plane;
        let right_neighbor =
            ((self.plane + 1) % number_of_planes) * sats_per_plane + self.number_in_plane;
        vec![
            (self.id, NodeId(top_neighbor)),
            (self.id, NodeId(right_neighbor)),
        ]
    }
}

impl Node for Satellite {
    fn get_id(&self) -> NodeId {
        self.id
    }

    fn get_node_type(&self) -> NodeType {
        NodeType::Satellite
    }

    fn get_position_ecef(&self) -> NodePosition {
        let orbit = self.orbit;
        let x: Length = Length::new::<kilometer>(orbit.x);
        let y: Length = Length::new::<kilometer>(orbit.y);
        let z: Length = Length::new::<kilometer>(orbit.z);
        NodePosition::new(x, y, z)
    }

    fn get_position_lla(&self) -> LLA {
        let lat = self.get_orbit().geodetic_latitude();
        let lon = self.get_orbit().geodetic_longitude() - 180.0;
        let alt = self.get_orbit().geodetic_height();
        LLA::new(lat, lon, alt)
    }
}

impl From<Satellite> for NxNode {
    fn from(value: Satellite) -> Self {
        NxNode {
            id: value.get_id().into(),
        }
    }
}
