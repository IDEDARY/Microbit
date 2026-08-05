//! The micro:bit game binary.
//!
//! Only hardware bootstrap lives here. Everything else — including the `App`,
//! plugins and systems — is normal Bevy code that could run on a desktop.

#![no_std]
#![no_main]

extern crate alloc;

use bevy_microbit::prelude::*;
use cortex_m as _;
use cortex_m_rt::entry;
use embedded_alloc::LlffHeap as Heap;
use panic_halt as _;

mod game;

// A static heap backing used by the global allocator. Sized to fit the ECS
// world within the micro:bit's 16 KiB of RAM (see README for the budget).
const HEAP_SIZE: usize = 6 * 1024;

/// Global allocator instance, handed the static heap buffer in `main`.
#[global_allocator]
static HEAP: Heap = Heap::empty();

/// Backing memory for the global allocator (stored in `.bss`).
static mut HEAP_MEM: [u8; HEAP_SIZE] = [0u8; HEAP_SIZE];

/// Device entry point, mirroring the official Bevy app shape.
#[entry]
fn main() -> ! {
    // Initialise the allocator over the reserved static buffer.
    unsafe {
        HEAP.init(&raw mut HEAP_MEM as *mut u8 as usize, HEAP_SIZE);
    }

    // Assemble and run the application. The embedded runner never returns,
    // so mark this point as unreachable.
    App::new()
        .add_plugins((MicrobitPlugins, game::GamePlugin))
        .run();
    unreachable!("the embedded runner never returns")
}
