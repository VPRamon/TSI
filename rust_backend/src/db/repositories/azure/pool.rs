// Azure pool implementation removed — placeholder
#![allow(dead_code, unused_variables)]

use crate::db::config::DbConfig;
use tiberius::Config;

/// Placeholder indicating implementation removed.
pub(crate) fn _azure_pool_todo() -> ! {
    todo!("Azure pool implementation removed — TODO: re-implement")
}

/// Type alias kept for compatibility, though the underlying implementation is a placeholder.
pub type DbPool = ();

pub async fn build_tiberius_config(_config: &DbConfig) -> Result<Config, String> {
    todo!("Azure placeholder: build_tiberius_config")
}

pub async fn init_pool(_config: &DbConfig) -> Result<(), String> {
    todo!("Azure placeholder: init_pool")
}

pub fn get_pool() -> Result<&'static DbPool, String> {
    todo!("Azure placeholder: get_pool")
}
