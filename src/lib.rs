#![cfg_attr(not(feature = "std"), no_std)]
#![allow(clippy::new_without_default)]
#![allow(clippy::let_and_return)]

#[cfg(test)]
#[macro_use]
extern crate approx;

#[cfg(test)]
#[macro_use]
extern crate lazy_static;

pub mod bitcrusher;
pub mod interpolation;
pub mod osc1;
pub mod osc2;
pub mod signal;
pub mod state_variable_filter;
pub mod taper;
pub mod tone;
pub mod wavetable_oscillator;
pub mod xfade;

#[cfg(feature = "std")]
pub mod all_pass_filter;
#[cfg(feature = "std")]
pub mod comb_filter;
#[cfg(feature = "std")]
pub mod delay;
#[cfg(feature = "std")]
pub mod noise;
#[cfg(feature = "std")]
pub mod ring_buffer;
#[cfg(feature = "std")]
pub mod spectral_analysis;
