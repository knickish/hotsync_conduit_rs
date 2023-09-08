#![cfg(target_os = "windows")]
mod condmgr;
mod condmgr_extern;

mod syncmgr;
mod syncmgr_extern;

mod error;

pub use condmgr::{ConduitInstallation, ConduitManager};
pub use error::SyncManagerError;
pub use syncmgr::*;
