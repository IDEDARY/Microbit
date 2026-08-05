//! The microbit game binary.

#![no_std]
#![no_main]

extern crate alloc;

use bevy_microbit::prelude::*;
use cortex_m as _;
use cortex_m_rt::entry;
use embedded_alloc::LlffHeap as Heap;
//use panic_halt as _;
use panic_rtt_target as _;
use rtt_target::{rprintln, rtt_init_print};

mod game;

// A static heap backing used by the global allocator. Sized to fit the ECS
// world within the micro:bit's 16 KiB of RAM (see README for the budget).
const HEAP_SIZE: usize = 11 * 1024;

/// Global allocator instance, handed the static heap buffer in `main`.
#[global_allocator]
pub static HEAP: Heap = Heap::empty();

/// Device entry point, mirroring the official Bevy app shape.
#[entry]
fn main() -> ! {
    rtt_init_print!();
    rprintln!("microbit: boot");

    // Initialise the allocator over the reserved static buffer.
   unsafe {
        embedded_alloc::init!(HEAP, HEAP_SIZE);
    }

    rprintln!("microbit: Alloc init");

    // Assemble and run the application.
    let app = App::new()
        .add_plugins((MicrobitPlugins, game::GamePlugin));
        //.run()

    rprintln!("Tick! heap used {}", crate::HEAP.used());

    loop {}
}
