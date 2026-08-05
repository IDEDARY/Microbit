//! LED matrix rendering.
//!
//! [`MicrobitRenderingPlugin`] owns the physical row-scanning. Once per 1 ms
//! tick it takes the logical [`FrameBuffer`] and lights the corresponding
//! physical pins for one display row. This is the only module that knows about
//! the micro:bit pin wiring.

use bevy_ecs::prelude::{Res, ResMut, Resource};
use embedded_hal::delay::DelayNs;
use embedded_hal::digital::OutputPin;
use microbit::gpio::DisplayPins;

use crate::app::{App, Plugin, Tick};
use crate::device::{Device, LED_LAYOUT};
use crate::framebuffer::{FrameBuffer, WIDTH};

/// How many LED rows the display multiplexes across.
const ROW_COUNT: usize = 3;

/// The display row currently being scanned out to the pins.
///
/// Tracks the internal multiplexing phase; game code never needs it.
#[derive(Debug, Resource)]
pub struct RenderState {
    /// The physical row (0-2) being scanned this tick.
    pub(crate) row: usize,
}
impl Default for RenderState {
    /// Starts scanning from the first row.
    fn default() -> Self {
        Self { row: 0 }
    }
}

/// Refreshes one row of the LED matrix every tick.
pub struct MicrobitRenderingPlugin;
impl Plugin for MicrobitRenderingPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(RenderState::default());
        app.insert_resource(FrameBuffer::new());
        app.add_systems(Tick, render_row);
    }
}

/// Clears the pins, lights the requested pixels of the current row, paces the
/// scan, and advances to the next row.
fn render_row(
    mut device: ResMut<Device>,
    frame: Res<FrameBuffer>,
    mut state: ResMut<RenderState>,
) {
    let pins = &mut device.display_pins;

    // Pull every pin to its inactive state before driving the active row.
    clear_pins(pins);

    // Energise the row being drawn, then light the lit columns of that row.
    set_row_high(pins, state.row);
    for (y, _) in frame.matrix().iter().enumerate() {
        for (x, _) in (0..WIDTH).into_iter().enumerate() {
            if frame.pixel(x, y) && LED_LAYOUT[y][x].0 == state.row {
                set_col_low(pins, LED_LAYOUT[y][x].1);
            }
        }
    }

    // One tick is one ms of wall-clock time on the hardware.
    device.timer.delay_ms(1u32);

    // Advance the scan pointer, wrapping back to the first row.
    state.row = if state.row == ROW_COUNT - 1 { 0 } else { state.row + 1 };
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
