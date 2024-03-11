use std::sync::Arc;

use nyx_space::cosmic::{Cosm, Frame};
use once_cell::sync::Lazy;
use uom::si::{angle::degree, f64::Angle};

// Load the NASA NAIF DE438 planetary ephemeris.
static COSM: Lazy<Arc<Cosm>> = Lazy::new(|| Cosm::de438());

pub(crate) fn nullpi() -> Angle {
    Angle::new::<degree>(0.0)
}

pub(crate) fn twopi() -> Angle {
    Angle::new::<degree>(360.0)
}

pub(crate) fn earth_frame() -> Frame {
    // Grab the Earth Mean Equator J2000 frame
    COSM.frame("EME2000")
}

pub(crate) fn cosm() -> Arc<Cosm> {
    COSM.to_owned()
}
