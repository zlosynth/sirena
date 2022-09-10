#![no_main]
#![no_std]

use sirena_examples as _;

#[cortex_m_rt::entry]
fn main() -> ! {
    defmt::println!("Hello, world!");

    sirena_examples::exit()
}
