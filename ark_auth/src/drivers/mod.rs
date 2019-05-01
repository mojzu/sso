//! # Drivers
//! Binary application drivers.
#[cfg(all(feature = "postgres", not(feature = "sqlite")))]
pub mod postgres;
#[cfg(all(feature = "sqlite", not(feature = "postgres")))]
pub mod sqlite;

// TODO(feature): Driver trait.
