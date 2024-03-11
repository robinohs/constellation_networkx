#[derive(Debug, Clone, Copy)]
pub(crate) struct LLA {
    lat: f64,
    lon: f64,
    alt: f64,
}

impl LLA {
    pub(crate) fn new(lat: f64, lon: f64, alt: f64) -> LLA {
        Self { lat, lon, alt }
    }

    pub(crate) fn get_lat(&self) -> f64 {
        self.lat
    }

    pub(crate) fn get_lon(&self) -> f64 {
        self.lon
    }

    pub(crate) fn get_alt(&self) -> f64 {
        self.alt
    }
}
