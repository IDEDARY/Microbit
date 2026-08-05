//! `bevy_microbit` — a no_std Bevy-flavored backend for the BBC micro:bit V1,
//! built on the [`tiny_ecs`] framework.
//!
//! Every piece of micro:bit hardware interaction lives behind a Bevy-idiomatic
//! API so game code reads like a desktop app. The backend is intentionally
//! free of heavyweight `bevy_ecs` machinery: `tiny_ecs` provides a bounded,
//! zero-alloc world and a generic `App`/`Plugin` shell, and this crate wires
//! the device, time, input, and LED rendering into it.
//!
//! The runner loop itself is *not* part of this crate — the application drives
//! `world.run_schedule(label)` and `world.flush_commands()` at the cadences it
//! chooses, keeping scheduling fully programmatic and hardware-agnostic.

#![no_std]
#![forbid(unsafe_op_in_unsafe_fn)]

pub mod app;
pub mod device;
pub mod framebuffer;
pub mod input;
pub mod render;
pub mod time;

pub use tiny_ecs;

/// Re-exports everything a typical game needs, mirroring `bevy::prelude`.
pub mod prelude {
    pub use tiny_ecs::prelude::*;
    pub use tiny_ecs::{Component, Resource};

    pub use crate::app::{microbit_runner, MicrobitPlugins, Tick};
    pub use crate::device::{Device, Entropy};
    pub use crate::framebuffer::FrameBuffer;
    pub use crate::input::{ButtonInput, ButtonKey, GameButton};
    pub use crate::render::RenderState;
    pub use crate::time::MicrobitTimePlugin;
}