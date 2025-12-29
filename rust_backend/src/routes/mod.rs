pub mod compare;
pub mod distribution;
pub mod insights;
pub mod landing;
pub mod skymap;
pub mod timeline;
pub mod trends;
pub mod validation;
pub mod visibility;

use pyo3::prelude::*;

/// Register all route-specific functions, classes and constants with the Python module.
/// This centralizes ownership of route registrations inside the `routes` module.
pub fn register_route_functions(m: &Bound<'_, PyModule>) -> PyResult<()> {
    // Delegate registration to each route module so that modules own their API surface.
    landing::register_routes(m)?;
    validation::register_routes(m)?;
    skymap::register_routes(m)?;
    distribution::register_routes(m)?;
    timeline::register_routes(m)?;
    insights::register_routes(m)?;
    trends::register_routes(m)?;
    compare::register_routes(m)?;
    visibility::register_routes(m)?;

    Ok(())
}
