#![allow(dead_code)]
#![allow(unknown_lints)]
#![allow(clippy::manual_is_multiple_of)]

pub mod charstate;
pub mod easing;
pub mod effects;
pub mod engine;
pub mod gradient;

#[cfg(test)]
mod tests;

pub use effects::{Effect, ALL_EFFECTS, EffectInfo};
pub use engine::{run_animation, Grid};
