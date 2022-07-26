#![no_std]
#![allow(clippy::new_without_default)]
#![allow(clippy::let_and_return)]

#[cfg(test)]
#[macro_use]
extern crate approx;

pub mod bitcrusher;
pub mod interpolation;
pub mod signal;
pub mod state_variable_filter;
pub mod taper;
pub mod tone;
pub mod white_noise;
pub mod xfade;

#[cfg(test)]
pub mod spectral_analysis;
