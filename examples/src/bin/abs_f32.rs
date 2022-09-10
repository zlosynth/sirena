#![no_main]
#![no_std]

use sirena_examples as _;

#[cortex_m_rt::entry]
fn main() -> ! {
    defmt::println!("CMSIS abs_f32");

    let a = [1.0, -1.0];
    let mut b = [0.0; 2];
    sirena::cmsis::abs_f32(&a, &mut b);
    assert_eq!(b[0], 1.0);
    assert_eq!(b[1], 1.0);

    sirena_examples::exit()
}
