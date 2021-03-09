#![cfg_attr(not(feature = "std"), no_std)]
#![allow(clippy::new_without_default)]
#![allow(clippy::let_and_return)]

#[cfg(test)]
#[macro_use]
extern crate approx;

#[cfg(test)]
#[macro_use]
extern crate lazy_static;

pub mod all_pass_filter;
pub mod comb_filter;
pub mod delay;
pub mod osc1;
pub mod osc2;
pub mod ring_buffer;
mod signal;
pub mod state_variable_filter;
pub mod taper;
mod tone;
pub mod wavetable_oscillator;
pub mod xfade;

#[cfg(feature = "std")]
pub mod noise;
pub mod spectral_analysis;
