//! Repo automation tasks. Entry point is the `xtask` binary (`cargo xtask <task>`);
//! the task implementations live in library modules so integration tests can drive
//! them directly against temp directories without a subprocess.

pub mod fetch_assets;
