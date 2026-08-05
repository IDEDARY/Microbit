//! Schedules, schedule labels, and the predefined label set.
//!
//! A schedule is an ordered, bounded list of systems run sequentially. Each
//! schedule is keyed in the [`World`](crate::world::WorldApi) by the
//! [`TypeId`](core::any::TypeId) of its label type, so any crate can mint a
//! new label simply by declaring a unit struct and deriving
//! [`ScheduleLabel`] — no central enum required.
//!
//! The four conventional labels ([`Startup`], [`PreUpdate`], [`Update`],
//! [`PostUpdate`]) are predefined here so platform plugins and apps can share
//! them without ceremony.

use core::any::TypeId;

use heapless::Vec as HVec;

use crate::system::System;

/// Maximum number of systems per schedule.
pub const MAX_SYSTEMS_PER: usize = 32;

// ---------------------------------------------------------------------
// --- Schedule labels -------------------------------------------------

/// Marker trait for types that identify a schedule.
///
/// Implemented via `#[derive(ScheduleLabel)]` (or manually for the predefined
/// labels below). The trait carries only a `'static` bound; the actual keying
/// is done with [`TypeId::of`] at the call site, giving every label type a
/// unique compile-time identity without a central registry.
pub trait ScheduleLabel: 'static {}

/// The one-time startup schedule.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub struct Startup;
impl ScheduleLabel for Startup {}

/// Runs at the start of each frame, before [`Update`].
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub struct PreUpdate;
impl ScheduleLabel for PreUpdate {}

/// Runs once per frame; holds the gameplay systems.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub struct Update;
impl ScheduleLabel for Update {}

/// Runs at the end of each frame, after [`Update`].
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub struct PostUpdate;
impl ScheduleLabel for PostUpdate {}

// ---------------------------------------------------------------------
// --- Schedule --------------------------------------------------------

/// A schedule: an ordered, bounded list of systems.
pub struct Schedule {
    /// The systems to run, in registration order.
    systems: HVec<System, MAX_SYSTEMS_PER>,
}

impl Schedule {
    /// Creates an empty schedule.
    pub const fn new() -> Self {
        Self {
            systems: HVec::new(),
        }
    }

    /// Appends a system; saturates silently when full.
    pub fn add(&mut self, system: System) {
        let _ = self.systems.push(system);
    }

    /// Runs every system in registration order against the raw world pointer.
    pub fn run(&self, world: *mut ()) {
        for system in self.systems.iter() {
            system(world);
        }
    }

    /// Returns the number of systems registered.
    pub fn len(&self) -> usize {
        self.systems.len()
    }

    /// Returns `true` when no systems are registered.
    pub fn is_empty(&self) -> bool {
        self.systems.is_empty()
    }
}

impl Default for Schedule {
    fn default() -> Self {
        Self::new()
    }
}