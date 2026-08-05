//! LED matrix rendering.
//!
//! [`MicrobitRenderingPlugin`] owns the physical row-scanning. Once per `Tick`
//! schedule it takes the logical [`FrameBuffer`] and lights the corresponding
//! physical pins for one display row. This is the only module that knows about
//! the micro:bit pin wiring.

use embedded_hal::delay::DelayNs;
use embedded_hal::digital::OutputPin;
use microbit::gpio::DisplayPins;
use tiny_ecs::prelude::*;

use crate::device::{Device, LED_LAYOUT};
use crate::framebuffer::{FrameBuffer, WIDTH};

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

/// Refreshes one row of the LED matrix every tick.
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

/// Clears the pins, lights the requested pixels of the current row, paces the
/// scan, and advances to the next row.
#[system]
fn render_row(mut device: ResMut<Device>, frame: Res<FrameBuffer>, mut state: ResMut<RenderState>) {
    let pins = &mut device.display_pins;

    // Pull every pin to its inactive state before driving the active row.
    clear_pins(pins);

    // Energise the row being drawn, then light the lit columns of that row.
    set_row_high(pins, state.row);
    for y in 0..crate::framebuffer::HEIGHT {
        for x in 0..WIDTH {
            if frame.pixel(x, y) && LED_LAYOUT[y][x].0 == state.row {
                set_col_low(pins, LED_LAYOUT[y][x].1);
            }
        }
    }

    // One tick is one ms of wall-clock time on the hardware.
    device.timer.delay_ms(1u32);

    // Advance the scan pointer, wrapping back to the first row.
    state.row = if state.row == crate::app::ROW_COUNT - 1 {
        0
    } else {
        state.row + 1
    };
}

// `MicrobitPlugins` is re-exported from here for back-compat with the old import
// shape; the canonical definition lives in `crate::app`.
// (No re-export now; kept as a marker comment.)
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