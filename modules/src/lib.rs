#![cfg_attr(not(feature = "std"), no_std)]
#![allow(clippy::new_without_default)]

#[cfg(test)]
#[macro_use]
extern crate approx;

pub mod all_pass_filter;
pub mod comb_filter;
pub mod delay;
pub mod ring_buffer;
pub mod wavetable_oscillator;

#[cfg(feature = "std")]
pub mod spectral_analysis;
