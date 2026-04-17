use qtty::{Degrees, Meters};
use siderust::calculus::altitude::AltitudePeriodsProvider;
use siderust::calculus::azimuth::AzimuthProvider;
use siderust::coordinates::centers::Geodetic;
use siderust::coordinates::frames::ECEF;
use siderust::coordinates::spherical::direction;
use siderust::time::ModifiedJulianDate;

use crate::api::{AltAzCurve, AltAzData, AltAzRequest, ScheduleId};

const SAMPLE_INTERVAL_MINUTES: f64 = 10.0;

pub fn compute_alt_az_data(
    schedule_id: ScheduleId,
    request: &AltAzRequest,
) -> Result<AltAzData, String> {
    if request.end_mjd <= request.start_mjd {
        return Err("end_mjd must be greater than start_mjd".to_string());
    }

    let observer = Geodetic::<ECEF>::new(
        Degrees::new(request.observatory.lon_deg),
        Degrees::new(request.observatory.lat_deg),
        Meters::new(request.observatory.height),
    );

    let duration_days = request.end_mjd - request.start_mjd;
    let sample_count = ((duration_days * 24.0 * 60.0) / SAMPLE_INTERVAL_MINUTES)
        .ceil()
        .max(1.0) as usize;
    let step_days = duration_days / sample_count as f64;

    let sample_times_mjd = (0..=sample_count)
        .map(|i| request.start_mjd + i as f64 * step_days)
        .collect::<Vec<_>>();

    let curves = request
        .targets
        .iter()
        .map(|target| {
            let subject = direction::ICRS::new(
                Degrees::new(target.target_ra_deg),
                Degrees::new(target.target_dec_deg),
            );

            let mut altitudes_deg = Vec::with_capacity(sample_times_mjd.len());
            let mut azimuths_deg = Vec::with_capacity(sample_times_mjd.len());

            for mjd in &sample_times_mjd {
                let mjd = ModifiedJulianDate::new(*mjd);
                altitudes_deg.push(subject.altitude_at(&observer, mjd).value().to_degrees());
                azimuths_deg.push(subject.azimuth_at(&observer, mjd).value().to_degrees());
            }

            AltAzCurve {
                original_block_id: target.original_block_id.clone(),
                block_name: target.block_name.clone(),
                priority: target.priority,
                altitudes_deg,
                azimuths_deg,
            }
        })
        .collect::<Vec<_>>();

    Ok(AltAzData {
        schedule_id: schedule_id.value(),
        sample_times_mjd,
        curves,
    })
}
