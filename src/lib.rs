#![forbid(unsafe_code)]
#![warn(
    clippy::pedantic,
    clippy::nursery,
    clippy::unwrap_used,
    clippy::expect_used
)]
#![allow(dead_code, clippy::missing_errors_doc)]

mod chunk;
mod chunk_storage;
mod file;
pub mod server;
mod storage;

pub use storage::FileStorage;
