//! LED matrix rendering with per-pixel brightness.
//!
//! [`MicrobitRenderingPlugin`] owns the physical scanning. Once per `Tick`
//! schedule it takes the logical [`FrameBuffer`] and drives one physical row
//! of the matrix, realising per-pixel brightness through **binary code
//! modulation** (BCM).
//!
//! # Multiplexing: space x time
//!
//! *Space domain* — the micro:bit wires its 25 LEDs into a 3x9 row/column
//! grid (see [`crate::device::LED_LAYOUT`]). Only one of the three rows is
//! energised at a time, so we cycle `row 0..3`, one per `Tick` (~1 ms). This is
//! the classic spatial multiplex used by the existing driver.
//!
//! *Time domain* — layered on top, each row's dwell time is sliced into eight
//! bit-planes whose durations follow powers of two: `1, 2, 4, ... 128` units
//! of [`PWM_UNIT_US`]. For bit-plane `b`, a pixel's column is lit iff bit `b`
//! of its `u8` brightness is set. Summed across the eight planes, a pixel at
//! brightness `v` is lit for `v` out of every `255` units — i.e. `v/255` duty
//! cycle — yielding 256 apparent intensity levels with only eight pin updates
//! per row and no visible flicker (full-frame refresh ≈ 327 Hz).
//!
//! Total row dwell is `255 * PWM_UNIT_US` ≈ 1.02 ms, matching the legacy 1 ms
//! scan so the runner cadence and `FRAME_MILLIS` are unchanged.

use embedded_hal::delay::DelayNs;
use embedded_hal::digital::OutputPin;
use microbit::gpio::DisplayPins;
use tiny_ecs::prelude::*;

use crate::device::{Device, LED_LAYOUT};
use crate::framebuffer::FrameBuffer;

/// Duration of the smallest BCM time slot, in microseconds.
///
/// Eight bit-planes of weight `1, 2, 4, ..., 128` sum to `255` slots, so one
/// row dwell is `255 * PWM_UNIT_US` = 1.02 ms. Tune on-device for the desired
/// brightness/glow trade-off; keep it small enough that the full frame
/// (`3 * 255 * PWM_UNIT_US`) stays well above ~120 Hz to avoid flicker.
pub const PWM_UNIT_US: u32 = 4;

/// Number of bit-planes used by the binary code modulation.
pub const BCM_BITS: usize = 8;

/// The display row currently being scanned out to the pins.
///
/// Tracks the internal multiplexing phase; game code never needs it.
#[derive(Debug, Clone, Resource)]
pub struct RenderState {
    /// The physical row (0-2) being scanned this tick.
    pub row: usize,
}

impl Default for RenderState {
    /// Starts scanning from the first row.
    fn default() -> Self {
        Self { row: 0 }
    }
}

// Manual `Default`-like constructor so the plugin can build it once.
impl RenderState {
    /// Creates a `RenderState` at row 0.
    pub const fn new() -> Self {
        Self { row: 0 }
    }
}

/// Refreshes one row of the LED matrix every tick using BCM for brightness.
pub struct MicrobitRenderingPlugin;
impl<W: WorldApi> Plugin<W> for MicrobitRenderingPlugin
where
    W: HasResource<Device>
        + HasResource<FrameBuffer>
        + HasResource<RenderState>
{
    fn build(&self, app: &mut App<W>) {
        app.insert_resource(RenderState::new());
        app.insert_resource(FrameBuffer::new());
        app.add_system(crate::app::Tick, render_row);
    }
}

/// Drives a single physical row through its eight BCM bit-planes, then advances
/// to the next row.
///
/// For each bit-plane the columns are first cleared, the active row is
/// energised, and the columns whose corresponding pixel has that bit set are
/// sunk. The plane is held for `2^bit * PWM_UNIT_US` microseconds, weighting
/// higher bits exponentially so the summed on-time maps linearly to the `u8`
/// brightness value.
#[system]
fn render_row(mut device: ResMut<Device>, frame: Res<FrameBuffer>, mut state: ResMut<RenderState>) {
    // Split-borrow distinct fields so the pins and the timer can be used
    // concurrently within the bit-plane loop.
    let Device { display_pins, timer, .. } = &mut *device;
    let pins = display_pins;
    let row = state.row;

    for bit in 0..BCM_BITS {
        // Begin each plane from a clean slate so unlit columns cannot bleed
        // across bit boundaries.
        clear_pins(pins);
        set_row_high(pins, row);

        // Light every pixel of this row whose `bit`-th brightness bit is set.
        for (y, row_layout) in LED_LAYOUT.iter().enumerate() {
            for (x, &(layout_row, layout_col)) in row_layout.iter().enumerate() {
                if layout_row == row && ((frame.pixel(x, y) >> bit) & 1) == 1 {
                    set_col_low(pins, layout_col);
                }
            }
        }

        // Hold the plane for a duration proportional to its bit weight.
        timer.delay_us((1u32 << bit) * PWM_UNIT_US);
    }

    // Advance the scan pointer, wrapping back to the first row.
    state.row = if state.row == crate::app::ROW_COUNT - 1 {
        0
    } else {
        state.row + 1
    };
}

/// Drives every row pin low and every column pin high (all LEDs off).
fn clear_pins(pins: &mut DisplayPins) {
    let _ = pins.row1.set_low();
    let _ = pins.row2.set_low();
    let _ = pins.row3.set_low();
    let _ = pins.col1.set_high();
    let _ = pins.col2.set_high();
    let _ = pins.col3.set_high();
    let _ = pins.col4.set_high();
    let _ = pins.col5.set_high();
    let _ = pins.col6.set_high();
    let _ = pins.col7.set_high();
    let _ = pins.col8.set_high();
    let _ = pins.col9.set_high();
}

/// Energises the given multiplexed row (0-2).
fn set_row_high(pins: &mut DisplayPins, row: usize) {
    match row {
        0 => {
            let _ = pins.row1.set_high();
        }
        1 => {
            let _ = pins.row2.set_high();
        }
        2 => {
            let _ = pins.row3.set_high();
        }
        _ => unreachable!("row index out of range for micro:bit display"),
    }
}

/// Sinks the given column pin to light it.
fn set_col_low(pins: &mut DisplayPins, col: usize) {
    match col {
        0 => {
            let _ = pins.col1.set_low();
        }
        1 => {
            let _ = pins.col2.set_low();
        }
        2 => {
            let _ = pins.col3.set_low();
        }
        3 => {
            let _ = pins.col4.set_low();
        }
        4 => {
            let _ = pins.col5.set_low();
        }
        5 => {
            let _ = pins.col6.set_low();
        }
        6 => {
            let _ = pins.col7.set_low();
        }
        7 => {
            let _ = pins.col8.set_low();
        }
        8 => {
            let _ = pins.col9.set_low();
        }
        _ => unreachable!("column index out of range for micro:bit display"),
    }
}
