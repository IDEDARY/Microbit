//! A single schedule: an ordered, bounded list of systems run sequentially.
//!
//! Schedules are keyed by a dynamic [`ScheduleLabel`](crate::system::ScheduleLabel)
//! in the [`World`](crate::world::WorldApi); each schedule holds a bounded
//! `heapless::Vec` of [`System`]s executed in registration order. There is no
//! executor graph and no parallelism — only the programmatic ordering the app
//! chooses when it calls `world.run_schedule(label)`.

use heapless::Vec as HVec;

use crate::system::System;

/// Maximum number of systems per schedule.
pub const MAX_SYSTEMS_PER: usize = 32;

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
    ///
    /// `world` is a `*mut ()` to keep schedules type-erased; the concrete
    /// `World` casts it back when invoking a system.
    pub fn run(&self, world: *mut ()) {
        for system in self.systems.iter() {
            // `System` is `fn(*mut ())` (a safe function pointer); calling it
            // is safe by the caller's invariant that `world` is valid.
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