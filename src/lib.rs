pub static DEBUG: bool = false;

pub mod activation;
pub mod categorize;
pub mod error;
pub mod generation;
pub mod neural_network;
pub mod node;
pub mod series;
#[cfg(test)]
pub mod tests;
mod utils;
