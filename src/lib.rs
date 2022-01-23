
/// Packing format
pub mod pack;

/// Drop-in replacements for std::fs::*;
pub mod dropin;
mod error;

pub use dropin::File;
pub use pack::BackPack;
pub use pack::RawFile;
pub use pack::InMemoryFile;
pub use pack::PackError;
pub use pack::Result;
