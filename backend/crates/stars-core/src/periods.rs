//! Possible observation periods (prescheduler results)

use crate::blocks::Blocks;
use crate::context::Context;
use crate::error::{Error, Result};
use crate::types::BlockPossiblePeriods;
use stars_core_sys as ffi;
use std::ptr;

/// Computed possible observation periods
///
/// Contains the time periods during which each scheduling block can be observed,
/// taking into account instrument constraints, astronomical conditions, and time windows.
///
/// # Example
///
/// ```rust,ignore
/// use stars_core::{Context, Blocks, PossiblePeriods};
///
/// let ctx = Context::from_file("schedule.json")?;
/// let blocks = Blocks::from_file("schedule.json")?;
///
/// // Compute possible periods
/// let periods = PossiblePeriods::compute(&ctx, &blocks)?;
///
/// // Export as JSON
/// let json = periods.to_json()?;
/// ```
pub struct PossiblePeriods {
    handle: ffi::StarsPossiblePeriodsHandle,
}

impl PossiblePeriods {
    /// Compute possible periods for scheduling blocks
    ///
    /// This runs the STARS prescheduler which computes when each task can be observed
    /// given instrument constraints, astronomical conditions, and time windows.
    pub fn compute(ctx: &Context, blocks: &Blocks) -> Result<Self> {
        let mut handle: ffi::StarsPossiblePeriodsHandle = ptr::null_mut();

        let result = unsafe {
            ffi::stars_compute_possible_periods(ctx.handle(), blocks.handle(), &mut handle)
        };

        if result.is_ok() {
            Ok(Self { handle })
        } else {
            Err(Error::from_ffi_result(&result))
        }
    }

    /// Export possible periods to JSON string
    pub fn to_json(&self) -> Result<String> {
        let mut out_json: *mut std::os::raw::c_char = ptr::null_mut();

        let result = unsafe { ffi::stars_possible_periods_to_json(self.handle, &mut out_json) };

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

    /// Export possible periods as typed values
    pub fn to_vec(&self) -> Result<Vec<BlockPossiblePeriods>> {
        let json = self.to_json()?;
        let periods: Vec<BlockPossiblePeriods> = serde_json::from_str(&json)?;
        Ok(periods)
    }

    /// Get the raw FFI handle (for internal use)
    pub(crate) fn handle(&self) -> ffi::StarsPossiblePeriodsHandle {
        self.handle
    }
}

impl Drop for PossiblePeriods {
    fn drop(&mut self) {
        if !self.handle.is_null() {
            unsafe {
                ffi::stars_possible_periods_destroy(self.handle);
            }
        }
    }
}

// PossiblePeriods owns its handle and can be sent between threads
unsafe impl Send for PossiblePeriods {}
