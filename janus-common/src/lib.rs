mod config;

// Re-export all the things from the config module.
pub use config::*;

// Include if the `validation` feature is enabled.
#[cfg(feature = "validation")]
pub mod validation;

// Re-export all the things from the validation module.
#[cfg(feature = "validation")]
pub use validation::*;
