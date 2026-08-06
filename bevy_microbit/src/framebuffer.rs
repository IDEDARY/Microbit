//! A software frame buffer representing the 5x5 LED matrix.
//!
//! Hardware-agnostic: a plain grid of 8-bit brightness values that the game
//! draws into. The [`crate::render::MicrobitRenderingPlugin`] scans it out to
//! the physical LEDs using binary code modulation to realise per-pixel
//! brightness, so [`FrameBuffer`] could equally be presented by a desktop mock.

use tiny_ecs::Resource;

/// Width of the micro:bit LED matrix (5 columns).
pub const WIDTH: usize = 5;
/// Height of the micro:bit LED matrix (5 rows).
pub const HEIGHT: usize = 5;

/// Fully-off brightness.
pub const OFF: u8 = 0;
/// Fully-on brightness.
pub const MAX: u8 = u8::MAX;

/// A `WIDTH`x`HEIGHT` grid of per-pixel brightness values, shared between the
/// game and the renderer as a resource.
///
/// Each cell holds a `u8` in `0..=255`; the renderer duty-cycles the
/// corresponding LED so that the value maps linearly to perceived intensity.
#[derive(Clone, Debug, Resource)]
pub struct FrameBuffer {
    /// The pixel matrix, indexed as `grid[row][column]`.
    grid: [[u8; WIDTH]; HEIGHT],
}
impl Default for FrameBuffer {
    /// Creates an all-off frame buffer.
    fn default() -> Self {
        Self::new()
    }
}
impl FrameBuffer {
    /// Creates a new, fully cleared frame buffer.
    pub const fn new() -> Self {
        Self {
            grid: [[OFF; WIDTH]; HEIGHT],
        }
    }

    /// Turns the whole display off.
    pub fn clear(&mut self) {
        self.grid = [[OFF; WIDTH]; HEIGHT];
    }

    /// Sets a single pixel's brightness, ignoring writes that fall outside the grid.
    pub fn set(&mut self, x: usize, y: usize, brightness: u8) {
        if let Some(row) = self.grid.get_mut(y)
            && let Some(cell) = row.get_mut(x)
        {
            *cell = brightness;
        }
    }

    /// Returns the brightness of the pixel at `(x, y)`. Out-of-bounds reads
    /// yield [`OFF`].
    pub fn pixel(&self, x: usize, y: usize) -> u8 {
        self.grid.get(y).and_then(|row| row.get(x)).copied().unwrap_or(OFF)
    }

    /// Fills an axis-aligned rectangle with the given brightness.
    pub fn fill_rect(&mut self, x: usize, y: usize, w: usize, h: usize, brightness: u8) {
        for dy in 0..h {
            for dx in 0..w {
                self.set(x + dx, y + dy, brightness);
            }
        }
    }

    /// Returns an immutable view over the internal pixel matrix.
    pub fn matrix(&self) -> &[[u8; WIDTH]; HEIGHT] {
        &self.grid
    }
}
