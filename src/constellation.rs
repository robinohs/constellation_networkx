use crate::groundstation::Groundstation;
use crate::helper::{self, nullpi, twopi};

use crate::networkx_graph::{Graph as NxGraph, Node as NxNode};

use crate::representations::undirected_link::{LinkType, UndirectedLink};
use crate::satellite::Satellite;
use itertools::Itertools;
use nyx_space::time::{Duration, Epoch};

use pyo3::pyclass;
use rayon::prelude::*;
use uom::si::f64::Time;

use uom::si::time::millisecond;
use uom::si::{
    f64::{Angle, Length},
    length::kilometer,
};

use self::node::{Node, NodeId};

pub(crate) mod node;

#[pyclass(module = "constellation")]
#[derive(Debug, Clone)]
pub struct Constellation {
    next_free_id: NodeId,
    number_of_satellites: u32,
    number_of_planes: u32,
    satellites: Vec<Satellite>,
    groundstations: Vec<Groundstation>,
    min_elevation: Angle,
    links: Vec<UndirectedLink>,
    epoch: Epoch,
}

impl Constellation {
    /// Creates a new single-shell constellation and the associated satellite definitions. <br/>
    /// IMPORTANT: Does not yet propagate the satellite orbits.
    ///
    /// # Arguments
    ///
    /// * `number_of_satellites` - The number of satellites in the constellation.
    /// * `number_of_planes` - The number of planes in the constellation.
    /// * `altitude` - The altitude of the shell.
    /// * `inclination` - The inclination of the satellite orbits.
    ///
    /// # Panics
    ///
    /// Panics if the number of satellites is not divisible by the number of planes. <br/>
    /// Will also panic if the number of planes or satellites is equal to 0, or if the the altitude is equal or below 0km.
    pub fn new(
        number_of_satellites: u32,
        number_of_planes: u32,
        inter_plane_spacing: u32,
        altitude: Length,
        inclination: Angle,
        dt: Epoch,
        min_elevation: Angle,
    ) -> Self {
        // validate arguments
        assert!(number_of_satellites > 0);
        assert!(number_of_planes > 0);
        assert!(number_of_satellites % number_of_planes == 0);
        assert!(altitude.get::<kilometer>() > 0.0);

        let frame = helper::earth_frame();
        let sats_per_plane = number_of_satellites / number_of_planes;

        // ΔΩ = 2𝜋/𝑃 in [0,2𝜋]
        let raan_delta: Angle = twopi() / number_of_planes as f64;
        // ΔΦ = 2𝜋/Q in [0,2𝜋]
        let phase_difference: Angle = twopi() / sats_per_plane as f64;
        // Δ𝑓 = 2𝜋𝐹/𝑃𝑄 in [0,2𝜋)
        let phase_offset: Angle =
            (twopi() * inter_plane_spacing as f64) / number_of_satellites as f64;
        assert!(raan_delta >= nullpi() && raan_delta <= twopi());
        assert!(phase_difference >= nullpi() && phase_difference <= twopi());
        assert!(phase_offset >= nullpi() && phase_offset < twopi());

        // create satellites
        let mut satellites = Vec::with_capacity(number_of_satellites as usize);
        // iterate over planes
        for plane in 0..number_of_planes {
            // calculate and validate raan of this plane
            let raan: Angle = raan_delta * plane as f64;
            assert!(raan >= nullpi() && raan <= twopi());
            // the phasing offset of this plane which depends on Δ𝑓 and the index of the plane
            let plane_phase_offset: Angle = phase_offset * plane as f64;
            // iterate over satellites in plane
            for number_in_plane in 0..sats_per_plane {
                let id = NodeId((number_in_plane + plane * sats_per_plane) as u32);
                // phase offset for this satellite
                let sat_phase: Angle = phase_difference * number_in_plane as f64;
                // argument of latitude is equal to the base offset of this plane + the phase of the satellite, mod 360.0
                let aol: Angle = (plane_phase_offset + sat_phase) % twopi();
                assert!(aol >= nullpi() && aol < twopi());

                // println!(
                //     "Create satellite({}-{}) at RAAN({}°), AOL({}°). ID: {}, TN: {:?}, BN: {:?}, LN: {:?}, RN: {:?}",
                //     plane,
                //     number_in_plane,
                //     raan.get::<degree>(),
                //     aol.get::<degree>(),
                //     id,
                //     top_neighbor,
                //     bottom_neighbor,
                //     left_neighbor,
                //     right_neighbor
                // );

                let satellite = Satellite::new(
                    id,
                    aol,
                    raan,
                    plane,
                    number_in_plane,
                    altitude,
                    inclination,
                    dt,
                    frame,
                );
                satellites.push(satellite);
            }
        }

        // create constellation
        let mut cstl = Constellation {
            next_free_id: number_of_satellites.into(),
            number_of_satellites,
            number_of_planes,
            satellites,
            groundstations: vec![],
            min_elevation,
            links: vec![],
            epoch: dt,
        };
        cstl.recalculate_isls();
        cstl
    }

    /// Returns the number of nodes in this constellation,
    /// which is the sum of satellites and groundstations.
    pub fn node_count(&self) -> u32 {
        (self.satellites.len() + self.groundstations.len())
            .try_into()
            .unwrap()
    }

