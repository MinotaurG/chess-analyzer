// crates/core/src/training/mod.rs
//! Training modules for chess improvement

pub mod coordinates;
pub mod openings;
pub mod visualization;

pub use coordinates::CoordinateTrainer;
pub use openings::{OpeningTrainer, OpeningLine, DrillResult};
pub use visualization::VisualizationDrill;
