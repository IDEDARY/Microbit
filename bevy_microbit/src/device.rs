//! Core micro:bit hardware resources and their setup plugin.
//!
//! All of the physical periphery is captured in a single [`Device`] resource by
//! [`MicrobitDevicePlugin`] during the `Startup` schedule. Consumer plugins
//! (input, rendering) borrow that resource rather than touching the board
//! themselves, which keeps the hardware initialization in exactly one place.

use bevy_ecs::prelude::{Commands, Resource};

use crate::app::{App, Plugin, Startup};
use embedded_hal::digital::InputPin;
use microbit::board::{Board, Buttons};
use microbit::gpio::DisplayPins;
use microbit::hal::timer::Timer;
use microbit::hal::pac::TIMER0;
use microbit::hal::rng::Rng;
use tinyrand::{RandRange, Seeded, Wyrand};

/// Everything borrowed from the board in one owned bundle.
///
/// This is intentionally read/written only by the platform plugins; game code
/// never inspects it.
#[derive(Resource)]
pub struct Device {
    /// Raw LED matrix pins, driven by the rendering plugin.
    pub(crate) display_pins: DisplayPins,
    /// Raw button pins, read by the input plugin.
    pub(crate) buttons: Buttons,
    /// Timer used to pace the LED row-scan at 1 ms per tick.
    pub(crate) timer: Timer<TIMER0>,
}

// SAFETY: The micro:bit is a single-core system with no concurrent access to
// the hardware registers. The pin wrappers hold raw register addresses, which
// would ordinarily make the type `!Send`/`!Sync`, but sharing them between ECS
// systems running on the single core is sound.
unsafe impl Send for Device {}
unsafe impl Sync for Device {}

/// A deterministic, seedable random source used by the game.
///
/// Seeded once from the on-die nRF51 hardware RNG at startup.
#[derive(Resource)]
pub struct Entropy(Wyrand);
impl Entropy {
    /// Returns a value uniformly within `0..end`.
    pub fn next_below(&mut self, end: usize) -> usize {
        self.0.next_range(0..end)
    }
}

/// Discovers the micro:bit board and turns it into the [`Device`]/[`Entropy`]
/// resources. Must be registered before any other platform plugin.
pub struct MicrobitDevicePlugin;
impl Plugin for MicrobitDevicePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_device);
    }
}

/// Takes ownership of the board peripherals and inserts them as resources.
fn setup_device(mut commands: Commands) {
    // There is exactly one board; taking it twice is a programmer error.
    let board = Board::take().expect("micro:bit board already taken");

    // Seed the game RNG from hardware entropy, in exactly the same way the
    // original app did.
    let mut hw_rng = Rng::new(board.RNG);
    let entropy = Entropy(Wyrand::seed(hw_rng.random_u64()));

    commands.insert_resource(Device {
        display_pins: board.display_pins,
        buttons: board.buttons,
        timer: Timer::new(board.TIMER0),
    });
    commands.insert_resource(entropy);
}

/// Mirrors the original LED-to-pin wiring table so the renderer can map a
/// logical `(row, col)` onto the physical row/column pin indices.
pub(crate) const LED_LAYOUT: [[(usize, usize); 5]; 5] = [
    [(0, 0), (1, 3), (0, 1), (1, 4), (0, 2)],
    [(2, 3), (2, 4), (2, 5), (2, 6), (2, 7)],
    [(1, 1), (0, 8), (1, 2), (2, 8), (1, 0)],
    [(0, 7), (0, 6), (0, 5), (0, 4), (0, 3)],
    [(2, 2), (1, 6), (2, 0), (1, 5), (2, 1)],
];

/// Pulls a raw button pin level as a `bool` (`true` = pressed/low).
pub(crate) fn read_button_pin<P: InputPin>(pin: &mut P) -> bool {
    pin.is_low().unwrap_or(false)
}
