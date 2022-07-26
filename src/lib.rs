#![no_std]
#![allow(clippy::new_without_default)]
#![allow(clippy::let_and_return)]

#[cfg(test)]
#[macro_use]
extern crate approx;

pub mod state_variable_filter;
pub mod white_noise;

#[cfg(test)]
pub mod spectral_analysis;
