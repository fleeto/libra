//! Native-origin support. Wire decoding alone grants no trusted provenance.

pub mod approved_build;
pub mod attest;
pub mod codec;
pub mod registry;
// Prepared for the later single reasoning factory; not a public proof API.
pub(crate) mod verify;
