use nyx_space::{od::ui::GroundStation, time::Epoch, Orbit};
use uom::si::{
    angle::degree,
    f64::{Angle, Length},
    length::kilometer,
};

use crate::{
    constellation::node::{Node, NodeId, NodePosition, NodeType},
    helper::{cosm, earth_frame},
    networkx_graph::Node as NxNode,
    representations::lla::LLA,
    satellite::Satellite,
};

#[derive(Debug, Clone)]
pub(crate) struct Groundstation {
    id: NodeId,
    /// Epoch of the satellite
    epoch: Epoch,
    groundstation: GroundStation,
    min_elevation: Angle,
}
impl Groundstation {
    pub(crate) fn new(
        id: NodeId,
        name: String,
        epoch: Epoch,
        lat: Angle,
        lon: Angle,
        alt: Length,
        min_elevation: Angle,
    ) -> Groundstation {
        let groundstation = GroundStation::from_point(
            name,
            lat.get::<degree>(),
            lon.get::<degree>(),
            alt.get::<kilometer>(),
            earth_frame(),
            cosm(),
        );

        Groundstation {
            id,
            epoch,
            groundstation,
            min_elevation,
        }
    }

    pub fn get_orbit(&self) -> Orbit {
        self.groundstation.to_orbit(self.epoch)
    }

    pub fn is_visible(&self, sat: &Satellite) -> bool {
        let (elevation, _, _) = self.groundstation.elevation_of(&sat.get_orbit());
        let elevation: Angle = Angle::new::<degree>(elevation);
        // println!(
        //     "Elevation between GS({}) and Sat({}) is {}",
        //     self.get_id(),
        //     sat.get_id(),
        //     elevation.get::<degree>()
        // );
        elevation >= self.min_elevation
    }

    pub(crate) fn update_epoch(&mut self, new_epoch: Epoch) {
        self.epoch = new_epoch
    }
}

impl Node for Groundstation {
    fn get_id(&self) -> NodeId {
        self.id
    }

    fn get_node_type(&self) -> NodeType {
        NodeType::Groundstation
    }

    fn get_position_ecef(&self) -> NodePosition {
        let orbit = self.get_orbit();
        let x: Length = Length::new::<kilometer>(orbit.x);
        let y: Length = Length::new::<kilometer>(orbit.y);
        let z: Length = Length::new::<kilometer>(orbit.z);
        NodePosition::new(x, y, z)
    }

    fn get_x(&self) -> Length {
        let orbit = self.get_orbit();
        Length::new::<kilometer>(orbit.x)
    }

    fn get_y(&self) -> Length {
        let orbit = self.get_orbit();
        Length::new::<kilometer>(orbit.y)
    }

    fn get_z(&self) -> Length {
        let orbit = self.get_orbit();
        Length::new::<kilometer>(orbit.z)
    }

    fn get_position_lla(&self) -> LLA {
        let lat = self.groundstation.latitude;
        let lon = self.groundstation.longitude;
        let alt = self.groundstation.height;
        LLA::new(lat, lon, alt)
    }

    fn get_lat(&self) -> Angle {
        Angle::new::<degree>(self.groundstation.latitude)
    }

    fn get_lon(&self) -> Angle {
        Angle::new::<degree>(self.groundstation.latitude)
    }

    fn get_height(&self) -> Length {
        Length::new::<kilometer>(self.groundstation.height)
    }
}

impl From<Groundstation> for NxNode {
    fn from(value: Groundstation) -> Self {
        NxNode {
            id: value.get_id().into(),
        }
    }
}
