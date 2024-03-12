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

#[derive(Debug, Clone, Copy)]
pub(crate) struct SatelliteNeighbors {
    id: NodeId,
    top: NodeId,
    right: NodeId,
    // bottom: NodeId,
    // left: NodeId,
}

impl SatelliteNeighbors {
    /// Returns the NodeId of the satellite which neighbors are encoded here.
    pub(crate) fn get_id(&self) -> NodeId {
        self.id
    }

    /// Returns the NodeId of the top neighbor (same plane, id in plane +1).
    pub(crate) fn get_top(&self) -> NodeId {
        self.top
    }

    /// Returns the NodeId of the right neighbor (same id in plane, plane + 1).
    pub(crate) fn get_right(&self) -> NodeId {
        self.right
    }

    // /// Returns the NodeId of the top neighbor (same plane, id in plane -1).
    // pub(crate) fn get_bottom(&self) -> NodeId {
    //     self.bottom
    // }

    // /// Returns the NodeId of the left neighbor (same id in plane, plane - 1).
    // pub(crate) fn get_left(&self) -> NodeId {
    //     self.left
    // }
}

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
    #[allow(clippy::too_many_arguments)]
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
        self.dt += duration;
    }

    pub fn get_orbit(&self) -> Orbit {
        self.orbit
    }

    pub fn is_ascending(&self) -> bool {
        let z_movement = self.orbit.velocity().z;
        z_movement >= 0.0
    }

    pub fn get_plane(&self) -> u32 {
        self.plane
    }

    pub fn number_in_plane(&self) -> u32 {
        self.number_in_plane
    }

    /// Computes all neighbor NodeIds of the given satellite in the constellation.
    pub(crate) fn get_neighbors(
        &self,
        sats_per_plane: u32,
        number_of_planes: u32,
    ) -> SatelliteNeighbors {
        let top_neighbor =
            ((self.number_in_plane + 1) % sats_per_plane) + self.plane * sats_per_plane;
        let right_neighbor =
            ((self.plane + 1) % number_of_planes) * sats_per_plane + self.number_in_plane;
        // let bottom_neighbor = (self
        //     .number_in_plane
        //     .checked_sub(1)
        //     .unwrap_or(sats_per_plane - 1))
        //     + self.plane * sats_per_plane;
        // let left_neighbor = (self.plane.checked_sub(1).unwrap_or(number_of_planes - 1))
        //     * sats_per_plane
        //     + self.number_in_plane;
        SatelliteNeighbors {
            id: self.id,
            top: top_neighbor.into(),
            right: right_neighbor.into(),
            // bottom: bottom_neighbor.into(),
            // left: left_neighbor.into(),
        }
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
        let x: Length = Length::new::<kilometer>(self.orbit.x);
        let y: Length = Length::new::<kilometer>(self.orbit.y);
        let z: Length = Length::new::<kilometer>(self.orbit.z);
        NodePosition::new(x, y, z)
    }

    fn get_x(&self) -> Length {
        Length::new::<kilometer>(self.orbit.x)
    }

    fn get_y(&self) -> Length {
        Length::new::<kilometer>(self.orbit.y)
    }

    fn get_z(&self) -> Length {
        Length::new::<kilometer>(self.orbit.z)
    }

    fn get_position_lla(&self) -> LLA {
        let lat = self.orbit.geodetic_latitude();
        let lon = self.orbit.geodetic_longitude() - 180.0;
        let alt = self.orbit.geodetic_height();
        LLA::new(lat, lon, alt)
    }

    fn get_lat(&self) -> Angle {
        Angle::new::<degree>(self.orbit.geodetic_latitude())
    }

    fn get_lon(&self) -> Angle {
        Angle::new::<degree>(self.orbit.geodetic_longitude())
    }

    fn get_height(&self) -> Length {
        Length::new::<kilometer>(self.orbit.geodetic_height())
    }
}

impl From<Satellite> for NxNode {
    fn from(value: Satellite) -> Self {
        NxNode {
            id: value.get_id().into(),
        }
    }
}
