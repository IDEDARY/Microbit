//! `bevy_microbit` — a no_std Bevy backend for the BBC micro:bit V1.
//!
//! This crate moves every piece of micro:bit hardware interaction behind a
//! Bevy-idiomatic API so that game code reads exactly like a desktop Bevy app.
//! It is built on the official [`bevy_ecs`] and [`bevy_time`] crates with
//! `bevy_reflect` disabled to keep the footprint flash-friendly, and with a
//! thin [`app::App`] shell in place of the (too heavy) `bevy_app` crate.
//!
//! # Quick start
//!
//! ```ignore
//! use bevy_microbit::prelude::*;
//!
//! fn main() -> ! {
//!     App::new()
//!         .add_plugins((MicrobitPlugins, MyGamePlugin))
//!         .run()
//! }
//! ```
//!
//! The core plans:
//!
//! * **Schedules** — `Startup`, `Update` (game-facing) plus a backend `Tick`
//!   schedule that scans the LED matrix every 1 ms.
//! * **`Res<Time>`** — driven per frame by [`time::MicrobitTimePlugin`].
//! * **`Res<ButtonInput<GameButton>>`** — edge-detected A/B buttons.
//! * **`ResMut<FrameBuffer>`** — a hardware-agnostic 5x5 pixel buffer the game
//!   draws into; [`render::MicrobitRenderingPlugin`] scans it out to the LEDs.

#![no_std]
#![warn(missing_docs)]
#![forbid(unsafe_op_in_unsafe_fn)]

extern crate alloc;

pub mod app;
pub mod device;
pub mod framebuffer;
pub mod input;
pub mod render;
pub mod time;

pub use bevy_ecs;
pub use bevy_time;

/// Registrar for every platform plugin required by a micro:bit app.
///
/// Adding this plugin (e.g. `add_plugins((MicrobitPlugins, MyGamePlugin))`)
/// installs device discovery, time, input, and LED rendering in one go.
#[derive(Default)]
pub struct MicrobitPlugins;
impl app::Plugin for MicrobitPlugins {
    fn build(&self, app: &mut app::App) {
        app.add_plugin(device::MicrobitDevicePlugin)
            .add_plugin(time::MicrobitTimePlugin)
            .add_plugin(input::MicrobitInputPlugin)
            .add_plugin(render::MicrobitRenderingPlugin);
    }
}

/// Re-exports everything a typical game needs, mirroring `bevy::prelude`.
pub mod prelude {
    pub use crate::app::{App, Plugin, PostUpdate, PreUpdate, Startup, Tick, Update};
    pub use bevy_ecs::prelude::*;
    // The derive macros are not part of `bevy_ecs::prelude`, so re-export them
    // from the macros crate to keep `use bevy_microbit::prelude::*` ergonomic.
    pub use bevy_ecs_macros::{Component, Resource};
    pub use bevy_time::{Time, Timer, TimerMode};

    pub use crate::device::Entropy;
    pub use crate::framebuffer::FrameBuffer;
    pub use crate::input::{ButtonInput, GameButton};
    pub use crate::MicrobitPlugins;
}
