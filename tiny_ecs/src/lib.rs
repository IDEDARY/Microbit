//! `tiny_ecs` — a bounded, zero-alloc, Bevy-flavored ECS for `no_std`
//! microcontroller targets.
//!
//! The job is to fit a Bevy-style app (Plugins, Resources, Systems, Components,
//! Schedules) inside the BBC micro:bit V1's 16 KiB of RAM, where `bevy_ecs`
//! alone spends most of the budget on archetype/schema bookkeeping. `tiny_ecs`
//! trades Bevy's dynamic archetype storage for per-component dense columns of
//! compile-time capacity, and trades its schedule executor graph for a simple
//! bounded `Label -> Schedule` map that the app drives itself.
//!
//! # Quick start
//!
//! Declare the `World`, `Plugin` and runner in your app crate:
//!
//! ```ignore
//! tiny_ecs::define_world! {
//!     pub struct World {
//!         entities: 64,
//!         schedules: 8,
//!         label: GameSchedule,
//!         components { player: Player [64], debris: Debris [64], }
//!         resources { frame: FrameBuffer, time: Time, }
//!     }
//! }
//!
//! #[derive(tiny_ecs_macros::ScheduleLabel, Default, PartialEq, Eq, Hash, Clone)]
//! enum GameSchedule { #[default] Startup, Update }
//!
//! fn main() -> ! {
//!     let mut world = World::<GameSchedule>::new();
//!     world.add_schedule(GameSchedule::Startup);
//!     world.add_system(GameSchedule::Update, tiny_ecs::system(update));
//!     World::run_schedule_raw(&mut world as *mut _ as *mut (), &GameSchedule::Update);
//! }
//! ```
//!
//! The `App`/`Plugin` shell mirrors Bevy's ergonomics on top of the concrete
//! `World`.

#![no_std]
#![warn(missing_docs)]
#![forbid(unsafe_op_in_unsafe_fn)]

pub mod app;
pub mod column;
pub mod commands;
pub mod commands_buffer;
pub mod entity;
pub mod schedule;
pub mod system;
pub mod time;
pub mod world;

// Re-export the proc-macros so users only depend on `tiny_ecs`.
// `Component`/`Resource` live in *both* namespaces here: the trait (type
// namespace, defined below) and the derive macro (macro namespace, re-exported
// from `tiny_ecs_macros`). The schedule-label trait is blanketed, so no
// `ScheduleLabel` derive is needed — types with the right supertrait bounds
// qualify automatically.
pub use tiny_ecs_macros::{define_world, system, Component, Resource};

/// Re-exports the `Time`/`Timer` family at the crate root for convenience.
pub use crate::time::{Time, Timer, TimerMode};

/// Marker trait for component types.
///
/// Implemented via `#[derive(Component)]`; the marker keeps the derive trivial
/// and column access is keyed by the `ColumnRef` impl that `define_world!`
/// generates per registered component.
pub trait Component: 'static {}

/// Marker trait for resource types.
///
/// Implemented via `#[derive(Resource)]`.
pub trait Resource: 'static {}

/// Re-exports the most-used names so `use tiny_ecs::prelude::*` reads like a
/// desktop Bevy import.
pub mod prelude {
    pub use crate::app::{App, Plugin, Plugins};
    pub use crate::column::{Column, ColumnOps};
    pub use crate::commands::{Commands, EntityTarget};
    pub use crate::commands_buffer::CommandBuffer;
    pub use crate::entity::Entity;
    pub use crate::schedule::{Schedule, MAX_SYSTEMS_PER};
    pub use crate::system::{
        ColumnRef, CommandsRef, Fetch, IntoSystem, Query, Res, ResMut, HasResource,
        ScheduleLabel, SpawnRef, StandardSchedules, System,
    };
    pub use crate::time::{Time, Timer, TimerMode};
    pub use crate::world::WorldApi;
    // Brings in the `Component`/`Resource` traits *and* their derive macros
    // (both namespaces at the crate root, since the macros are re-exported
    // there and the traits are defined there).
    pub use crate::{Component, Resource};
    pub use tiny_ecs_macros::{define_world, system};
}