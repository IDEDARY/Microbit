//! The concrete `World`, schedule execution, entity lifetime, and the shared
//! `World` trait generated against by `define_world!`.
//!
//! A `World` is a *concrete* struct produced per-application by the
//! `define_world!` proc-macro rather than an open-ended dynamic registry:
//! each component/resource lives in its own statically-sized slot, eliminating
//! the `IndexMap`/`HashMap` bookkeeping that pushes `bevy_ecs` past the RAM
//! budget. Schedules are a bounded `SmallMap` keyed by the app's
//! [`ScheduleLabel`] type, with no executor or graph.

use crate::commands_buffer::CommandBuffer;
use crate::entity::Entity;
use crate::system::{ResourceInsRef, StandardSchedules, System};

/// Maximum number of schedules that may be registered.
///
/// Deliberately small: a micro:bit game uses a handful (Startup, PreUpdate,
/// Update, PostUpdate, Tick). Bump if an app needs more.
pub const MAX_SCHEDULES: usize = 12;

/// Maximum number of systems per schedule.
pub const MAX_SYSTEMS_PER: usize = 32;

// ---------------------------------------------------------------------
// --- World trait ------------------------------------------------------

/// Behaviour every concrete `World<L>` produced by `define_world!` implements.
///
/// `L` is the app's [`ScheduleLabel`] type, used as the key in the schedule
/// map. The trait pulls the shared logic (schedules, entity allocation,
/// command flushing) out of the generated struct, so the macro stays small.
pub trait WorldApi {
    /// The app's schedule label type, which must expose the standard labels.
    type Label: StandardSchedules;

    /// Adds a schedule under `label` if it does not already exist.
    fn add_schedule(&mut self, label: Self::Label);

    /// Adds a `system` to the schedule identified by `label`.
    fn add_system(&mut self, label: Self::Label, system: System);

    /// Runs the schedule identified by `label`, if it exists.
    fn run_schedule(&mut self, label: &Self::Label);

    /// Allocates a fresh entity id; returns `None` if the entity budget is
    /// exhausted.
    fn spawn_empty(&mut self) -> Option<Entity>;

    /// Inserts a resource of type `R`, replacing any previous instance.
    fn insert_resource<R: 'static>(&mut self, resource: R)
    where
        Self: ResourceInsRef<R>;

    /// Marks `entity` as dead, removing it from every column and freeing its
    /// id for reuse.
    fn despawn(&mut self, entity: Entity);

    /// Returns the number of live entities.
    fn entity_count(&self) -> usize;

    /// Drains the pending command buffer and applies it.
    fn flush_commands(&mut self);

    /// Returns a raw pointer to the command buffer (for the `Commands` param).
    fn commands_ptr(&mut self) -> *mut CommandBuffer;
}