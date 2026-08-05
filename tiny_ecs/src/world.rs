//! The concrete `World`, schedule execution, entity lifetime, and the shared
//! `WorldApi` trait generated against by `define_world!`.
//!
//! A `World` is a *concrete* struct produced per-application by the
//! `define_world!` proc-macro rather than an open-ended dynamic registry:
//! each component/resource lives in its own statically-sized slot, eliminating
//! the `IndexMap`/`HashMap` bookkeeping that pushes `bevy_ecs` past the RAM
//! budget. Schedules are a bounded `heapless::LinearMap` keyed by
//! [`TypeId`](core::any::TypeId), so any crate can mint a label via
//! `#[derive(ScheduleLabel)]` without a central enum.

use crate::commands_buffer::CommandBuffer;
use crate::entity::Entity;
use crate::schedule::ScheduleLabel;
use crate::system::{ResourceInsRef, System};

/// Maximum number of schedules that may be registered.
pub const MAX_SCHEDULES: usize = 12;

/// Behaviour every concrete `World` produced by `define_world!` implements.
///
/// Schedules are keyed by [`TypeId`](core::any::TypeId) of the label type `L`,
/// so the methods below are generic over `L: ScheduleLabel` and take the label
/// as a zero-sized marker value (e.g. `world.run_schedule(Update)`).
pub trait WorldApi {
    /// Adds a schedule under label `L` if it does not already exist.
    fn add_schedule<L: ScheduleLabel>(&mut self, label: L);

    /// Adds a `system` to the schedule identified by label `L`.
    fn add_system<L: ScheduleLabel>(&mut self, label: L, system: System);

    /// Runs the schedule identified by label `L`, if it exists.
    fn run_schedule<L: ScheduleLabel>(&mut self, label: L);

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