//! A software frame buffer representing the 5x5 LED matrix.
//!
//! This type is intentionally hardware-agnostic: it is a plain grid of booleans
//! that the game draws into. The [`crate::render::MicrobitRenderingPlugin`] is
//! the only component that knows how to scan those pixels out to the physical
//! LEDs, so [`FrameBuffer`] could equally be presented by a desktop mock.

use bevy_ecs::prelude::Resource;

/// Width of the micro:bit LED matrix (5 columns).
pub const WIDTH: usize = 5;
/// Height of the micro:bit LED matrix (5 rows).
pub const HEIGHT: usize = 5;

/// A `WIDTH`x`HEIGHT` grid of on/off pixels, shared between the game and the
/// renderer as a Bevy resource.
#[derive(Clone, Debug, Resource)]
pub struct FrameBuffer {
    /// The pixel matrix, indexed as `grid[row][column]`.
    grid: [[bool; WIDTH]; HEIGHT],
}

impl Default for FrameBuffer {
    /// Creates an all-off frame buffer.
    fn default() -> Self {
        Self::new()
    }
}

impl FrameBuffer {
    /// Creates a new, fully cleared frame buffer.
    pub fn new() -> Self {
        Self {
            grid: [[false; WIDTH]; HEIGHT],
        }
    }

    /// Turns the whole display off.
    pub fn clear(&mut self) {
        self.grid = [[false; WIDTH]; HEIGHT];
    }

    /// Sets a single pixel, ignoring writes that fall outside the grid.
    pub fn set(&mut self, x: usize, y: usize, on: bool) {
        if let Some(row) = self.grid.get_mut(y)
            && let Some(cell) = row.get_mut(x)
        {
            *cell = on;
        }
    }

    /// Returns whether the pixel at `(x, y)` is lit. Out-of-bounds reads yield
    /// `false`.
    pub fn pixel(&self, x: usize, y: usize) -> bool {
        self.grid.get(y).and_then(|row| row.get(x)).copied().unwrap_or(false)
    }

    /// Fills an axis-aligned rectangle with `on`.
    pub fn fill_rect(&mut self, x: usize, y: usize, w: usize, h: usize, on: bool) {
        for dy in 0..h {
            for dx in 0..w {
                self.set(x + dx, y + dy, on);
            }
        }
    }

    /// Returns an immutable view over the internal pixel matrix.
    pub fn matrix(&self) -> &[[bool; WIDTH]; HEIGHT] {
        &self.grid
    }
}
