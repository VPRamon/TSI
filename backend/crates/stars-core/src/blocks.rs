//! Scheduling Blocks collection

use crate::error::{Error, Result};
use stars_core_sys as ffi;
use std::ffi::CString;
use std::ptr;

/// Collection of scheduling blocks
///
/// Represents a set of scheduling blocks (tasks, sequences, etc.) that can be
/// scheduled by the STARS scheduling algorithm.
///
/// # Example
///
/// ```rust,ignore
/// use stars_core::Blocks;
///
/// // Load from JSON string
/// let json = r#"{ "schedulingBlocks": [...] }"#;
/// let blocks = Blocks::from_json(json)?;
///
/// // Or load from file
/// let blocks = Blocks::from_file("schedule.json")?;
///
/// // Get count
/// println!("Loaded {} blocks", blocks.len()?);
/// ```
pub struct Blocks {
    handle: ffi::StarsBlocksHandle,
}

impl Blocks {
    /// Load scheduling blocks from a JSON string
    ///
    /// The JSON can be either:
    /// - An array of scheduling block objects
    /// - An object with a `schedulingBlocks` key containing the array
    pub fn from_json(json: &str) -> Result<Self> {
        let c_json =
            CString::new(json).map_err(|_| Error::InvalidInput("JSON contains null bytes".into()))?;
        let mut handle: ffi::StarsBlocksHandle = ptr::null_mut();

        let result = unsafe { ffi::stars_blocks_load_json(c_json.as_ptr(), &mut handle) };

        if result.is_ok() {
            Ok(Self { handle })
        } else {
            Err(Error::from_ffi_result(&result))
        }
    }

    /// Load scheduling blocks from a schedule JSON file
    pub fn from_file(file_path: &str) -> Result<Self> {
        let c_path = CString::new(file_path)
            .map_err(|_| Error::InvalidInput("Path contains null bytes".into()))?;
        let mut handle: ffi::StarsBlocksHandle = ptr::null_mut();

        let result = unsafe { ffi::stars_blocks_load_file(c_path.as_ptr(), &mut handle) };

        if result.is_ok() {
            Ok(Self { handle })
        } else {
            Err(Error::from_ffi_result(&result))
        }
    }

    /// Serialize blocks to JSON string
    pub fn to_json(&self) -> Result<String> {
        let mut out_json: *mut std::os::raw::c_char = ptr::null_mut();

        let result = unsafe { ffi::stars_blocks_to_json(self.handle, &mut out_json) };

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

    /// Get the number of blocks in the collection
    pub fn len(&self) -> Result<usize> {
        let mut count: usize = 0;

        let result = unsafe { ffi::stars_blocks_count(self.handle, &mut count) };

        if result.is_ok() {
            Ok(count)
        } else {
            Err(Error::from_ffi_result(&result))
        }
    }

    /// Check if the collection is empty
    pub fn is_empty(&self) -> Result<bool> {
        Ok(self.len()? == 0)
    }

    /// Get a single block by index as JSON
    pub fn get(&self, index: usize) -> Result<String> {
        let mut out_json: *mut std::os::raw::c_char = ptr::null_mut();

        let result = unsafe { ffi::stars_blocks_get_at(self.handle, index, &mut out_json) };

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

    /// Get a single block by index as a typed value
    pub fn get_as<T: serde::de::DeserializeOwned>(&self, index: usize) -> Result<T> {
        let json = self.get(index)?;
        let value: T = serde_json::from_str(&json)?;
        Ok(value)
    }

    /// Iterate over blocks as JSON strings
    pub fn iter(&self) -> BlocksIter<'_> {
        BlocksIter {
            blocks: self,
            index: 0,
            len: self.len().unwrap_or(0),
        }
    }

    /// Get the raw FFI handle (for internal use)
    pub(crate) fn handle(&self) -> ffi::StarsBlocksHandle {
        self.handle
    }
}

impl Drop for Blocks {
    fn drop(&mut self) {
        if !self.handle.is_null() {
            unsafe {
                ffi::stars_blocks_destroy(self.handle);
            }
        }
    }
}

// Blocks owns its handle and can be sent between threads
unsafe impl Send for Blocks {}

/// Iterator over scheduling blocks
pub struct BlocksIter<'a> {
    blocks: &'a Blocks,
    index: usize,
    len: usize,
}

impl<'a> Iterator for BlocksIter<'a> {
    type Item = Result<String>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index >= self.len {
            None
        } else {
            let result = self.blocks.get(self.index);
            self.index += 1;
            Some(result)
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let remaining = self.len.saturating_sub(self.index);
        (remaining, Some(remaining))
    }
}

impl<'a> ExactSizeIterator for BlocksIter<'a> {}
