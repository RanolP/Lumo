//! Project model (D-05): glob kind-suffixed files, cat them into one
//! namespace with additive language merge and strict collisions (D-22),
//! then DCE from the manifest pipelines (D-27/D-33).

pub mod dce;
pub mod fields;
pub mod first;
pub mod loader;
pub mod merge;
pub mod model;
pub mod praat;
