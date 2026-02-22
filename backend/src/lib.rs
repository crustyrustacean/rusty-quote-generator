// src/lib.rs

// module declarations
pub mod configuration;
pub mod routes;
pub mod startup;
pub mod telemetry;

// re-exports
pub use configuration::*;
pub use startup::*;
pub use telemetry::*;
