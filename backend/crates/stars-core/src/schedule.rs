//! Schedule result

use crate::blocks::Blocks;
use crate::context::Context;
use crate::error::{Error, Result};
use crate::periods::PossiblePeriods;
use crate::types::{ScheduleResult, ScheduleStats, SchedulingParams};
use stars_core_sys as ffi;
use std::ptr;

/// Schedule result from the scheduling algorithm
///
/// Contains the scheduled units, unscheduled blocks, and fitness score.
///
/// # Example
///
/// ```rust,ignore
/// use stars_core::{Context, Blocks, Schedule, SchedulingParams};
///
/// let ctx = Context::from_file("schedule.json")?;
/// let blocks = Blocks::from_file("schedule.json")?;
/// let params = SchedulingParams::default();
///
/// let schedule = Schedule::run(&ctx, &blocks, None, params)?;
///
/// // Get statistics
/// let stats = schedule.stats()?;
/// println!("Scheduled {}/{} blocks", stats.scheduled_count, stats.total_blocks);
/// println!("Fitness: {}", stats.fitness);
/// ```
pub struct Schedule {
    handle: ffi::StarsScheduleHandle,
}

impl Schedule {
    /// Run the scheduling algorithm
    ///
    /// # Arguments
    ///
    /// * `ctx` - STARS context with instrument and execution period
    /// * `blocks` - Collection of scheduling blocks
    /// * `periods` - Optional pre-computed possible periods (if None, computed internally)
    /// * `params` - Scheduling parameters
    pub fn run(
        ctx: &Context,
        blocks: &Blocks,
        periods: Option<&PossiblePeriods>,
        params: SchedulingParams,
    ) -> Result<Self> {
        let mut handle: ffi::StarsScheduleHandle = ptr::null_mut();
        let periods_handle = periods.map(|p| p.handle()).unwrap_or(ptr::null_mut());
        let ffi_params = params.into();

        let result = unsafe {
            ffi::stars_run_scheduler(
                ctx.handle(),
                blocks.handle(),
                periods_handle,
                ffi_params,
                &mut handle,
            )
        };

        if result.is_ok() {
            Ok(Self { handle })
        } else {
            Err(Error::from_ffi_result(&result))
        }
    }

    /// Export schedule to JSON string
    pub fn to_json(&self) -> Result<String> {
        let mut out_json: *mut std::os::raw::c_char = ptr::null_mut();

        let result = unsafe { ffi::stars_schedule_to_json(self.handle, &mut out_json) };

        if result.is_ok() {
            let json_str = unsafe {
                if out_json.is_null() {
                    return Err(Error::NullPointer);
                }
                let s = std::ffi::CStr::from_ptr(out_json)
                    .to_string_lossy()
                    .into_owned();
                ffi::stars_free_string(out_json);
                s
            };
            Ok(json_str)
        } else {
            Err(Error::from_ffi_result(&result))
        }
    }

    /// Export schedule as typed result
    pub fn to_result(&self) -> Result<ScheduleResult> {
        let json = self.to_json()?;
        let result: ScheduleResult = serde_json::from_str(&json)?;
        Ok(result)
    }

    /// Get schedule statistics
    pub fn stats(&self) -> Result<ScheduleStats> {
        let mut out_json: *mut std::os::raw::c_char = ptr::null_mut();

        let result = unsafe { ffi::stars_schedule_get_stats(self.handle, &mut out_json) };

        if result.is_ok() {
            let json_str = unsafe {
                if out_json.is_null() {
                    return Err(Error::NullPointer);
                }
                let s = std::ffi::CStr::from_ptr(out_json)
                    .to_string_lossy()
                    .into_owned();
                ffi::stars_free_string(out_json);
                s
            };
            let stats: ScheduleStats = serde_json::from_str(&json_str)?;
            Ok(stats)
        } else {
            Err(Error::from_ffi_result(&result))
        }
    }
}

impl Drop for Schedule {
    fn drop(&mut self) {
        if !self.handle.is_null() {
            unsafe {
                ffi::stars_schedule_destroy(self.handle);
            }
        }
    }
}

// Schedule owns its handle and can be sent between threads
unsafe impl Send for Schedule {}