    /// Propagates all satellites in this constellation for the given step.
    pub fn propagate(&mut self, step: Time) {
        // increase epoch
        self.epoch += Duration::from_f64(
            step.get::<millisecond>(),
            nyx_space::time::Unit::Millisecond,
        );
        self.satellites
            .par_iter_mut()
            .for_each(|sat| sat.propagate(step));
        self.groundstations
            .par_iter_mut()
            .for_each(|gs| gs.update_epoch(self.epoch));
        self.recalculate_ground_visibilities();
    }

    /// Calculates the distance between two nodes given by their IDs.
    pub fn distance(&self, first: NodeId, second: NodeId) -> Length {
        let first = self.get_node(first).get_position_ecef();
        let second = self.get_node(second).get_position_ecef();

        let fp = (first.get_x(), first.get_y(), first.get_z());
        let sp = (second.get_x(), second.get_y(), second.get_z());
        Length::new::<kilometer>(f64::sqrt(
            (fp.0 - sp.0).powf(2.0) + (fp.1 - sp.1).powf(2.0) + (fp.2 - sp.2).powf(2.0),
        ))
    }

    /// Adds a ground station to the constellation.
    /// The ground station is assigned the next free ID in the constellation.
    ///
    /// # Arguments
    ///
    /// * `lat` - Latitude of the ground station.
    /// * `lon` - Longitude of the ground station.
    /// * `alt` - The altitude of the ground station (Height above mean sea level).
    ///
    pub fn add_groundstation(&mut self, name: String, lat: Angle, lon: Angle, alt: Length) {
        let id = self.next_id();
        let groundstation =
            Groundstation::new(id, name, self.epoch, lat, lon, alt, self.min_elevation);
        self.groundstations.push(groundstation);
    }

    /// Recalculates the visibility of the satellites for the constellation using the minimal elevation assigned to the constellation.
    pub fn recalculate_ground_visibilities(&mut self) {
        self.links.retain(|link| link.link_type() == LinkType::ISL);
        let mut pairs: Vec<UndirectedLink> = self
            .groundstations
            .iter()
            .cartesian_product(&self.satellites)
            // .par_bridge()
            .filter(|(gs, sat)| gs.is_visible(&sat))
            .map(|(gs, sat)| {
                let distance: Length = self.distance(gs.get_id(), sat.get_id());
                UndirectedLink::new_gsl(gs.get_id(), sat.get_id(), distance)
            })
            .collect();
        self.links.append(&mut pairs);
    }

    pub(crate) fn get_nodes(&self) -> Vec<Box<&dyn Node>> {
        (0..self.node_count())
            .map_into::<NodeId>()
            .map(|id| self.get_node(id))
            .collect_vec()
    }

    fn recalculate_isls(&mut self) {
        self.links.retain(|link| link.link_type() == LinkType::GSL);
        let sats_per_plane = self.number_of_satellites / self.number_of_planes;
        let mut pairs: Vec<UndirectedLink> = self
            .satellites
            .iter()
            // get top and right neighbor
            .flat_map(|sat| sat.get_top_right_neighbors(sats_per_plane, self.number_of_planes))
            // calculate distance and create link
            .map(|(sat_a, sat_b)| {
                let distance: Length = self.distance(sat_a, sat_b);
                UndirectedLink::new_isl(sat_a, sat_b, distance)
            })
            .collect();
        self.links.append(&mut pairs);
    }

    /// Returns the next free ID for further usage.
    ///
    /// ### Important (Side effect)
    /// This method increases the next free ID after returning the previous one.<br/>
    /// Thus, the caller must make sure:
    /// - no ID is assigned twice (is unique)
    /// - each ID is actually assigned since otherwise there could be intermediate unused IDs
    fn next_id(&mut self) -> NodeId {
        let tmp = self.next_free_id;
        self.next_free_id = tmp.next();
        tmp
    }

    fn get_node(&self, id: NodeId) -> Box<&dyn Node> {
        assert!(id < self.next_free_id);
        if id < NodeId(self.number_of_satellites) {
            Box::new(self.get_satellite(id))
        } else {
            Box::new(self.get_groundstation(id))
        }
    }

    fn get_satellite(&self, id: NodeId) -> &Satellite {
        assert!(id < NodeId(self.number_of_satellites));
        let index = id.0 as usize;
        self.satellites.get(index).unwrap()
    }

    fn get_groundstation(&self, id: NodeId) -> &Groundstation {
        assert!(id >= NodeId(self.number_of_satellites));
        assert!(id < self.next_free_id);
        // calculated index in ground station vector
        let index = (id.0 - self.number_of_satellites) as usize;
        self.groundstations.get(index).unwrap()
    }
}

impl Into<NxGraph> for Constellation {
    fn into(self) -> NxGraph {
        let nodes = [
            self.satellites
                .iter()
                .cloned()
                .map_into::<NxNode>()
                .collect_vec(),
            self.groundstations
                .iter()
                .cloned()
                .map_into::<NxNode>()
                .collect_vec(),
        ]
        .concat();
        let links = self.links.iter().cloned().map_into().collect_vec();
        NxGraph::new(nodes, links)
    }
}
