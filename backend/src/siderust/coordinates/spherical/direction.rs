use qtty::Degrees;

/// Minimal ICRS direction type used by the repo.
#[derive(Debug, Clone, Copy)]
pub struct ICRS {
    ra: Degrees,
    dec: Degrees,
}

impl ICRS {
    pub fn new(ra: Degrees, dec: Degrees) -> Self {
        Self { ra, dec }
    }

    pub fn ra(&self) -> Degrees {
        self.ra
    }

    pub fn dec(&self) -> Degrees {
        self.dec
    }
}
