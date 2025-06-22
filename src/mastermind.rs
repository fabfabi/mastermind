//#[macro_use]
// extern crate fstrings;

pub mod mastermind_gameplay;
pub mod mastermind_io;
mod mastermind_mechanics;
pub mod mastermind_solver;

pub use mastermind_gameplay::game;

pub use mastermind_mechanics::ConfigType;
